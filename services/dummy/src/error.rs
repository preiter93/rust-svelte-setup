use thiserror::Error;
use tonic::Code;
use tonic::Status;

/// Error for [`crate::proto::api_service_server::ApiService::create_entity`]
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("missing user id")]
    MissingUserId,
    #[error("invalid user id: {0}")]
    InvalidUserId(String),
    #[error("missing entity id")]
    MissingEntityId,
    #[error("invalid entity id: {0}")]
    InvalidEntityId(String),
    #[error("entity not found: {0}")]
    EntityNotFound(String),
    #[error("insert entity error: {0}")]
    InsertEntity(DBError),
    #[error("get entity error: {0}")]
    GetEntity(DBError),
}

impl From<Error> for Status {
    fn from(err: Error) -> Self {
        let code = match err {
            Error::MissingUserId
            | Error::InvalidUserId(_)
            | Error::MissingEntityId
            | Error::InvalidEntityId(_) => Code::InvalidArgument,
            Error::EntityNotFound(_) => Code::NotFound,
            Error::GetEntity(_) | Error::InsertEntity(_) => Code::Internal,
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
    #[error("Entity not found")]
    NotFound,
}
