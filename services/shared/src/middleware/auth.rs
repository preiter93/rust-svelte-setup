use crate::{
    cookie::{extract_session_token_cookie, set_session_token_cookie},
    session::SessionState,
};
use axum::body::Body;
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

/// Trait for types that can authenticate a session token.
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

/// The result of a successful session authentication.
#[derive(Debug, Clone, Default)]
pub struct AuthenticatedSession {
    /// The current state of the session.
    pub session_state: SessionState,
    /// Whether the session cookie should be refreshed.
    pub should_refresh_cookie: bool,
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
        if self.no_auth.contains(&request.uri().path().to_string()) {
            return Box::pin(self.inner.call(request));
        }

        // Be careful when cloning inner services:
        //
        // https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let mut validator = self.auth_client.clone();

        Box::pin(async move {
            // Extract session token from cookies
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

            // Authenticate session and store session state in request extensions
            match validator.authenticate_session(&token).await {
                Ok(s) => {
                    request.extensions_mut().insert(s.session_state);

                    let mut resp = inner.call(request).await?;

                    if s.should_refresh_cookie {
                        set_session_token_cookie(&mut resp, &token);
                    }

                    return Ok(resp);
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

#[cfg(test)]
mod tests {
    use crate::session::SESSION_TOKEN_COOKIE_KEY;
    use std::future::Ready;
    use std::future::ready;

    use http::header::SET_COOKIE;
    use tower::Service;

    use super::*;

    struct TestCase<ReqBody> {
        given_request: Request<ReqBody>,
        given_validation_result: Result<AuthenticatedSession, AuthenticateSessionErr>,
        given_no_auth: Vec<String>,
        want_status_code: StatusCode,
        want_resp_set_cookies: Option<&'static str>,
    }

    impl<ReqBody: Default> Default for TestCase<ReqBody> {
        fn default() -> Self {
            Self {
                given_request: Request::<ReqBody>::default(),
                given_validation_result: Ok(AuthenticatedSession::default()),
                given_no_auth: Vec::new(),
                want_status_code: StatusCode::INTERNAL_SERVER_ERROR,
                want_resp_set_cookies: None,
            }
        }
    }
    impl<ReqBody> TestCase<ReqBody>
    where
        ReqBody: Send + 'static,
    {
        async fn run(self) {
            // given
            let mut service = SessionAuthService {
                inner: MockService::default(),
                auth_client: MockAuthClient {
                    response: self.given_validation_result,
                },
                no_auth: self.given_no_auth,
            };

            // when
            let resp = service.call(self.given_request).await.unwrap();

            // then
            assert_eq!(resp.status(), self.want_status_code);
            let resp_set_cookies = resp.headers().get(SET_COOKIE).map(|x| x.to_str().unwrap());
            assert_eq!(resp_set_cookies, self.want_resp_set_cookies);
        }
    }

    #[tokio::test]
    async fn test_authenticated() {
        let c = format!("{}={}", SESSION_TOKEN_COOKIE_KEY, "token");
        TestCase {
            given_request: Request::builder().header("Cookie", c).body(()).unwrap(),
            want_status_code: StatusCode::OK,
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_authenticated_and_refresh_cookie() {
        let c = format!("{}={}", SESSION_TOKEN_COOKIE_KEY, "token");
        TestCase {
            given_request: Request::builder().header("Cookie", c).body(()).unwrap(),
            given_validation_result: Ok(AuthenticatedSession {
                session_state: SessionState::default(),
                should_refresh_cookie: true,
            }),
            want_resp_set_cookies: Some(
                "session_token=token; Max-Age=604800; Path=/; HttpOnly; SameSite=Lax",
            ),
            want_status_code: StatusCode::OK,
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_skip_preflight_requests() {
        TestCase {
            given_request: Request::builder().method("OPTIONS").body(()).unwrap(),
            want_status_code: StatusCode::OK,
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_skip_no_auth_endpoints() {
        TestCase {
            given_request: Request::builder().uri("/no-auth").body(()).unwrap(),
            given_no_auth: vec![String::from("/no-auth")],
            want_status_code: StatusCode::OK,
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_unauthenticated_missing_cookies() {
        TestCase {
            given_request: Request::builder().body(()).unwrap(),
            want_status_code: StatusCode::UNAUTHORIZED,
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_unauthenticated_missing_session_token_cookie() {
        TestCase {
            given_request: Request::builder().header("Cookie", "").body(()).unwrap(),
            given_validation_result: Ok(AuthenticatedSession::default()),
            want_status_code: StatusCode::UNAUTHORIZED,
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_unauthenticated_invalid_token() {
        let session_token = "token";
        let value = format!("{}={}", SESSION_TOKEN_COOKIE_KEY, session_token);

        TestCase {
            given_request: Request::builder().header("Cookie", value).body(()).unwrap(),
            given_validation_result: Err(AuthenticateSessionErr::Unauthenticated),
            want_status_code: StatusCode::UNAUTHORIZED,
            ..Default::default()
        }
        .run()
        .await;
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
