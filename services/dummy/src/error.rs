use thiserror::Error;
use tonic::Code;
use tonic::Status;

/// Error for [`crate::proto::api_service_server::ApiService::create_entity`]
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CreateEntityErr {
    #[error("database error: {0}")]
    Database(#[from] DBError),
}

impl From<CreateEntityErr> for Status {
    fn from(err: CreateEntityErr) -> Self {
        let code = match err {
            CreateEntityErr::Database(_) => Code::Internal,
        };
        Status::new(code, err.to_string())
    }
}

/// Error for [`crate::proto::api_service_server::ApiService::get_entity`]
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum GetEntityErr {
    #[error("missing entity id")]
    MissingEntityId,

    #[error("not a uuid")]
    NotAUUID,

    #[error("entity not found: {0}")]
    NotFound(String),

    #[error("database error: {0}")]
    Database(#[from] DBError),
}

impl From<GetEntityErr> for Status {
    fn from(err: GetEntityErr) -> Self {
        let code = match err {
            GetEntityErr::MissingEntityId | GetEntityErr::NotAUUID => Code::InvalidArgument,
            GetEntityErr::NotFound(_) => Code::NotFound,
            GetEntityErr::Database(_) => Code::Internal,
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
