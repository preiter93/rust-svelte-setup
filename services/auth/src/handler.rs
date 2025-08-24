//! # Session-based authentication:
//! - The user logs in with a username and password
//! - The server authenticates the user and generates a session token
//! - The session token is stored in the database together with user info
//! - The token is sent to the client and stored in a cookie or local storage
//! - For requests the client sends the session token
//! - The server fetches user id from the token via the database and authorizes the user
//!
//! # Further readings
//! <https://lucia-auth.com/sessions/basic>
use std::marker::PhantomData;

use crate::{
    db::DBClient,
    error::DeleteSessionErr,
    utils::{Now, RandomStringGenerator, SystemNow},
};
use shared::session::SESSION_TOKEN_EXPIRY_DURATION;
use tonic::{Request, Response, Status};
use tracing::instrument;

use crate::{
    error::{
        CreateSessionErr, DBError, HandleGoogleCallbackErr, StartGoogleLoginErr, ValidateSessionErr,
    },
    proto::{
        CreateSessionReq, CreateSessionResp, DeleteSessionReq, DeleteSessionResp,
        HandleGoogleCallbackReq, HandleGoogleCallbackResp, StartGoogleLoginReq,
        StartGoogleLoginResp, ValidateSessionReq, ValidateSessionResp,
        api_service_server::ApiService,
    },
    utils::{GoogleOAuth, OAuth, constant_time_equal, hash_secret},
};

#[derive(Clone)]
pub struct Handler<D, R, Now> {
    pub db: D,
    pub google: GoogleOAuth<R>,
    _now: PhantomData<Now>,
}

impl<D, R> Handler<D, R, SystemNow> {
    pub fn new(db: D, google: GoogleOAuth<R>) -> Self {
        Self {
            db,
            google,
            _now: PhantomData,
        }
    }
}

type SessionToken = String;

