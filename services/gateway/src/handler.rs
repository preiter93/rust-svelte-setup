use auth::proto::CreateSessionReq;
use auth::proto::api_service_client::ApiServiceClient as AuthServiceClient;
use axum::response::{IntoResponse, Response};
use axum::{Json, extract::State, http::StatusCode};
use axum_macros::debug_handler;
use serde_json::json;
use thiserror::Error;
use tonic::{Request, Status};

use crate::utils::grpc_to_http_status;

#[derive(Clone)]
pub(crate) struct Handler {
    auth_client: AuthServiceClient<tonic::transport::Channel>,
}

impl Handler {
    pub(crate) async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let auth_client = AuthServiceClient::connect("http://auth:50051").await?;
        Ok(Self { auth_client })
    }
}

#[debug_handler]
pub async fn create_session(State(mut h): State<Handler>) -> Result<Json<String>, GatewayError> {
    let req = Request::new(CreateSessionReq {});
    let resp = h.auth_client.create_session(req).await?;

    let resp_json = serde_json::to_string(&resp.into_inner())?;
    Ok(Json(resp_json))
}

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("gRPC request failed: {0}")]
    RequestError(#[from] Status),
    #[error("failed to serialize response: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
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
