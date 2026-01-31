/// OAuth errors
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
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

    #[error("unexpected HTTP status code: {0}")]
    UnexpectedStatusCode(reqwest::StatusCode),

    #[error("parse URL: {0}")]
    ParseURL(#[from] url::ParseError),
}