#[tonic::async_trait]
impl<D, R, N> ApiService for Handler<D, R, N>
where
    D: DBClient,
    R: RandomStringGenerator,
    N: Now,
{
    /// Creates a new session.
    ///
    /// # Errors
    /// - database error
    ///
    /// # Further readings
    /// <https://lucia-auth.com/sessions/basic>
    #[instrument(skip(self), err)]
    async fn create_session(
        &self,
        req: Request<CreateSessionReq>,
    ) -> Result<Response<CreateSessionResp>, Status> {
        let req = req.into_inner();
        if req.user_id.is_empty() {
            return Err(CreateSessionErr::MissingUserUID.into());
        }

        let now = N::now();

        let id = R::generate_secure_random_string();
        let secret = R::generate_secure_random_string();
        let secret_hash = hash_secret(&secret);

        self.db
            .insert_session(&id, &secret_hash, &req.user_id, now)
            .await
            .map_err(CreateSessionErr::Database)?;

        Ok(Response::new(CreateSessionResp {
            token: format!("{id}.{secret}"),
        }))
    }

    /// Validates a sessions token by parsing out the id and secret
    /// from the token, getting the session with the id, checking
    /// the expiration and comparing the secret against the hash.
    ///
    /// # Errors
    /// - token is malformed
    /// - session is expired
    /// - session secret is invalid
    /// - database error
    ///
    /// # Further readings
    /// <https://lucia-auth.com/sessions/basic>
    #[instrument(skip(self), err)]
    async fn validate_session(
        &self,
        req: Request<ValidateSessionReq>,
    ) -> Result<Response<ValidateSessionResp>, Status> {
        let token = req.into_inner().token;

        if token.is_empty() {
            return Err(ValidateSessionErr::MissingToken.into());
        }

        let token_parts: Vec<_> = token.split('.').collect();
        if token_parts.len() != 2 {
            return Err(ValidateSessionErr::InvalidFormat.into());
        }

        let session_id = token_parts[0];
        let session_secret = token_parts[1];

        let session = self.db.get_session(session_id).await.map_err(|e| match e {
            DBError::NotFound => ValidateSessionErr::NotFound,
            _ => ValidateSessionErr::Database(e),
        })?;

        if N::now() >= session.expires_at {
            let result = self.db.delete_session(&session.id).await;
            result.map_err(ValidateSessionErr::Database)?;
            return Err(ValidateSessionErr::Expired.into());
        }

        let mut should_refresh_cookie = false;
        if session.expires_at.signed_duration_since(N::now()) < SESSION_TOKEN_EXPIRY_DURATION / 2 {
            if let Some(new_expiry) = N::now().checked_add_signed(SESSION_TOKEN_EXPIRY_DURATION) {
                let _ = self.db.update_session(&session_id, &new_expiry).await;
                should_refresh_cookie = true;
            }
        }

        let token_secret_hash = hash_secret(session_secret);
        let valid_secret = constant_time_equal(&token_secret_hash, &session.secret_hash);
        if !valid_secret {
            return Err(ValidateSessionErr::SecretMismatch.into());
        }

        Ok(Response::new(ValidateSessionResp {
            user_id: session.user_id,
            should_refresh_cookie,
        }))
    }

    /// Deletes a session.
    ///
    /// # Errors
    /// - token is malformed
    /// - database error
    ///
    /// # Further readings
    /// <https://lucia-auth.com/sessions/basic>
    #[instrument(skip(self), err)]
    async fn delete_session(
        &self,
        req: Request<DeleteSessionReq>,
    ) -> Result<Response<DeleteSessionResp>, Status> {
        let token = req.into_inner().token;

        if token.is_empty() {
            return Err(DeleteSessionErr::MissingToken.into());
        }

        let token_parts: Vec<_> = token.split('.').collect();
        if token_parts.len() != 2 {
            return Err(DeleteSessionErr::InvalidFormat.into());
        }

        let session_id = token_parts[0];

        self.db
            .delete_session(session_id)
            .await
            .map_err(DeleteSessionErr::Database)?;

        Ok(Response::new(DeleteSessionResp {}))
    }

    /// Starts a google login.
    ///
    /// # Errors
    /// - generating authorization url
    #[instrument(skip(self), err)]
    async fn start_google_login(
        &self,
        _: Request<StartGoogleLoginReq>,
    ) -> Result<Response<StartGoogleLoginResp>, Status> {
        let (state, code_verifier) = (
            OAuth::<R>::generate_state(),
            OAuth::<R>::generate_code_verifier(),
        );

        let authorization_url = self
            .google
            .generate_authorization_url(&state, &code_verifier)
            .map_err(|_| StartGoogleLoginErr::AuthorizationUrl)?;

        Ok(Response::new(StartGoogleLoginResp {
            state,
            code_verifier,
            authorization_url,
        }))
    }

    /// Handles a google login callback
    ///
    /// # Errors
    /// - validating authorization code
    /// - decoding the id token
    #[instrument(skip(self), err)]
    async fn handle_google_callback(
        &self,
        req: Request<HandleGoogleCallbackReq>,
    ) -> Result<Response<HandleGoogleCallbackResp>, Status> {
        let req = req.into_inner();
        let tokens = self
            .google
            .validate_authorization_code(&req.code, &req.code_verifier)
            .await
            .map_err(|_| HandleGoogleCallbackErr::ValidateAuthorizationCode)?;

        let claims = self
            .google
            .decode_id_token(&tokens.id_token)
            .await
            .map_err(|_| HandleGoogleCallbackErr::DecodeIdToken)?;

        return Ok(Response::new(HandleGoogleCallbackResp {
            google_id: claims.sub,
            name: claims.name,
            email: claims.email,
        }));
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::utils::{
        Session,
        tests::{
            MockRandomStringGenerator, MockUTC, assert_response, fixture_session, fixture_token,
        },
    };
    use chrono::TimeZone;
    use tokio::sync::Mutex;
    use tonic::Code;

    use crate::db::test::MockDBClient;

    use super::*;

    struct CreateSessionTestCase {
        given_req: CreateSessionReq,
        given_db_insert_session: Result<(), DBError>,
        want_resp: Result<CreateSessionResp, Code>,
    }

    impl Default for CreateSessionTestCase {
        fn default() -> Self {
            Self {
                given_req: CreateSessionReq {
                    user_id: "user-id".to_string(),
                },
                given_db_insert_session: Ok(()),
                want_resp: Ok(CreateSessionResp {
                    token: fixture_token(),
                }),
            }
        }
    }

    impl CreateSessionTestCase {
        async fn run(self) {
            // given
            let db = MockDBClient {
                insert_session: Mutex::new(Some(self.given_db_insert_session)),
                ..Default::default()
            };
            let google = GoogleOAuth::<MockRandomStringGenerator>::default();
            let service = Handler {
                db,
                google,
                _now: PhantomData::<MockUTC>,
            };

            // when
            let req = Request::new(self.given_req);
            let got = service.create_session(req).await;

            // then
            assert_response(got, self.want_resp);
        }
    }

    #[tokio::test]
    async fn test_create_session_happy_path() {
        CreateSessionTestCase {
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_create_session_missing_user_id() {
        CreateSessionTestCase {
            given_req: CreateSessionReq {
                user_id: String::new(),
            },
            want_resp: Err(Code::InvalidArgument),
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_create_session_db_error() {
        CreateSessionTestCase {
            given_db_insert_session: Err(DBError::Unknown),
            want_resp: Err(Code::Internal),
            ..Default::default()
        }
        .run()
        .await;
    }

    struct DeleteSessionTestCase {
        given_req: DeleteSessionReq,
        given_db_delete_session: Result<(), DBError>,
        want_resp: Result<DeleteSessionResp, Code>,
    }

    impl Default for DeleteSessionTestCase {
        fn default() -> Self {
            Self {
                given_req: DeleteSessionReq {
                    token: fixture_token(),
                },
                given_db_delete_session: Ok(()),
                want_resp: Ok(DeleteSessionResp {}),
            }
        }
    }

    impl DeleteSessionTestCase {
        async fn run(self) {
            // given
            let db = MockDBClient {
                delete_session: Mutex::new(Some(self.given_db_delete_session)),
                ..Default::default()
            };
            let google = GoogleOAuth::<MockRandomStringGenerator>::default();
            let service = Handler {
                db,
                google,
                _now: PhantomData::<MockUTC>,
            };

            // when
            let req = Request::new(self.given_req);
            let got = service.delete_session(req).await;

            // then
            assert_response(got, self.want_resp);
        }
    }

    #[tokio::test]
    async fn test_delete_session_happy_path() {
        DeleteSessionTestCase {
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_delete_session_missing_token() {
        DeleteSessionTestCase {
            given_req: DeleteSessionReq {
                token: String::new(),
            },
            want_resp: Err(Code::InvalidArgument),
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_delete_session_invalid_format() {
        DeleteSessionTestCase {
            given_req: DeleteSessionReq {
                token: "invalid-format".to_string(),
            },
            want_resp: Err(Code::InvalidArgument),
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_delete_session_db_error() {
        DeleteSessionTestCase {
            given_db_delete_session: Err(DBError::Unknown),
            want_resp: Err(Code::Internal),
            ..Default::default()
        }
        .run()
        .await;
    }

    struct ValidateSessionTestCase {
        given_req: ValidateSessionReq,
        given_db_get_session: Result<Session, DBError>,
        want_resp: Result<ValidateSessionResp, Code>,
    }

    impl Default for ValidateSessionTestCase {
        fn default() -> Self {
            Self {
                given_req: ValidateSessionReq {
                    token: fixture_token(),
                },
                given_db_get_session: Ok(fixture_session(|_| {})),
                want_resp: Ok(ValidateSessionResp {
                    user_id: "user-id".to_string(),
                    should_refresh_cookie: false,
                }),
            }
        }
    }

    impl ValidateSessionTestCase {
        async fn run(self) {
            // given
            let db = MockDBClient {
                get_session: Mutex::new(Some(self.given_db_get_session)),
                delete_session: Mutex::new(Some(Ok(()))),
                update_session: Mutex::new(Some(Ok(()))),
                ..Default::default()
            };
            let google = GoogleOAuth::<MockRandomStringGenerator>::default();
            let service = Handler {
                db,
                google,
                _now: PhantomData::<MockUTC>,
            };

            // when
            let req = Request::new(self.given_req);
            let got = service.validate_session(req).await;

            // then
            assert_response(got, self.want_resp);
        }
    }

    #[tokio::test]
    async fn test_validate_session_happy_path() {
        ValidateSessionTestCase {
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_validate_session_missing_token() {
        ValidateSessionTestCase {
            given_req: ValidateSessionReq {
                token: String::new(),
            },
            want_resp: Err(Code::InvalidArgument),
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_validate_session_invalid_format() {
        ValidateSessionTestCase {
            given_req: ValidateSessionReq {
                token: "invalid-format".to_string(),
            },
            want_resp: Err(Code::InvalidArgument),
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_validate_session_not_found() {
        ValidateSessionTestCase {
            given_db_get_session: Err(DBError::NotFound),
            want_resp: Err(Code::Unauthenticated),
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_validate_session_expired() {
        ValidateSessionTestCase {
            given_db_get_session: Ok(fixture_session(|session| {
                session.expires_at = chrono::Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
            })),
            want_resp: Err(Code::Unauthenticated),
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_validate_session_almost_expired() {
        ValidateSessionTestCase {
            given_db_get_session: Ok(fixture_session(|session| {
                session.expires_at = chrono::Utc.with_ymd_and_hms(2020, 1, 2, 0, 0, 0).unwrap();
            })),
            want_resp: Ok(ValidateSessionResp {
                user_id: "user-id".to_string(),
                should_refresh_cookie: true,
            }),
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_validate_session_secret_mismatch() {
        ValidateSessionTestCase {
            given_db_get_session: Ok(fixture_session(|session| {
                session.secret_hash = vec![1];
            })),
            want_resp: Err(Code::Unauthenticated),
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_validate_session_db_error() {
        ValidateSessionTestCase {
            given_db_get_session: Err(DBError::Unknown),
            want_resp: Err(Code::Internal),
            ..Default::default()
        }
        .run()
        .await;
    }
}
