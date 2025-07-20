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
use chrono::{DateTime, Duration, Utc};
use thiserror::Error;
use tonic::{Code, Request, Response, Status};
use tracing::instrument;

use crate::{
    db::{DBCLient, DBError},
    proto::{
        CreateSessionReq, CreateSessionResp, ValidateSessionReq, ValidateSessionResp,
        api_service_server::ApiService,
    },
    utils::{constant_time_equal, generate_secure_random_string, hash_secret},
};

const SESSION_EXPIRES_IN_SECONDS: i64 = 60 * 60 * 24; // 1 day

#[derive(Clone)]
pub struct Handler {
    pub db: DBCLient,
}

type SessionToken = String;

#[tonic::async_trait]
impl ApiService for Handler {
    /// Creates a new session.
    ///
    /// # Errors
    /// - database error
    ///
    /// # Further readings
    /// <https://lucia-auth.com/sessions/basic>
    #[instrument(skip_all)]
    async fn create_session(
        &self,
        req: Request<CreateSessionReq>,
    ) -> Result<Response<CreateSessionResp>, Status> {
        let req = req.into_inner();
        if req.user_id.is_empty() {
            return Err(CreateSessionErr::MissingUserUID.into());
        }

        let now: DateTime<Utc> = Utc::now();

        let id = generate_secure_random_string();
        let secret = generate_secure_random_string();
        let secret_hash = hash_secret(&secret);

        self.db
            .insert_session(&id, &secret_hash, &req.user_id, now)
            .await
            .map_err(CreateSessionErr::Database)?;

        let resp = CreateSessionResp {
            token: format!("{id}.{secret}"),
        };

        Ok(Response::new(resp))
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
    async fn validate_session(
        &self,
        req: Request<ValidateSessionReq>,
    ) -> Result<Response<ValidateSessionResp>, Status> {
        let token = req.into_inner().token;
        let token_parts: Vec<_> = token.split('.').collect();
        if token_parts.len() != 2 {
            return Err(ValidateSessionErr::InvalidFormat.into());
        }

        let session_id = token_parts[0];
        let session_secret = token_parts[1];

        let session = self
            .db
            .get_session(session_id)
            .await
            .map_err(ValidateSessionErr::Database)?;

        let is_expired = Utc::now().signed_duration_since(session.created_at)
            >= Duration::seconds(SESSION_EXPIRES_IN_SECONDS);
        if is_expired {
            self.db
                .delete_session(&session.id)
                .await
                .map_err(ValidateSessionErr::Database)?;
            return Err(ValidateSessionErr::Expired.into());
        }

        let token_secret_hash = hash_secret(session_secret);
        let valid_secret = constant_time_equal(&token_secret_hash, &session.secret_hash);
        if !valid_secret {
            return Err(ValidateSessionErr::SecretMismatch.into());
        }

        let resp = ValidateSessionResp {};

        Ok(Response::new(resp))
    }
}

#[derive(Debug, Error)]
pub enum CreateSessionErr {
    #[error("missing user id")]
    MissingUserUID,

    #[error("database error: {0}")]
    Database(#[from] DBError),
}

impl From<CreateSessionErr> for Status {
    fn from(err: CreateSessionErr) -> Self {
        let code = match err {
            CreateSessionErr::MissingUserUID => Code::InvalidArgument,
            CreateSessionErr::Database(_) => Code::Internal,
        };
        Status::new(code, err.to_string())
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ValidateSessionErr {
    #[error("invalid token format")]
    InvalidFormat,

    #[error("token secret mismatch")]
    SecretMismatch,

    #[error("token expired")]
    Expired,

    #[error("database error: {0}")]
    Database(#[from] DBError),
}

impl From<ValidateSessionErr> for Status {
    fn from(err: ValidateSessionErr) -> Self {
        let code = match err {
            ValidateSessionErr::InvalidFormat
            | ValidateSessionErr::SecretMismatch
            | ValidateSessionErr::Expired => Code::Unauthenticated,
            ValidateSessionErr::Database(_) => Code::Internal,
        };
        Status::new(code, err.to_string())
    }
}
