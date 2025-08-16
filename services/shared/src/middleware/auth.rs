use crate::session::{SESSION_TOKEN_COOKIE_KEY, SessionState};
use axum::{Router, body::Body};
use core::pin::Pin;
use http::{HeaderValue, Method, Request, Response, StatusCode};
use std::task::{Context, Poll};
use thiserror::Error;
use tonic::async_trait;
use tower::{Layer, Service, ServiceBuilder};

/// Adds session token authentication middleware to a http router.
pub fn add_session_auth_middleware_for_http<A>(
    router: Router,
    session_validator: A,
    no_auth: Vec<String>,
) -> Router
where
    A: SessionValidator + Clone + 'static,
{
    let layer = SessionAuthLayer {
        session_validator,
        no_auth,
    };
    router.layer(ServiceBuilder::new().layer(layer))
}

/// Trait for types that can validate a session token and return a user id.
#[async_trait]
pub trait SessionValidator: Send + Sync {
    /// Validates a given session token. Should be implemented by the auth client.
    ///
    /// # Returns
    /// - Some(SessionState) if the token is valid.
    /// - ValidateSessionErr::Unauthenticated if the session is missing,
    ///   the token is invalid or expired
    /// - ValidateSessionErr::Internal if there was an internal error, e.g.
    ///   connecting to a database
    async fn validate_session(&mut self, token: String)
    -> Result<SessionState, ValidateSessionErr>;
}

/// Authentication layer that validates a session token from incoming requests.  
///
/// After successful authentication the middleware inserts the user id
/// into the request's extensions allowing handlers to access the user.
#[derive(Clone)]
struct SessionAuthLayer<V> {
    /// The session validator used to check authentication.
    pub session_validator: V,

    /// Request uri paths for which authentication should be skipped.
    pub no_auth: Vec<String>,
}

/// Service produced by [`SessionAuthLayer`] that authenticates request with a session token.
#[derive(Clone)]
struct SessionAuthService<S, V> {
    /// The inner service.
    pub inner: S,

    /// The session validator used to check authentication.
    pub session_validator: V,

    /// Request uri paths for which authentication should be skipped.
    pub no_auth: Vec<String>,
}

impl<S, V: Clone> Layer<S> for SessionAuthLayer<V> {
    type Service = SessionAuthService<S, V>;

    fn layer(&self, inner: S) -> Self::Service {
        SessionAuthService {
            inner,
            session_validator: self.session_validator.clone(),
            no_auth: self.no_auth.clone(),
        }
    }
}

impl<S, ReqBody, Validator> Service<Request<ReqBody>> for SessionAuthService<S, Validator>
where
    S: Service<Request<ReqBody>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send,
    ReqBody: Send + 'static,
    Validator: SessionValidator + Clone + Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<ReqBody>) -> Self::Future {
        // Allow preflight
        if request.method() == Method::OPTIONS {
            return Box::pin(self.inner.call(request));
        }

        // Allow certain paths with no auth
        if self.no_auth.contains(&request.uri().path().to_string()) {
            return Box::pin(self.inner.call(request));
        }

        // Be careful when cloning inner services:
        //
        // https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let mut validator = self.session_validator.clone();

        Box::pin(async move {
            // Extract session token from cookies
            let Some(cookie) = request.headers().get("cookie") else {
                return unauthorized_response("missing cookies");
            };
            let Some(token) = extract_session_token(cookie) else {
                return unauthorized_response("missing session token");
            };

            // TODO: Check session cookie expiry and extend if needed

            // Validate token and store session state in request extensions
            match validator.validate_session(token).await {
                Ok(user_id) => {
                    request.extensions_mut().insert(user_id);
                    inner.call(request).await
                }
                Err(err) => unauthorized_response(&err.to_string()),
            }
        })
    }
}

fn extract_session_token(header_value: &HeaderValue) -> Option<String> {
    let Ok(cookie_str) = header_value.to_str() else {
        return None;
    };

    for cookie in cookie_str.split(';') {
        let cookie = cookie.trim();
        let mut parts = cookie.splitn(2, '=');
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            if key == SESSION_TOKEN_COOKIE_KEY {
                return Some(value.to_string());
            }
        }
    }

    None
}

fn unauthorized_response<S: Into<String>, E>(message: S) -> Result<Response<Body>, E> {
    Ok(Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body(Body::from(message.into()))
        .unwrap())
}

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

// Error for validate_session
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ValidateSessionErr {
    #[error("unauthenticated")]
    Unauthenticated,

    #[error("internal error")]
    Internal,
}
