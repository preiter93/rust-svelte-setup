use crate::error::CookieError;
use axum::http::StatusCode;
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use time::Duration;
use tonic::Code;

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
pub fn create_oauth_cookie<S, T>(name: S, value: T) -> Cookie<'static>
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
