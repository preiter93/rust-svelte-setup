use axum::response::{IntoResponse, Response};
use axum::{Json, http::StatusCode};
use serde_json::json;
use thiserror::Error;
use tonic::Status;

use crate::utils::grpc_to_http_status;

/// Error for api endpoints.
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("unauthenticated")]
    Unauthenticated,
    #[error("gRPC request failed: {0}")]
    RequestError(#[from] Status),
    #[error("failed to serialize response: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::Unauthenticated => (StatusCode::UNAUTHORIZED, "unauthenticated".to_string()),
            Self::RequestError(e) => (
                grpc_to_http_status(e.code()),
                Self::RequestError(e).to_string(),
            ),
            Self::SerializationError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Self::SerializationError(e).to_string(),
            ),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

/// Error for oauth endpoints
#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("gRPC request failed: {0}")]
    RequestError(#[from] Status),
    #[error("state mismatch in oauth flow")]
    StateMismatch,
    #[error("cookie error: {0}")]
    CookieError(#[from] CookieError),
}

impl IntoResponse for OAuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::RequestError(e) => (
                grpc_to_http_status(e.code()),
                Self::RequestError(e).to_string(),
            ),
            Self::StateMismatch => (StatusCode::UNAUTHORIZED, Self::StateMismatch.to_string()),
            Self::CookieError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Self::CookieError(e).to_string(),
            ),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

/// Error for cookies
#[derive(Debug, Error)]
pub enum CookieError {
    #[error("missing cookie: {0}")]
    Missing(String),
}
