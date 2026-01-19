//! Validate session endpoint
//!
//! Validates a session token by parsing out the id and secret
//! from the token, getting the session with the id, checking
//! the expiration and comparing the secret against the hash.

use tonic::{Request, Response, Status};

use crate::{
    db::DBClient,
    error::{DBError, Error},
    proto::{ValidateSessionReq, ValidateSessionResp},
    server::Server,
    utils::{constant_time_equal, hash_secret},
};
use common::Now;
use oauth::RandomSource;
use setup::session::SESSION_TOKEN_EXPIRY_DURATION;

impl<D, R, N> Server<D, R, N>
where
    D: DBClient,
    R: RandomSource + Clone,
    N: Now,
{
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
    pub async fn validate_session(
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
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use chrono::TimeZone;
    use common::mock::MockNow;
    use oauth::mock::MockRandom;
    use rstest::rstest;
    use testutils::assert_response;
    use tokio::sync::Mutex;
    use tonic::{Code, Request};

    use crate::{
        db::test::MockDBClient,
        error::DBError,
        oauth::{github::GithubOAuth, google::GoogleOAuth},
        proto::{ValidateSessionReq, ValidateSessionResp},
        server::Server,
        utils::{
            Session,
            tests::{fixture_session, fixture_token, fixture_uuid},
        },
    };

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
        let service = Server {
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
}
