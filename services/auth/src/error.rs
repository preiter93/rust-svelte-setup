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
    GetOauthAccount(#[from] DBError),
}

impl From<Error> for Status {
    fn from(err: Error) -> Self {
        let code = match err {
            Error::InvalidToken
            | Error::MissingToken
            | Error::MissingUserId
            | Error::InvalidUserId(_)
            | Error::MissingOauthAccountID => Code::InvalidArgument,
            Error::SecretMismatch | Error::ExpiredToken | Error::NotFound => Code::Unauthenticated,
            Error::GetSession(_)
            | Error::DeleteSession(_)
            | Error::InsertSession(_)
            | Error::UpdateOauthAccount(_)
            | Error::GetOauthAccount(_) => Code::Internal,
        };
        Status::new(code, err.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum OAuthError {
    #[error("failed to exchange code: {0}")]
    ExchangeCode(#[from] ExchangeCodeErr),
    #[error("failed to generate authorization url")]
    GenerateAuthorizationUrl(#[from] url::ParseError),
    #[error("oauth provider is not supported")]
    UnsupportedOauthProvider,
    #[error("missing id token")]
    MissingIDToken,
    #[error("failed to decode id token")]
    DecodeIdToken,
    #[error("failed to get user information")]
    GetUserInformation,
    #[error("upsert oauth account error: {0}")]
    UpsertOauthAccount(#[from] DBError),
}

impl From<OAuthError> for Status {
    fn from(err: OAuthError) -> Self {
        Status::new(Code::Internal, err.to_string())
    }
}
/// Error for `exchange_code`
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ExchangeCodeErr {
    #[error("failed to build request body")]
    BuildRequestBody(#[from] serde_urlencoded::ser::Error),
    #[error("failed to build http client")]
    BuildHttpClient,
    #[error("failed to send request")]
    SendRequest(#[from] reqwest::Error),
    #[error("failed to validate authorization code")]
    ValidateAuthorizationCode,
    #[error("missing id token")]
    MissingIDToken,
    #[error("failed to decode id token")]
    DecodeIdToken(#[from] jsonwebtoken::errors::Error),
    #[error("missing kid in token")]
    MissingKID,
    #[error("no matchin jwks found")]
    NoMatchingJWKS,
    #[error("missing access token")]
    MissingAccessToken,
    #[error("missing expires in")]
    MissingExpiresIn,
    #[error("missing x user id")]
    MissingXUserID,
    #[error("no email found")]
    NoEmailFound,
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
