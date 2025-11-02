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
    error::{Error, OAuthError},
    proto::{
        GetOauthAccountReq, GetOauthAccountResp, HandleOauthCallbackReq, HandleOauthCallbackResp,
        LinkOauthAccountReq, LinkOauthAccountResp, OauthProvider, StartOauthLoginReq,
        StartOauthLoginResp,
    },
    utils::{GithubOAuth, Now, OAuthProvider, RandomValueGeneratorTrait, Session, SystemNow},
};
use crate::{
    error::DBError,
    proto::{
        CreateSessionReq, CreateSessionResp, DeleteSessionReq, DeleteSessionResp,
        ValidateSessionReq, ValidateSessionResp, api_service_server::ApiService,
    },
    utils::{GoogleOAuth, OAuthHelper, constant_time_equal, hash_secret},
};
use setup::{session::SESSION_TOKEN_EXPIRY_DURATION, validate_user_id};
use tonic::{Request, Response, Status};
use tracing::instrument;

#[derive(Clone)]
pub struct Handler<D, R, N> {
    pub db: D,
    pub google: GoogleOAuth<R>,
    pub github: GithubOAuth<R>,
    _now: PhantomData<N>,
}

impl<D, R> Handler<D, R, SystemNow> {
    pub fn new(db: D, google: GoogleOAuth<R>, github: GithubOAuth<R>) -> Self {
        Self {
            db,
            google,
            github,
            _now: PhantomData,
        }
    }
}

type SessionToken = String;

