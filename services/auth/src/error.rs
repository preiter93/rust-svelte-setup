use thiserror::Error;
use tonic::{Code, Status};

/// Error for [`crate::proto::api_service_server::ApiService::create_session`]
#[derive(Debug, Error)]
pub enum CreateSessionErr {
    #[error("missing user id")]
    MissingUserUID,

    #[error("insert session error: {0}")]
    InsertSession(#[from] DBError),
}

impl From<CreateSessionErr> for Status {
    fn from(err: CreateSessionErr) -> Self {
        let code = match err {
            CreateSessionErr::MissingUserUID => Code::InvalidArgument,
            CreateSessionErr::InsertSession(_) => Code::Internal,
        };
        Status::new(code, err.to_string())
    }
}

/// Error for [`crate::proto::api_service_server::ApiService::validate_session`]
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ValidateSessionErr {
    #[error("missing token")]
    MissingToken,

    #[error("invalid token format")]
    InvalidFormat,

    #[error("token secret mismatch")]
    SecretMismatch,

    #[error("token expired")]
    Expired,

    #[error("token not found for session {0}")]
    NotFound(String),

    #[error("get session error: {0}")]
    GetSession(DBError),

    #[error("delete session error: {0}")]
    DeleteSession(DBError),
}

impl From<ValidateSessionErr> for Status {
    fn from(err: ValidateSessionErr) -> Self {
        let code = match err {
            ValidateSessionErr::InvalidFormat | ValidateSessionErr::MissingToken => {
                Code::InvalidArgument
            }
            ValidateSessionErr::SecretMismatch
            | ValidateSessionErr::Expired
            | ValidateSessionErr::NotFound(_) => Code::Unauthenticated,
            ValidateSessionErr::GetSession(_) | ValidateSessionErr::DeleteSession(_) => {
                Code::Internal
            }
        };
        Status::new(code, err.to_string())
    }
}

/// Error for [`crate::proto::api_service_server::ApiService::delete_session`]
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DeleteSessionErr {
    #[error("missing token")]
    MissingToken,

    #[error("invalid token format")]
    InvalidFormat,

    #[error("delete session error: {0}")]
    DeleteSession(#[from] DBError),
}

impl From<DeleteSessionErr> for Status {
    fn from(err: DeleteSessionErr) -> Self {
        let code = match err {
            DeleteSessionErr::MissingToken | DeleteSessionErr::InvalidFormat => {
                Code::InvalidArgument
            }
            DeleteSessionErr::DeleteSession(_) => Code::Internal,
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
        Status::new(Code::Internal, err.to_string())
    }
}

/// Error for [`crate::proto::api_service_server::ApiService::handle_google_callback`]
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum HandleGoogleCallbackErr {
    #[error("failed to validate authorization code")]
    ValidateAuthorizationCode,

    #[error("missing id token")]
    MissingIDToken,

    #[error("failed to decode id token")]
    DecodeIdToken,
}

impl From<HandleGoogleCallbackErr> for Status {
    fn from(err: HandleGoogleCallbackErr) -> Self {
        Status::new(Code::Internal, err.to_string())
    }
}

/// Error for [`crate::proto::api_service_server::ApiService::start_github_login`]
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StartGithubLoginErr {
    #[error("failed to generate authorization url")]
    AuthorizationUrl,
}

impl From<StartGithubLoginErr> for Status {
    fn from(err: StartGithubLoginErr) -> Self {
        Status::new(Code::Internal, err.to_string())
    }
}

/// Error for [`crate::proto::api_service_server::ApiService::handle_github_callback`]
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum HandleGithubCallbackErr {
    #[error("failed to validate authorization code")]
    ValidateAuthorizationCode,

    #[error("missing access token")]
    MissingAccessToken,

    #[error("failed to get user information")]
    GetUserInformation,

    #[error("failed to get email information")]
    GetEmailInformation,

    #[error("failed to decode id token")]
    DecodeIdToken,
}

impl From<HandleGithubCallbackErr> for Status {
    fn from(err: HandleGithubCallbackErr) -> Self {
        Status::new(Code::Internal, err.to_string())
    }
}

/// Error for `start_{provider}_login`
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StartOauthLoginErr {
    #[error("failed to generate authorization url")]
    GenerateAuthorizationUrl(#[from] url::ParseError),

    #[error("oauth provider is not supported")]
    UnsupportedOauthProvider,
}

impl From<StartOauthLoginErr> for Status {
    fn from(err: StartOauthLoginErr) -> Self {
        Status::new(Code::Internal, err.to_string())
    }
}

/// Error for `handle_{provider}_callback`
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum HandleOauthCallbackErr {
    #[error("failed to exchange code: {0}")]
    ExchangeCode(#[from] ExchangeCodeErr),

    #[error("oauth provider is not supported")]
    UnsupportedOauthProvider,

    #[error("upsert oauth account error: {0}")]
    UpsertOauthAccount(#[from] DBError),
}

impl From<HandleOauthCallbackErr> for Status {
    fn from(err: HandleOauthCallbackErr) -> Self {
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

    #[error("no email found")]
    NoEmailFound,
}

/// Error for [`crate::proto::api_service_server::ApiService::link_oauth_account`]
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum LinkOauthAccountErr {
    #[error("missing oauth token id")]
    MissingOauthAccountID,

    #[error("missing user id")]
    MissingUserID,

    #[error("update oauth account error: {0}")]
    UpdateOauthAccount(#[from] DBError),
}

impl From<LinkOauthAccountErr> for Status {
    fn from(err: LinkOauthAccountErr) -> Self {
        let code = match err {
            LinkOauthAccountErr::MissingUserID => Code::InvalidArgument,
            LinkOauthAccountErr::MissingOauthAccountID => Code::InvalidArgument,
            _ => Code::Internal,
        };
        Status::new(code, err.to_string())
    }
}

/// Error for [`crate::proto::api_service_server::ApiService::get_oauth_account`]
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum GetOauthAccountErr {
    #[error("missing user id")]
    MissingUserID,

    #[error("get oauth account error: {0}")]
    GetOauthAccount(#[from] DBError),
}

impl From<GetOauthAccountErr> for Status {
    fn from(err: GetOauthAccountErr) -> Self {
        let code = match err {
            GetOauthAccountErr::MissingUserID => Code::InvalidArgument,
            _ => Code::Internal,
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

    #[error("connection pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),

    #[error("entity not found: {0}")]
    NotFound(String),

    #[error("conversion error: {0}")]
    Conversion(String),

    #[error("invalid input")]
    InvalidInput,
}
