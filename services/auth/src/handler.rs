use chrono::{DateTime, Duration, Utc};
use thiserror::Error;

use crate::{
    db::{DBCLient, DBError},
    proto::Session,
    utils::{constant_time_equal, datetime_to_prost, generate_secure_random_string, hash_secret},
};

const SESSION_EXPIRES_IN_SECONDS: i64 = 60 * 60 * 24;

#[derive(Clone)]
pub struct Server {
    pub db: DBCLient,
}

// pub struct SessionWithToken {
//     session: Session,
//     token: String,
// }

impl Server {
    /// [`Documentation`]: https://lucia-auth.com/sessions/basic
    ///
    /// # Errors
    /// - database error
    pub async fn create_session(&self) -> Result<(Session, String), CreateSessionError> {
        let now: DateTime<Utc> = Utc::now();
        let timestamp = datetime_to_prost(now);

        let id = generate_secure_random_string();
        let secret = generate_secure_random_string();
        let secret_hash = hash_secret(&secret);

        self.db.insert_session(&id, &secret_hash, now).await?;

        let token = format!("{id}.{secret}");

        Ok((
            Session {
                id,
                secret_hash,
                created_at: Some(timestamp),
            },
            token,
        ))
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
    /// [`Documentation`]: https://lucia-auth.com/sessions/basic
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

        Ok(session.into())
    }
}

#[derive(Debug, Error)]
pub enum CreateSessionError {
    #[error("database error: {0}")]
    Database(#[from] DBError),
}

#[derive(Debug, Error)]
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
