use axum::body::Body;
use futures::future::BoxFuture;
use http::{HeaderValue, Method, Request, Response, StatusCode};
use std::task::{Context, Poll};
use tonic::async_trait;
use tower::Service;

use crate::session::SessionState;

/// Middleware that performs authentication by validating
/// a session token from incoming requests.  
///
/// After successful authentication the middleware inserts the user id
/// into the request's extensions allowing handlers to access the user.
#[derive(Clone)]
pub struct AuthMiddleware<S, V> {
    pub inner: S,
    pub session_validator: V,
}

/// Trait for types that can validate a session token and return a user id.
#[async_trait]
pub trait SessionValidator: Send + Sync {
    /// Validate the given session token.
    ///
    /// # Returns
    /// - Some(UserId) if the token is valid.
    /// - None if the token is invalid or expired.
    async fn validate_session(&mut self, token: String) -> Option<SessionState>;
}

impl<S, B, V> Service<Request<B>> for AuthMiddleware<S, V>
where
    S: Service<Request<B>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
    V: SessionValidator + Clone + Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<B>) -> Self::Future {
        // Allow preflight
        if request.method() == Method::OPTIONS {
            return Box::pin(self.inner.call(request));
        }

        // Allow certain paths without auth
        const NO_AUTH: &[&str] = &["/auth/google/login", "/auth/google/callback"];
        if NO_AUTH.contains(&request.uri().path()) {
            return Box::pin(self.inner.call(request));
        }

        // Extract cookie
        let Some(cookie) = request.headers().get("cookie") else {
            return Box::pin(async { unauthorized_response() });
        };
        let Some(token) = extract_session_token(cookie) else {
            return Box::pin(async { unauthorized_response() });
        };

        // Validate token and add user id to extensions
        let mut inner = self.inner.clone();
        let mut validator = self.session_validator.clone();

        Box::pin(async move {
            let Some(user_id) = validator.validate_session(token).await else {
                return unauthorized_response();
            };

            request.extensions_mut().insert(user_id);
            inner.call(request).await
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
            if key == "session_token" {
                return Some(value.to_string());
            }
        }
    }

    None
}

fn unauthorized_response<E>() -> Result<Response<Body>, E> {
    Ok(Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body(Body::from("unauthenticated"))
        .unwrap())
}
