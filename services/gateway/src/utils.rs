use axum::http::{HeaderMap, StatusCode, header::COOKIE};
use shared::cookie::extract_cookie_by_name;
use tonic::Code;

use crate::error::OAuthError;

pub(crate) const OAUTH_STATE: &'static str = "oauth_state";
pub(crate) const OAUTH_CODE_VERIFIER: &'static str = "oauth_code_verifier";

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

pub(crate) struct OauthCookieJar<'a>(&'a HeaderMap);

impl<'a> OauthCookieJar<'a> {
    pub(crate) fn from_headers(headers: &'a HeaderMap) -> Result<Self, OAuthError> {
        if headers.get(COOKIE).is_none() {
            return Err(OAuthError::MissingCookie("missing cookie header"));
        }
        Ok(Self(headers))
    }

    pub(crate) fn extract(&self, name: &'static str) -> Result<String, OAuthError> {
        let cookies = self.0.get(COOKIE).unwrap();
        extract_cookie_by_name(name, cookies).ok_or(OAuthError::MissingCookie(name))
    }
}
