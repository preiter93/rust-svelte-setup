use crate::{
    cookie::{extract_session_token_cookie, set_session_token_cookie},
    session::SessionState,
};
use axum::{body::Body, response::IntoResponse};
use core::pin::Pin;
use http::{Method, Request, Response, StatusCode, header::COOKIE};
use std::task::{Context, Poll};
use thiserror::Error;
use tonic::async_trait;
use tower::{Layer, Service};

/// Authentication layer that validates a session token from incoming requests.  
///
/// After successful authentication the middleware inserts the user id
/// into the request's extensions allowing handlers to access the user.
#[derive(Clone)]
pub struct SessionAuthLayer<V> {
    /// The session validator used to check authentication.
    pub session_validator: V,

    /// Request uri paths for which authentication should be skipped.
    pub no_auth_endpoints: Vec<String>,
}

impl<V> SessionAuthLayer<V> {
    /// Creates a new [`SessionAuthLayer`].
    pub fn new(session_validator: V, no_auth_endpoints: Vec<String>) -> Self {
        Self {
            session_validator,
            no_auth_endpoints,
        }
    }
}

/// Service produced by [`SessionAuthLayer`] that authenticates request with a session token.
#[derive(Clone)]
pub struct SessionAuthService<S, V> {
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
            no_auth: self.no_auth_endpoints.clone(),
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
            let Some(cookie) = request.headers().get(COOKIE) else {
                return Ok(Unauthorized("missing cookies").into_response());
            };
            let Some(token) = extract_session_token_cookie(cookie) else {
                return Ok(Unauthorized("missing session token").into_response());
            };

            // Validate token and store session state in request extensions
            match validator.validate_session(&token).await {
                Ok(s) => {
                    request.extensions_mut().insert(s.session_state);

                    let mut resp = inner.call(request).await?;

                    if s.should_refresh_cookie {
                        set_session_token_cookie(&mut resp, &token);
                    }

                    return Ok(resp);
                }
                Err(err) => Ok(Unauthorized(&err.to_string()).into_response()),
            }
        })
    }
}

/// Trait for types that can validate a session token and return a user id.
#[async_trait]
pub trait SessionValidator: Send + Sync {
    /// Validates a session token.
    ///
    /// # Returns
    /// - `Ok(ValidSession)` if the token is valid.
    /// - `Err(ValidateSessionErr::Unauthenticated)` if the session is missing,
    ///   the token is invalid, or expired.
    /// - `Err(ValidateSessionErr::Internal(_))` if an internal error occurred
    ///   (e.g., connecting to a database).
    async fn validate_session(&mut self, token: &str) -> Result<ValidSession, ValidateSessionErr>;
}

/// The result of a successful session validation.
#[derive(Debug, Clone)]
pub struct ValidSession {
    /// The current state of the session.
    pub session_state: SessionState,
    /// Whether the session cookie should be refreshed.
    pub should_refresh_cookie: bool,
}

impl ValidSession {
    /// Creates a new [`ValidSession`].
    pub fn new(session_state: SessionState, should_refresh_cookie: bool) -> Self {
        Self {
            session_state,
            should_refresh_cookie,
        }
    }
}

struct Unauthorized<Msg>(Msg);

impl<S: Into<String>> IntoResponse for Unauthorized<S> {
    fn into_response(self) -> Response<Body> {
        let body = Body::from(self.0.into());
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(body)
            .expect("failed to build response")
    }
}

pub(crate) type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

// Error for validate_session
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ValidateSessionErr {
    #[error("unauthenticated")]
    Unauthenticated,

    #[error("internal error")]
    Internal,
}
