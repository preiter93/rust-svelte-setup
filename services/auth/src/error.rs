use thiserror::Error;
use tonic::{Code, Status};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("missing user id")]
    MissingUserId,

    #[error("invalid user id: {0}")]
    InvalidUserId(String),

    #[error("missing oauth account id")]
    MissingOauthAccountID,

    #[error("missing token")]
    MissingToken,

    #[error("invalid token")]
    InvalidToken,

    #[error("token expired")]
    ExpiredToken,

    #[error("token secret mismatch")]
    SecretMismatch,

    #[error("token not found")]
    NotFound,

    #[error("get session error: {0}")]
    GetSession(DBError),

    #[error("delete session error: {0}")]
    DeleteSession(DBError),

    #[error("insert session error: {0}")]
    InsertSession(DBError),

    #[error("update oauth account error: {0}")]
    UpdateOauthAccount(DBError),

    #[error("get oauth account error: {0}")]
    GetOauthAccount(DBError),

    #[error("oauth provider is not specified")]
    UnspecifiedOauthProvider,

    #[error("upsert oauth account error: {0}")]
    UpsertOauthAccount(DBError),
}

impl From<Error> for Status {
    fn from(err: Error) -> Self {
        let code = match err {
            Error::InvalidToken
            | Error::MissingToken
            | Error::MissingUserId
            | Error::InvalidUserId(_)
            | Error::UnspecifiedOauthProvider
            | Error::MissingOauthAccountID => Code::InvalidArgument,
            Error::SecretMismatch | Error::ExpiredToken | Error::NotFound => Code::Unauthenticated,
            Error::GetSession(_)
            | Error::DeleteSession(_)
            | Error::InsertSession(_)
            | Error::UpdateOauthAccount(_)
            | Error::UpsertOauthAccount(_)
            | Error::GetOauthAccount(_) => Code::Internal,
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

    #[error("entity not found: {0}")]
    NotFound(String),
}
