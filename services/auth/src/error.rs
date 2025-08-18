use thiserror::Error;
use tonic::{Code, Status};

/// Error for [`crate::proto::api_service_server::ApiService::create_session`]
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

/// Error for [`crate::proto::api_service_server::ApiService::validate_session`]
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ValidateSessionErr {
    #[error("invalid token format")]
    InvalidFormat,

    #[error("token secret mismatch")]
    SecretMismatch,

    #[error("token expired")]
    Expired,

    #[error("token not found")]
    NotFound,

    #[error("database error: {0}")]
    Database(#[from] DBError),
}

impl From<ValidateSessionErr> for Status {
    fn from(err: ValidateSessionErr) -> Self {
        let code = match err {
            ValidateSessionErr::InvalidFormat
            | ValidateSessionErr::SecretMismatch
            | ValidateSessionErr::Expired
            | ValidateSessionErr::NotFound => Code::Unauthenticated,
            ValidateSessionErr::Database(_) => Code::Internal,
        };
        Status::new(code, err.to_string())
    }
}

/// Error for [`crate::proto::api_service_server::ApiService::delete_session`]
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DeleteSessionErr {
    #[error("invalid token format")]
    InvalidFormat,

    #[error("database error: {0}")]
    Database(#[from] DBError),
}

impl From<DeleteSessionErr> for Status {
    fn from(err: DeleteSessionErr) -> Self {
        let code = match err {
            DeleteSessionErr::InvalidFormat => Code::Unauthenticated,
            DeleteSessionErr::Database(_) => Code::Internal,
        };
        Status::new(code, err.to_string())
    }
}

/// Error for [`crate::proto::api_service_server::ApiService::start_google_login`]
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StartGoogleLoginErr {
    #[error("failed to generate authorization url")]
    AuthorizationUrl,
}

impl From<StartGoogleLoginErr> for Status {
    fn from(err: StartGoogleLoginErr) -> Self {
        let code = match err {
            _ => Code::Internal,
        };
        Status::new(code, err.to_string())
    }
}

/// Error for [`crate::proto::api_service_server::ApiService::handle_google_callback`]
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum HandleGoogleCallbackErr {
    #[error("failed to validate authorization code")]
    ValidateAuthorizationCode,

    #[error("failed to decode id token")]
    DecodeIdToken,
}

impl From<HandleGoogleCallbackErr> for Status {
    fn from(err: HandleGoogleCallbackErr) -> Self {
        let code = match err {
            _ => Code::Internal,
        };
        Status::new(code, err.to_string())
    }
}

// Database error
#[derive(Debug, Error)]
pub enum DBError {
    #[error("Database error: {0}")]
    Error(#[from] tokio_postgres::Error),

    #[error("connection pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),

    #[error("entity not found")]
    NotFound,

    #[error("conversion error: {0}")]
    Conversion(String),
}
