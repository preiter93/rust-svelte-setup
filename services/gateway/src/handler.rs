use auth::AuthClient;
use auth::proto::{CreateSessionReq, CreateSessionResp};
use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use axum::{Json, extract::State, http::StatusCode};
use axum_macros::debug_handler;
use serde_json::json;
use thiserror::Error;
use tonic::{Request, Status};
use tracing::instrument;
use user::UserClient;
use user::proto::{
    CreateUserReq, CreateUserResp, GetUserReq, GetUserResp, ListUsersReq, ListUsersResp,
    get_user_req,
};

use crate::utils::grpc_to_http_status;

#[derive(Clone)]
pub(crate) struct Handler {
    auth_client: AuthClient,
    user_client: UserClient,
}

impl Handler {
    pub(crate) async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let auth_client = AuthClient::new().await?;
        let user_client = UserClient::new().await?;
        Ok(Self {
            auth_client,
            user_client,
        })
    }
}

#[debug_handler]
#[instrument(skip(h))]
pub async fn create_session(
    State(mut h): State<Handler>,
    Json(payload): Json<CreateSessionReq>,
) -> Result<Json<CreateSessionResp>, GatewayError> {
    let req = Request::new(payload);
    let resp = h.auth_client.create_session(req).await?;

    Ok(Json(resp.into_inner()))
}

// #[debug_handler]
// #[instrument(skip(h))]
// pub async fn validate_session(
//     State(mut h): State<Handler>,
//     Json(payload): Json<ValidateSessionReq>,
// ) -> Result<Json<String>, GatewayError> {
//     let req = Request::new(payload);
//     let resp = h.auth_client.validate_session(req).await?;
//
//     let resp_json = serde_json::to_string(&resp.into_inner())?;
//     Ok(Json(resp_json))
// }

#[debug_handler]
#[instrument(skip(h))]
pub async fn list_users(State(mut h): State<Handler>) -> Result<Json<ListUsersResp>, GatewayError> {
    let req = Request::new(ListUsersReq {});
    let resp = h.user_client.list_users(req).await?;

    Ok(Json(resp.into_inner()))
}

#[debug_handler]
#[instrument(skip(h))]
pub async fn create_user(
    State(mut h): State<Handler>,
    Json(payload): Json<CreateUserReq>,
) -> Result<Json<CreateUserResp>, GatewayError> {
    let req = Request::new(payload);
    let resp = h.user_client.create_user(req).await?;

    Ok(Json(resp.into_inner()))
}

#[debug_handler]
#[instrument(skip(h))]
pub async fn get_user(
    State(mut h): State<Handler>,
    Path(id): Path<String>,
) -> Result<Json<GetUserResp>, GatewayError> {
    let identifier = get_user_req::Identifier::Id(id);

    let req = Request::new(GetUserReq {
        identifier: Some(identifier),
    });
    let resp = h.user_client.get_user(req).await?;

    Ok(Json(resp.into_inner()))
}
#[debug_handler]
#[instrument(skip(h))]
pub async fn get_user_by_google_id(
    State(mut h): State<Handler>,
    Path(google_id): Path<String>,
) -> Result<Json<GetUserResp>, GatewayError> {
    let identifier = get_user_req::Identifier::GoogleId(google_id);

    let req = Request::new(GetUserReq {
        identifier: Some(identifier),
    });
    let resp = h.user_client.get_user(req).await?;

    Ok(Json(resp.into_inner()))
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
