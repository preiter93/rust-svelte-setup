use thiserror::Error;
use tonic::Code;
use tonic::Status;

/// Error for [`crate::proto::api_service_server::ApiService::create_user`]
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CreateUserErr {
    #[error("database error: {0}")]
    Database(#[from] DBError),

    #[error("missing email")]
    MissingEmail,

    #[error("missing email")]
    MissingName,
}

impl From<CreateUserErr> for Status {
    fn from(err: CreateUserErr) -> Self {
        let code = match err {
            CreateUserErr::MissingName | CreateUserErr::MissingEmail => Code::InvalidArgument,
            CreateUserErr::Database(_) => Code::Internal,
        };
        Status::new(code, err.to_string())
    }
}

/// Error for [`crate::proto::api_service_server::ApiService::get_user`]
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum GetUserErr {
    #[error("missing user id")]
    MissingUserId,

    #[error("not a uuid")]
    NotAUUID,

    #[error("user not found")]
    NotFound,

    #[error("database error: {0}")]
    Database(#[from] DBError),
}

impl From<GetUserErr> for Status {
    fn from(err: GetUserErr) -> Self {
        let code = match err {
            GetUserErr::MissingUserId | GetUserErr::NotAUUID => Code::InvalidArgument,
            GetUserErr::NotFound => Code::NotFound,
            GetUserErr::Database(_) => Code::Internal,
        };
        Status::new(code, err.to_string())
    }
}

// Database error
#[derive(Debug, Error)]
pub enum DBError {
    #[error("An unknown error occured")]
    Unknown,

    #[error("Database error: {0}")]
    Error(#[from] tokio_postgres::Error),

    #[error("Connection pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),

    #[error("Entity not found")]
    NotFound,

    #[error("Conversion error: {0}")]
    Conversion(String),
}
