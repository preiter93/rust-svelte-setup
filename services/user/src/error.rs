use thiserror::Error;
use tonic::{Code, Status};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("missing user id")]
    MissingUserId,

    #[error("invalid user id: {0}")]
    InvalidUserId(String),

    #[error("missing user name")]
    MissingUserName,

    #[error("missing user email")]
    MissingUserEmail,

    #[error("user not found: {0}")]
    UserNotFound(String),

    #[error("get user error: {0}")]
    GetUser(DBError),

    #[error("insert user error: {0}")]
    InsertUser(DBError),
}

impl From<Error> for Status {
    fn from(err: Error) -> Self {
        let code = match err {
            Error::MissingUserName
            | Error::MissingUserEmail
            | Error::MissingUserId
            | Error::InvalidUserId(_) => Code::InvalidArgument,
            Error::UserNotFound(_) => Code::NotFound,
            Error::GetUser(_) | Error::InsertUser(_) => Code::Internal,
        };
        Status::new(code, err.to_string())
    }
}

// Database error
#[derive(Debug, Error)]
pub enum DBError {
    #[error("unknown error occured")]
    Unknown,

    #[error("internal database error: {0}")]
    Internal(#[from] tokio_postgres::Error),

    #[error("connection error: {0}")]
    Connection(#[from] deadpool_postgres::PoolError),

    #[error("entity not found")]
    NotFound,
}
