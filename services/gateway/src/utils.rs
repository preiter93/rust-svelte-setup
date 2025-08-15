use crate::error::CookieError;
use axum::http::StatusCode;
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use time::Duration;
use tonic::Code;

pub const SESSION_TOKEN_KEY: &'static str = "session_token";

/// Maps grpc codes to http status codes.
///
/// # Documentation
/// <https://chromium.googlesource.com/external/github.com/grpc/grpc/+/refs/tags/v1.21.4-pre1/doc/statuscodes.md>
pub(crate) fn grpc_to_http_status(code: Code) -> StatusCode {
    match code {
        Code::Ok => StatusCode::OK,
        Code::Cancelled => StatusCode::REQUEST_TIMEOUT,
        Code::InvalidArgument | Code::FailedPrecondition | Code::OutOfRange => {
            StatusCode::BAD_REQUEST
        }
        Code::DeadlineExceeded => StatusCode::GATEWAY_TIMEOUT,
        Code::NotFound => StatusCode::NOT_FOUND,
        Code::AlreadyExists => StatusCode::CONFLICT,
        Code::PermissionDenied => StatusCode::FORBIDDEN,
        Code::Unauthenticated => StatusCode::UNAUTHORIZED,
        Code::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS,
        Code::Unimplemented => StatusCode::NOT_IMPLEMENTED,
        Code::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// Extracts a cookie value from the given jar by name.
/// Returns error if cookie is missing.
pub fn extract_cookie(jar: &CookieJar, name: &str) -> Result<String, CookieError> {
    jar.get(name)
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| CookieError::Missing(name.to_string()))
}

/// Creates a generic OAuth cookie with configurable name and value.
/// Sets common security attributes and a default 10-minute expiration.
pub fn build_oauth_cookie<S, T>(name: S, value: T) -> Cookie<'static>
where
    S: Into<String>,
    T: Into<String>,
{
    Cookie::build((name.into(), value.into()))
        .http_only(true)
        .secure(false) // TODO: Enable in production
        .max_age(Duration::seconds(60 * 10)) // 10 minutes
        .path("/")
        .same_site(SameSite::Lax)
        .build()
}

/// Creates a session token cookie with 7-day expiration.
pub fn build_session_token_cookie<T>(token: T) -> Cookie<'static>
where
    T: Into<String>,
{
    const SESSION_TOKEN_KEY: &str = "session_token";

    Cookie::build((SESSION_TOKEN_KEY, token.into()))
        .http_only(true)
        .secure(false) // TODO: Enable in production
        .max_age(Duration::seconds(60 * 60 * 24 * 7)) // 7 days
        .path("/")
        .same_site(SameSite::Lax)
        .build()
}
