use crate::cookie::{extract_session_token_cookie, set_session_token_cookie};
use crate::session::SessionState;
use axum::body::Body;
use core::pin::Pin;
use http::{Method, Request, Response, StatusCode, header::COOKIE};
use std::task::{Context, Poll};
use thiserror::Error;
use tonic::async_trait;
use tower::{Layer, Service};

#[async_trait]
pub trait SessionAuthClient: Send + Sync {
    /// Authenticates a session token.
    ///
    /// # Returns
    /// - [`AuthenticatedSession`] if the token is valid.
    /// - [`AuthenticateSessionErr::Unauthenticated`] if the session is missing not in the db or the token is invalid/expired.
    /// - [`AuthenticateSessionErr::Internal`] on internal errors (e.g., connecting to a database).
    async fn authenticate_session(
        &mut self,
        token: &str,
    ) -> Result<AuthenticatedSession, AuthenticateSessionErr>;
}

/// Service produced by [`SessionAuthLayer`] that authenticates request with a session token.
#[derive(Clone)]
pub struct SessionAuthService<S, V> {
    /// The inner service.
    pub inner: S,

    /// The auth client with which to authenticate the session.
    pub auth_client: V,

    /// Request uri paths for which authentication should be skipped.
    pub no_auth: Vec<String>,
}

/// Authentication layer that validates a session token from incoming requests.
///
/// After successful authentication the middleware inserts the user id
/// into the request's extensions allowing handlers to access the user.
#[derive(Clone)]
pub struct SessionAuthLayer<A> {
    /// The session validator used to check authentication.
    pub session_auth_client: A,

    /// Request uri paths for which authentication should be skipped.
    pub no_auth_endpoints: Vec<String>,
}

impl<A> SessionAuthLayer<A> {
    /// Creates a new [`SessionAuthLayer`].
    pub fn new(session_auth_client: A, no_auth_endpoints: Vec<String>) -> Self {
        Self {
            session_auth_client,
            no_auth_endpoints,
        }
    }
}

/// The result of a successful session authentication.
#[derive(Debug, Clone, Default)]
pub struct AuthenticatedSession {
    /// The current state of the session.
    pub session_state: SessionState,
    /// Whether the session cookie should be refreshed.
    pub should_refresh_cookie: bool,
}

impl<S, V: Clone> Layer<S> for SessionAuthLayer<V> {
    type Service = SessionAuthService<S, V>;

    fn layer(&self, inner: S) -> Self::Service {
        SessionAuthService {
            inner,
            auth_client: self.session_auth_client.clone(),
            no_auth: self.no_auth_endpoints.clone(),
        }
    }
}

impl<S, ReqBody, Validator> Service<Request<ReqBody>> for SessionAuthService<S, Validator>
where
    S: Service<Request<ReqBody>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send,
    ReqBody: Send + 'static,
    Validator: SessionAuthClient + Clone + Send + 'static,
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
        let req_path = request.uri().path();
        if self.no_auth.iter().any(|p| matches_pattern(p, req_path)) {
            return Box::pin(self.inner.call(request));
        }

        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let mut validator = self.auth_client.clone();

        // Extract session token from cookies and authenticate the session
        Box::pin(async move {
            let Some(cookie) = request.headers().get(COOKIE) else {
                return Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Body::from("missing cookies"))
                    .unwrap());
            };
            let Some(token) = extract_session_token_cookie(cookie) else {
                return Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Body::from("missing session token"))
                    .unwrap());
            };

            match validator.authenticate_session(&token).await {
                Ok(s) => {
                    request.extensions_mut().insert(s.session_state);

                    let mut resp = inner.call(request).await?;

                    if s.should_refresh_cookie {
                        set_session_token_cookie(&mut resp, &token);
                    }

                    Ok(resp)
                }
                Err(err) => Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Body::from(err.to_string()))
                    .unwrap()),
            }
        })
    }
}

pub(crate) type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Error for [`SessionAuthClient::authenticate_session`].
#[derive(Debug, Clone, Error)]
#[non_exhaustive]
pub enum AuthenticateSessionErr {
    #[error("unauthenticated")]
    Unauthenticated,

    #[error("internal error")]
    Internal,
}

