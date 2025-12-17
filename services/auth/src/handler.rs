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
    error::Error,
    oauth::{github::GithubOAuth, google::GoogleOAuth},
    proto::{
        GetOauthAccountReq, GetOauthAccountResp, HandleOauthCallbackReq, HandleOauthCallbackResp,
        LinkOauthAccountReq, LinkOauthAccountResp, OauthProvider, StartOauthLoginReq,
        StartOauthLoginResp,
    },
    utils::Session,
};
use crate::{
    error::DBError,
    proto::{
        CreateSessionReq, CreateSessionResp, DeleteSessionReq, DeleteSessionResp,
        ValidateSessionReq, ValidateSessionResp, api_service_server::ApiService,
    },
    utils::{constant_time_equal, hash_secret},
};
use common::{Now, SystemNow};
use oauth::{OAuth, OAuthProvider as _, RandomSource};
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
    R: RandomSource + Clone,
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

        let id = R::alphanumeric(24);
        let secret = R::alphanumeric(24);
        let token = format!("{id}.{secret}");

        let session = Session {
            id,
            secret_hash: hash_secret(&secret),
            created_at: N::now(),
            user_id,
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
        if session.expires_at.signed_duration_since(N::now()) < SESSION_TOKEN_EXPIRY_DURATION / 2
            && let Some(new_expiry) = N::now().checked_add_signed(SESSION_TOKEN_EXPIRY_DURATION)
        {
            let _ = self.db.update_session(session_id, &new_expiry).await;
            should_refresh_cookie = true;
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

        let state = OAuth::<R>::generate_state();
        let (code_verifier, authorization_url) = match req.provider() {
            OauthProvider::Google => {
                let verifier = OAuth::<R>::generate_code_verifier();
                let challenge = OAuth::<R>::create_s256_code_challenge(&verifier);

                let auth_url = self.google.generate_authorization_url(&state, &challenge)?;

                (verifier, auth_url)
            }
            OauthProvider::Github => {
                let auth_url = self.github.generate_authorization_url(&state, "")?;

                (String::new(), auth_url)
            }
            _ => return Err(Error::UnspecifiedOauthProvider.into()),
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
            _ => return Err(Error::UnspecifiedOauthProvider.into()),
        }?;

        let account = self
            .db
            .upsert_oauth_account(&account)
            .await
            .map_err(Error::UpsertOauthAccount)?;

        return Ok(Response::new(HandleOauthCallbackResp {
            account_id: account.id,
            external_user_name: account.external_user_name.unwrap_or_default(),
            external_user_email: account.external_user_email.unwrap_or_default(),
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
            external_user_id: account.external_user_id,
        }))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::utils::{
        OAuthAccount, Session,
        tests::{fixture_oauth_account, fixture_session, fixture_token, fixture_uuid},
    };
    use chrono::TimeZone;
    use common::mock::MockNow;
    use oauth::mock::MockRandom;
    use rstest::rstest;
    use testutils::assert_response;
    use tokio::sync::Mutex;
    use tonic::{Code, Request};

    use crate::db::test::MockDBClient;

    use super::*;

    // --------------------------
    // CreateSession
    // --------------------------
    #[rstest]
    #[case::happy_path(
        CreateSessionReq {
            user_id: fixture_uuid().to_string(),
        },
        Ok(()),
        Ok(CreateSessionResp {
            token: fixture_token(),
        })
    )]
    #[case::missing_user_id(
        CreateSessionReq {
            user_id: String::new(),
        },
        Ok(()),
        Err(Code::InvalidArgument)
    )]
    #[case::db_error(
        CreateSessionReq {
            user_id: fixture_uuid().to_string(),
        },
        Err(DBError::Unknown),
        Err(Code::Internal)
    )]
    #[tokio::test]
    async fn test_create_session(
        #[case] req: CreateSessionReq,
        #[case] db_result: Result<(), DBError>,
        #[case] want: Result<CreateSessionResp, Code>,
    ) {
        // given
        let db = MockDBClient {
            insert_session: Mutex::new(Some(db_result)),
            ..Default::default()
        };
        let service = Handler {
            db,
            google: GoogleOAuth::<MockRandom>::default(),
            github: GithubOAuth::<MockRandom>::default(),
            _now: PhantomData::<MockNow>,
        };

        // when
        let got = service.create_session(Request::new(req)).await;

        // then
        assert_response(got, want);
    }

    // --------------------------
    // DeleteSession
    // --------------------------
    #[rstest]
    #[case::happy_path(
        DeleteSessionReq {
            token: fixture_token(),
        },
        Ok(()),
        Ok(DeleteSessionResp {})
    )]
    #[case::missing_token(
        DeleteSessionReq {
            token: String::new(),
        },
        Ok(()),
        Err(Code::InvalidArgument)
    )]
    #[case::invalid_format(
        DeleteSessionReq {
            token: "invalid-format".to_string(),
        },
        Ok(()),
        Err(Code::InvalidArgument)
    )]
    #[case::db_error(
        DeleteSessionReq {
            token: fixture_token(),
        },
        Err(DBError::Unknown),
        Err(Code::Internal)
    )]
    #[tokio::test]
    async fn test_delete_session(
        #[case] req: DeleteSessionReq,
        #[case] db_result: Result<(), DBError>,
        #[case] want: Result<DeleteSessionResp, Code>,
    ) {
        // given
        let db = MockDBClient {
            delete_session: Mutex::new(Some(db_result)),
            ..Default::default()
        };
        let service = Handler {
            db,
            google: GoogleOAuth::<MockRandom>::default(),
            github: GithubOAuth::<MockRandom>::default(),
            _now: PhantomData::<MockNow>,
        };

        // when
        let got = service.delete_session(Request::new(req)).await;

        // then
        assert_response(got, want);
    }

    // --------------------------
    // ValidateSession
    // --------------------------
    #[rstest]
    #[case::happy_path(
        ValidateSessionReq {
            token: fixture_token(),
        },
        Ok(fixture_session(|_| {})),
        0,
        0,
        Ok(ValidateSessionResp {
            user_id: fixture_uuid().to_string(),
            should_refresh_cookie: false,
        })
    )]
    #[case::missing_token(
        ValidateSessionReq {
            token: String::new(),
        },
        Ok(fixture_session(|_| {})),
        0,
        0,
        Err(Code::InvalidArgument)
    )]
    #[case::invalid_format(
        ValidateSessionReq {
            token: "invalid-format".to_string(),
        },
        Ok(fixture_session(|_| {})),
        0,
        0,
        Err(Code::InvalidArgument)
    )]
    #[case::not_found(
        ValidateSessionReq {
            token: fixture_token(),
        },
        Err(DBError::NotFound(String::new())),
        0,
        0,
        Err(Code::Unauthenticated)
    )]
    #[case::expired(
        ValidateSessionReq {
            token: fixture_token(),
        },
        Ok(fixture_session(|session| {
            session.expires_at = chrono::Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
        })),
        0,
        1,
        Err(Code::Unauthenticated)
    )]
    #[case::almost_expired(
        ValidateSessionReq {
            token: fixture_token(),
        },
        Ok(fixture_session(|session| {
            session.expires_at = chrono::Utc.with_ymd_and_hms(2020, 1, 2, 0, 0, 0).unwrap();
        })),
        1,
        0,
        Ok(ValidateSessionResp {
            user_id: fixture_uuid().to_string(),
            should_refresh_cookie: true,
        })
    )]
    #[case::secret_mismatch(
        ValidateSessionReq {
            token: fixture_token(),
        },
        Ok(fixture_session(|session| {
            session.secret_hash = vec![1];
        })),
        0,
        0,
        Err(Code::Unauthenticated)
    )]
    #[case::db_error(
        ValidateSessionReq {
            token: fixture_token(),
        },
        Err(DBError::Unknown),
        0,
        0,
        Err(Code::Internal)
    )]
    #[tokio::test]
    async fn test_validate_session(
        #[case] req: ValidateSessionReq,
        #[case] db_result: Result<Session, DBError>,
        #[case] want_update_count: usize,
        #[case] want_delete_count: usize,
        #[case] want: Result<ValidateSessionResp, Code>,
    ) {
        // given
        let db = MockDBClient {
            get_session: Mutex::new(Some(db_result)),
            delete_session: Mutex::new(Some(Ok(()))),
            update_session: Mutex::new(Some(Ok(()))),
            ..Default::default()
        };
        let service = Handler {
            db,
            google: GoogleOAuth::<MockRandom>::default(),
            github: GithubOAuth::<MockRandom>::default(),
            _now: PhantomData::<MockNow>,
        };

        // when
        let got = service.validate_session(Request::new(req)).await;

        // then
        assert_response(got, want);

        assert_eq!(
            *service.db.update_session_count.lock().await,
            want_update_count,
            "update_session_count mismatch",
        );

        assert_eq!(
            *service.db.delete_session_count.lock().await,
            want_delete_count,
            "delete_session_count mismatch",
        );
    }

    // --------------------------
    // GetOauthAccount
    // --------------------------
    #[rstest]
    #[case::happy_path(
        GetOauthAccountReq {
            user_id: fixture_uuid().to_string(),
            provider: OauthProvider::Google as i32,
        },
        Ok(fixture_oauth_account(|_| {})),
        Ok(GetOauthAccountResp {
            external_user_id: "external-user-id".to_string(),
        })
    )]
    #[case::missing_user_id(
        GetOauthAccountReq {
            user_id: String::new(),
            ..Default::default()
        },
        Ok(fixture_oauth_account(|_| {})),
        Err(Code::InvalidArgument)
    )]
    #[tokio::test]
    async fn test_get_oauth_account(
        #[case] req: GetOauthAccountReq,
        #[case] db_result: Result<OAuthAccount, DBError>,
        #[case] want: Result<GetOauthAccountResp, Code>,
    ) {
        // given
        let db = MockDBClient {
            get_oauth_account: Mutex::new(Some(db_result)),
            ..Default::default()
        };
        let service = Handler {
            db,
            google: GoogleOAuth::<MockRandom>::default(),
            github: GithubOAuth::<MockRandom>::default(),
            _now: PhantomData::<MockNow>,
        };

        // when
        let got = service.get_oauth_account(Request::new(req)).await;

        // then
        assert_response(got, want);
    }
}
