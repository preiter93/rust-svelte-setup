use tonic::{Code, Status};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("oauth error: {0}")]
    Error(#[from] oauth::Error),

    #[error("missing id token")]
    MissingIDToken,

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

    #[error("missing email")]
    MissingEmail,

    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("unexpected HTTP status code: {0}")]
    UnexpectedStatusCode(reqwest::StatusCode),
}

impl From<Error> for Status {
    fn from(err: Error) -> Self {
        Status::new(Code::Internal, err.to_string())
    }
}
