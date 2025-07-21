use axum::http::StatusCode;
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use thiserror::Error;
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

pub(crate) fn secure_cookie(name: String, value: String) -> Cookie<'static> {
    Cookie::build((name, value))
        .http_only(true) // FOR TESTING
        .secure(false) // TODO: secure on PROD
        .max_age(time::Duration::seconds(60 * 10))
        .path("/")
        .same_site(SameSite::Lax)
        .build()
}

pub(crate) fn get_cookie_value(jar: &CookieJar, name: &str) -> Result<String, CookieError> {
    jar.get(name)
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| CookieError::Missing(name.to_string()))
}

#[derive(Debug, Error)]
pub(crate) enum CookieError {
    #[error("missing cookie: {0}")]
    Missing(String),
}