fn matches_pattern(pattern: &str, path: &str) -> bool {
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let path_parts: Vec<&str> = path.split('/').collect();

    if pattern_parts.len() != path_parts.len() {
        return false;
    }

    for (pattern, path) in pattern_parts.iter().zip(path_parts.iter()) {
        if *pattern == "*" {
            continue;
        }
        if pattern != path {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use crate::session::SESSION_TOKEN_COOKIE_KEY;
    use std::future::Ready;
    use std::future::ready;

    use http::header::SET_COOKIE;
    use rstest::rstest;
    use tower::Service;

    use super::*;

    #[rstest]
    #[case::authenticated(
        {
            let c = format!("{}={}", SESSION_TOKEN_COOKIE_KEY, "token");
            Request::builder().header("Cookie", c).body(()).unwrap()
        },
        Ok(AuthenticatedSession::default()),
        Vec::new(),
        StatusCode::OK,
        None
    )]
    #[case::authenticated_and_refresh_cookie(
        {
            let c = format!("{}={}", SESSION_TOKEN_COOKIE_KEY, "token");
            Request::builder().header("Cookie", c).body(()).unwrap()
        },
        Ok(AuthenticatedSession {
            session_state: SessionState::default(),
            should_refresh_cookie: true,
        }),
        Vec::new(),
        StatusCode::OK,
        Some("session_token=token; Max-Age=604800; Path=/; Secure; HttpOnly; SameSite=None")
    )]
    #[case::skip_preflight_requests(
        Request::builder().method("OPTIONS").body(()).unwrap(),
        Ok(AuthenticatedSession::default()),
        Vec::new(),
        StatusCode::OK,
        None
    )]
    #[case::skip_no_auth_endpoints(
        Request::builder().uri("/no-auth").body(()).unwrap(),
        Ok(AuthenticatedSession::default()),
        vec![String::from("/no-auth")],
        StatusCode::OK,
        None
    )]
    #[case::skip_no_auth_endpoints_with_wildcard(
        Request::builder().uri("/google/no-auth").body(()).unwrap(),
        Ok(AuthenticatedSession::default()),
        vec![String::from("/*/no-auth")],
        StatusCode::OK,
        None
    )]
    #[case::unauthenticated_missing_cookies(
        Request::builder().body(()).unwrap(),
        Ok(AuthenticatedSession::default()),
        Vec::new(),
        StatusCode::UNAUTHORIZED,
        None
    )]
    #[case::unauthenticated_missing_session_token_cookie(
        Request::builder().header("Cookie", "").body(()).unwrap(),
        Ok(AuthenticatedSession::default()),
        Vec::new(),
        StatusCode::UNAUTHORIZED,
        None
    )]
    #[case::unauthenticated_invalid_token(
        {
            let session_token = "token";
            let value = format!("{}={}", SESSION_TOKEN_COOKIE_KEY, session_token);
            Request::builder().header("Cookie", value).body(()).unwrap()
        },
        Err(AuthenticateSessionErr::Unauthenticated),
        Vec::new(),
        StatusCode::UNAUTHORIZED,
        None
    )]
    #[tokio::test]
    async fn test_auth_middleware(
        #[case] request: Request<()>,
        #[case] validation_result: Result<AuthenticatedSession, AuthenticateSessionErr>,
        #[case] no_auth: Vec<String>,
        #[case] want_status: StatusCode,
        #[case] want_set_cookies: Option<&str>,
    ) {
        // given
        let mut service = SessionAuthService {
            inner: MockService::default(),
            auth_client: MockAuthClient {
                response: validation_result,
            },
            no_auth,
        };

        // when
        let resp = service.call(request).await.unwrap();

        // then
        assert_eq!(resp.status(), want_status);
        let resp_set_cookies = resp.headers().get(SET_COOKIE).map(|x| x.to_str().unwrap());
        assert_eq!(resp_set_cookies, want_set_cookies);
    }

    #[derive(Clone, Default)]
    struct MockService;

    impl<ReqBody> Service<Request<ReqBody>> for MockService
    where
        ReqBody: Send + 'static,
    {
        type Response = Response<Body>;
        type Error = std::convert::Infallible;
        type Future = Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _req: Request<ReqBody>) -> Self::Future {
            ready(Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())
                .unwrap()))
        }
    }

    #[derive(Clone)]
    struct MockAuthClient {
        response: Result<AuthenticatedSession, AuthenticateSessionErr>,
    }

    #[async_trait]
    impl SessionAuthClient for MockAuthClient {
        async fn authenticate_session(
            &mut self,
            _: &str,
        ) -> Result<AuthenticatedSession, AuthenticateSessionErr> {
            return self.response.clone();
        }
    }
}
