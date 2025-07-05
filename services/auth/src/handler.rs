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

use crate::{
    db::{DBCLient, DBError},
    proto::Session,
    utils::{constant_time_equal, generate_secure_random_string, hash_secret},
};

const SESSION_EXPIRES_IN_SECONDS: i64 = 60 * 60 * 24; // 1 day

#[derive(Clone)]
pub struct Service {
    pub db: DBCLient,
}

type SessionToken = String;

impl Service {
    /// Creates a new session.
    ///
    /// # Errors
    /// - database error
    ///
    /// # Further readings
    /// <https://lucia-auth.com/sessions/basic>
    pub async fn create_session(&self) -> Result<SessionToken, CreateSessionError> {
        let now: DateTime<Utc> = Utc::now();

        let id = generate_secure_random_string();
        let secret = generate_secure_random_string();
        let secret_hash = hash_secret(&secret);

        self.db.insert_session(&id, &secret_hash, now).await?;

        let token = format!("{id}.{secret}");

        Ok(token)
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
    pub async fn validate_session_token(
        &self,
        token: &str,
    ) -> Result<Session, ValidateSessionTokenError> {
        let token_parts: Vec<_> = token.split('.').collect();
        if token_parts.len() != 2 {
            return Err(ValidateSessionTokenError::InvalidFormat);
        }

        let session_id = token_parts[0];
        println!("session id {session_id} token {token}");
        let session_secret = token_parts[1];

        let session = self.db.get_session(session_id).await?;

        let is_expired = Utc::now().signed_duration_since(session.created_at)
            >= Duration::seconds(SESSION_EXPIRES_IN_SECONDS);
        if is_expired {
            self.db.delete_session(&session.id).await?;
            return Err(ValidateSessionTokenError::Expired);
        }

        let token_secret_hash = hash_secret(session_secret);
        let valid_secret = constant_time_equal(&token_secret_hash, &session.secret_hash);
        if !valid_secret {
            return Err(ValidateSessionTokenError::SecretMismatch);
        }

        Ok(Session {
            token: token.to_owned(),
        })
    }
}

#[derive(Debug, Error)]
pub enum CreateSessionError {
    #[error("database error: {0}")]
    Database(#[from] DBError),
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ValidateSessionTokenError {
    #[error("invalid token format")]
    InvalidFormat,

    #[error("token secret mismatch")]
    SecretMismatch,

    #[error("token expired")]
    Expired,

    #[error("database error: {0}")]
    Database(#[from] DBError),
}