#[tonic::async_trait]
impl<D, R, N> ApiService for Handler<D, R, N>
where
    D: DBClient,
    R: RandomValueGeneratorTrait + Clone,
    N: Now,
{
    /// Creates a new session.
    ///
    /// # Errors
    /// - database error
    ///
    /// # Further readings
    /// <https://lucia-auth.com/sessions/basic>
    #[instrument(skip_all, fields(user_id), err)]
    async fn create_session(
        &self,
        req: Request<CreateSessionReq>,
    ) -> Result<Response<CreateSessionResp>, Status> {
        let req = req.into_inner();

        let user_id = validate_user_id(&req.user_id)?;

        let id = R::generate_secure_random_string();
        let secret = R::generate_secure_random_string();
        let token = format!("{id}.{secret}");

        let session = Session {
            id,
            secret_hash: hash_secret(&secret),
            created_at: N::now(),
            user_id: user_id,
            ..Default::default()
        };

        self.db
            .insert_session(session)
            .await
            .map_err(Error::InsertSession)?;

        Ok(Response::new(CreateSessionResp { token }))
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
    #[instrument(skip_all, err)]
    async fn validate_session(
        &self,
        req: Request<ValidateSessionReq>,
    ) -> Result<Response<ValidateSessionResp>, Status> {
        let token = req.into_inner().token;

        if token.is_empty() {
            return Err(Error::MissingToken.into());
        }

        let token_parts: Vec<_> = token.split('.').collect();
        if token_parts.len() != 2 {
            return Err(Error::InvalidToken.into());
        }

        let session_id = token_parts[0];
        let session_secret = token_parts[1];

        let session = self.db.get_session(session_id).await.map_err(|e| match e {
            DBError::NotFound(_) => Error::NotFound,
            _ => Error::GetSession(e),
        })?;

        if N::now() >= session.expires_at {
            let result = self.db.delete_session(&session.id).await;
            result.map_err(Error::DeleteSession)?;
            return Err(Error::ExpiredToken.into());
        }

        let mut should_refresh_cookie = false;
        if session.expires_at.signed_duration_since(N::now()) < SESSION_TOKEN_EXPIRY_DURATION / 2 {
            if let Some(new_expiry) = N::now().checked_add_signed(SESSION_TOKEN_EXPIRY_DURATION) {
                let _ = self.db.update_session(session_id, &new_expiry).await;
                should_refresh_cookie = true;
            }
        }

        let token_secret_hash = hash_secret(session_secret);
        let valid_secret = constant_time_equal(&token_secret_hash, &session.secret_hash);
        if !valid_secret {
            return Err(Error::SecretMismatch.into());
        }

        Ok(Response::new(ValidateSessionResp {
            user_id: session.user_id.to_string(),
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
    #[instrument(skip_all, err)]
    async fn delete_session(
        &self,
        req: Request<DeleteSessionReq>,
    ) -> Result<Response<DeleteSessionResp>, Status> {
        let token = req.into_inner().token;

        if token.is_empty() {
            return Err(Error::MissingToken.into());
        }

        let token_parts: Vec<_> = token.split('.').collect();
        if token_parts.len() != 2 {
            return Err(Error::InvalidToken.into());
        }

        let session_id = token_parts[0];

        self.db
            .delete_session(session_id)
            .await
            .map_err(Error::DeleteSession)?;

        Ok(Response::new(DeleteSessionResp {}))
    }

    /// Starts a oauth login.
    ///
    /// # Errors
    /// - generating authorization url
    #[instrument(skip_all, err)]
    async fn start_oauth_login(
        &self,
        req: Request<StartOauthLoginReq>,
    ) -> Result<Response<StartOauthLoginResp>, Status> {
        let req = req.into_inner();

        let state = OAuthHelper::<R>::generate_state();
        let (code_verifier, authorization_url) = match req.provider() {
            OauthProvider::Google => {
                let code_verifier = OAuthHelper::<R>::generate_code_verifier();
                let code_challenge = OAuthHelper::<R>::create_s256_code_challenge(&code_verifier);

                let authorization_url = self
                    .google
                    .generate_authorization_url(&state, &code_challenge)
                    .map_err(OAuthError::GenerateAuthorizationUrl)?;

                (code_verifier, authorization_url)
            }
            OauthProvider::Github => {
                let authorization_url = self
                    .github
                    .generate_authorization_url(&state, "")
                    .map_err(OAuthError::GenerateAuthorizationUrl)?;

                (String::new(), authorization_url)
            }
            _ => return Err(OAuthError::UnsupportedOauthProvider.into()),
        };

        Ok(Response::new(StartOauthLoginResp {
            state,
            code_verifier,
            authorization_url,
        }))
    }

    /// Handles a oauth login callback
    ///
    /// # Errors
    /// - validating authorization code
    /// - decoding the id token
    /// - upserting oauth token (db)
    #[instrument(skip_all, err)]
    async fn handle_oauth_callback(
        &self,
        req: Request<HandleOauthCallbackReq>,
    ) -> Result<Response<HandleOauthCallbackResp>, Status> {
        let req = req.into_inner();

        let (code, code_verifier) = (&req.code, &req.code_verifier);

        let account = match req.provider() {
            OauthProvider::Google => self.google.exchange_code(code, code_verifier).await,
            OauthProvider::Github => self.github.exchange_code(code, code_verifier).await,
            _ => return Err(OAuthError::UnsupportedOauthProvider.into()),
        }
        .map_err(OAuthError::ExchangeCode)?;

        let account = self
            .db
            .upsert_oauth_account(&account)
            .await
            .map_err(OAuthError::UpsertOauthAccount)?;

        return Ok(Response::new(HandleOauthCallbackResp {
            account_id: account.id,
            provider_user_name: account.provider_user_name.unwrap_or_default(),
            provider_user_email: account.provider_user_email.unwrap_or_default(),
            user_id: account.user_id.map(|e| e.to_string()).unwrap_or_default(),
        }));
    }

    /// Links a user_id to an oauth token.
    ///
    /// # Errors
    /// - missing oauth token id
    /// - missing user id
    /// - updating oauth token (db)
    #[instrument(skip_all, fields(user_id), err)]
    async fn link_oauth_account(
        &self,
        req: Request<LinkOauthAccountReq>,
    ) -> Result<Response<LinkOauthAccountResp>, Status> {
        let req = req.into_inner();

        let account_id = req.account_id;
        if account_id.is_empty() {
            return Err(Error::MissingOauthAccountID.into());
        }

        let user_id = validate_user_id(&req.user_id)?;

        self.db
            .update_oauth_account(&account_id, user_id)
            .await
            .map_err(Error::UpdateOauthAccount)?;

        Ok(Response::new(LinkOauthAccountResp {}))
    }

    #[instrument(skip_all, fields(user_id), err)]
    async fn get_oauth_account(
        &self,
        req: Request<GetOauthAccountReq>,
    ) -> Result<Response<GetOauthAccountResp>, Status> {
        let req = req.into_inner();

        let user_id = validate_user_id(&req.user_id)?;

        let account = self
            .db
            .get_oauth_account(user_id, req.provider())
            .await
            .map_err(Error::GetOauthAccount)?;

        Ok(Response::new(GetOauthAccountResp {
            access_token: account.access_token.unwrap_or_default(),
        }))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::utils::{
        OAuthAccount, Session,
        tests::{
            MockNow, MockRandomValueGenerator, assert_response, fixture_oauth_account,
            fixture_session, fixture_token, fixture_uuid,
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
                    user_id: fixture_uuid().to_string(),
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
            let service = Handler {
                db,
                google: GoogleOAuth::<MockRandomValueGenerator>::default(),
                github: GithubOAuth::<MockRandomValueGenerator>::default(),
                _now: PhantomData::<MockNow>,
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
            let service = Handler {
                db,
                google: GoogleOAuth::<MockRandomValueGenerator>::default(),
                github: GithubOAuth::<MockRandomValueGenerator>::default(),
                _now: PhantomData::<MockNow>,
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
        want_update_session_count: usize,
        want_delete_session_count: usize,
        want_resp: Result<ValidateSessionResp, Code>,
    }

    impl Default for ValidateSessionTestCase {
        fn default() -> Self {
            Self {
                given_req: ValidateSessionReq {
                    token: fixture_token(),
                },
                given_db_get_session: Ok(fixture_session(|_| {})),
                want_update_session_count: 0,
                want_delete_session_count: 0,
                want_resp: Ok(ValidateSessionResp {
                    user_id: fixture_uuid().to_string(),
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
            let service = Handler {
                db,
                google: GoogleOAuth::<MockRandomValueGenerator>::default(),
                github: GithubOAuth::<MockRandomValueGenerator>::default(),
                _now: PhantomData::<MockNow>,
            };

            // when
            let req = Request::new(self.given_req);
            let got = service.validate_session(req).await;

            // then
            assert_response(got, self.want_resp);

            assert_eq!(
                *service.db.update_session_count.lock().await,
                self.want_update_session_count,
                "update_session_count mismatch",
            );

            assert_eq!(
                *service.db.delete_session_count.lock().await,
                self.want_delete_session_count,
                "delete_session_count mismatch",
            );
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
            given_db_get_session: Err(DBError::NotFound(String::new())),
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
            want_delete_session_count: 1,
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
            want_update_session_count: 1,
            want_resp: Ok(ValidateSessionResp {
                user_id: fixture_uuid().to_string(),
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

    struct GetOauthAccountTestCase {
        given_req: GetOauthAccountReq,
        given_get_oauth_account: Result<OAuthAccount, DBError>,
        want_resp: Result<GetOauthAccountResp, Code>,
    }

    impl Default for GetOauthAccountTestCase {
        fn default() -> Self {
            Self {
                given_req: GetOauthAccountReq {
                    user_id: fixture_uuid().to_string(),
                    provider: OauthProvider::Google as i32,
                },
                want_resp: Ok(GetOauthAccountResp {
                    access_token: "access-token".to_string(),
                }),
                given_get_oauth_account: Ok(fixture_oauth_account(|_| {})),
            }
        }
    }

    impl GetOauthAccountTestCase {
        async fn run(self) {
            // given
            let db = MockDBClient {
                get_oauth_account: Mutex::new(Some(self.given_get_oauth_account)),
                ..Default::default()
            };
            let service = Handler {
                db,
                google: GoogleOAuth::<MockRandomValueGenerator>::default(),
                github: GithubOAuth::<MockRandomValueGenerator>::default(),
                _now: PhantomData::<MockNow>,
            };

            // when
            let req = Request::new(self.given_req);
            let got = service.get_oauth_account(req).await;

            // then
            assert_response(got, self.want_resp);
        }
    }

    #[tokio::test]
    async fn test_get_oauth_account_happy_path() {
        GetOauthAccountTestCase {
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_get_oauth_account_missing_user_id() {
        GetOauthAccountTestCase {
            given_req: GetOauthAccountReq {
                user_id: String::new(),
                ..Default::default()
            },
            want_resp: Err(Code::InvalidArgument),
            ..Default::default()
        }
        .run()
        .await;
    }
}
