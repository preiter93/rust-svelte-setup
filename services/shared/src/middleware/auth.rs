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
#[derive(Debug, Clone, Default)]
pub struct ValidSession {
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
                Err(err) => Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Body::from(err.to_string()))
                    .unwrap()),
            }
        })
    }
}

pub(crate) type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Error for validate_session
#[derive(Debug, Clone, Error)]
#[non_exhaustive]
pub enum ValidateSessionErr {
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

    struct AuthMiddlewareTestCase<ReqBody> {
        given_request: Request<ReqBody>,
        given_validation_result: Result<ValidSession, ValidateSessionErr>,
        given_no_auth: Vec<String>,
        want_status_code: StatusCode,
        want_resp_set_cookies: Option<&'static str>,
    }

    impl<ReqBody: Default> Default for AuthMiddlewareTestCase<ReqBody> {
        fn default() -> Self {
            Self {
                given_request: Request::<ReqBody>::default(),
                given_validation_result: Ok(ValidSession::default()),
                given_no_auth: Vec::new(),
                want_status_code: StatusCode::INTERNAL_SERVER_ERROR,
                want_resp_set_cookies: None,
            }
        }
    }

    async fn run_auth_middleware_tets_case<ReqBody>(tc: AuthMiddlewareTestCase<ReqBody>)
    where
        ReqBody: Send + 'static,
    {
        // given
        let auth_client = AuthClient {
            response: tc.given_validation_result,
        };

        let mut service = SessionAuthService {
            inner: DummyService::default(),
            session_validator: auth_client,
            no_auth: tc.given_no_auth,
        };

        // when
        let resp = service.call(tc.given_request).await.unwrap();

        // then
        assert_eq!(resp.status(), tc.want_status_code);
        let resp_set_cookies = resp.headers().get(SET_COOKIE).map(|x| x.to_str().unwrap());
        assert_eq!(resp_set_cookies, tc.want_resp_set_cookies);
    }

    #[tokio::test]
    async fn test_authenticated() {
        let c = format!("{}={}", SESSION_TOKEN_COOKIE_KEY, "token");
        run_auth_middleware_tets_case(AuthMiddlewareTestCase {
            given_request: Request::builder().header("Cookie", c).body(()).unwrap(),
            want_status_code: StatusCode::OK,
            ..Default::default()
        })
        .await;
    }

    #[tokio::test]
    async fn test_authenticated_and_refresh_cookie() {
        let c = format!("{}={}", SESSION_TOKEN_COOKIE_KEY, "token");
        run_auth_middleware_tets_case(AuthMiddlewareTestCase {
            given_request: Request::builder().header("Cookie", c).body(()).unwrap(),
            given_validation_result: Ok(ValidSession {
                session_state: SessionState::default(),
                should_refresh_cookie: true,
            }),
            want_resp_set_cookies: Some(
                "session_token=token; Max-Age=604800; Path=/; HttpOnly; SameSite=Lax",
            ),
            want_status_code: StatusCode::OK,
            ..Default::default()
        })
        .await;
    }

    #[tokio::test]
    async fn test_skip_preflight_requests() {
        run_auth_middleware_tets_case(AuthMiddlewareTestCase {
            given_request: Request::builder().method("OPTIONS").body(()).unwrap(),
            want_status_code: StatusCode::OK,
            ..Default::default()
        })
        .await;
    }

    #[tokio::test]
    async fn test_skip_no_auth_endpoints() {
        run_auth_middleware_tets_case(AuthMiddlewareTestCase {
            given_request: Request::builder().uri("/no-auth").body(()).unwrap(),
            given_no_auth: vec![String::from("/no-auth")],
            want_status_code: StatusCode::OK,
            ..Default::default()
        })
        .await;
    }

    #[tokio::test]
    async fn test_unauthenticated_missing_cookies() {
        run_auth_middleware_tets_case(AuthMiddlewareTestCase {
            given_request: Request::builder().body(()).unwrap(),
            want_status_code: StatusCode::UNAUTHORIZED,
            ..Default::default()
        })
        .await;
    }

    #[tokio::test]
    async fn test_unauthenticated_missing_session_token_cookie() {
        run_auth_middleware_tets_case(AuthMiddlewareTestCase {
            given_request: Request::builder().header("Cookie", "").body(()).unwrap(),
            given_validation_result: Ok(ValidSession::default()),
            want_status_code: StatusCode::UNAUTHORIZED,
            ..Default::default()
        })
        .await;
    }

    #[tokio::test]
    async fn test_unauthenticated_invalid_token() {
        let session_token = "token";
        let value = format!("{}={}", SESSION_TOKEN_COOKIE_KEY, session_token);

        run_auth_middleware_tets_case(AuthMiddlewareTestCase {
            given_request: Request::builder().header("Cookie", value).body(()).unwrap(),
            given_validation_result: Err(ValidateSessionErr::Unauthenticated),
            want_status_code: StatusCode::UNAUTHORIZED,
            ..Default::default()
        })
        .await;
    }

    #[derive(Clone, Default)]
    struct DummyService;

    impl<ReqBody> Service<Request<ReqBody>> for DummyService
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
            let resp = Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())
                .unwrap();
            ready(Ok(resp))
        }
    }

    #[derive(Clone)]
    struct AuthClient {
        response: Result<ValidSession, ValidateSessionErr>,
    }

    #[async_trait]
    impl SessionValidator for AuthClient {
        async fn validate_session(&mut self, _: &str) -> Result<ValidSession, ValidateSessionErr> {
            return self.response.clone();
        }
    }
}
