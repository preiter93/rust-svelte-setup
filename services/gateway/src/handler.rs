use auth::AuthClient;
use auth::proto::{
    CreateSessionReq, CreateSessionResp, HandleGoogleCallbackReq, StartGoogleLoginReq,
    StartGoogleLoginResp, ValidateSessionReq,
};
use axum::extract::Query;
use axum::response::Redirect;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use axum_extra::extract::CookieJar;
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use axum_macros::debug_handler;
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;
use tonic::{Request, Status};
use tracing::instrument;
use user::{
    UserClient,
    proto::{
        CreateUserReq, CreateUserResp, GetUserIdFromGoogleIdReq, GetUserIdFromGoogleIdResp,
        GetUserReq, GetUserResp,
    },
};

use crate::utils::{CookieError, get_cookie_value, grpc_to_http_status, secure_cookie};

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

// ----------------------------------------
//         UNAUTHENTICATED ENDPOINTS
// ----------------------------------------

#[debug_handler]
#[instrument(skip(h), err)]
pub async fn create_session(
    State(mut h): State<Handler>,
    Json(payload): Json<CreateSessionReq>,
) -> Result<Json<CreateSessionResp>, GatewayError> {
    let req = Request::new(payload);
    let resp = h.auth_client.create_session(req).await?;

    Ok(Json(resp.into_inner()))
}

#[debug_handler]
#[instrument(skip(h), err)]
pub async fn create_user(
    State(mut h): State<Handler>,
    Json(payload): Json<CreateUserReq>,
) -> Result<Json<CreateUserResp>, GatewayError> {
    let req = Request::new(payload);
    let resp = h.user_client.create_user(req).await?;

    Ok(Json(resp.into_inner()))
}

#[debug_handler]
#[instrument(skip(h), err)]
pub async fn get_user_id_by_google_id(
    State(mut h): State<Handler>,
    Path(google_id): Path<String>,
) -> Result<Json<GetUserIdFromGoogleIdResp>, GatewayError> {
    let req = Request::new(GetUserIdFromGoogleIdReq { google_id });
    let resp = h.user_client.get_user_id_from_google_id(req).await?;

    Ok(Json(resp.into_inner()))
}

#[debug_handler]
#[instrument(skip(h), err)]
pub async fn start_google_login(
    State(mut h): State<Handler>,
    jar: CookieJar,
) -> Result<Response, GatewayError> {
    let req = Request::new(StartGoogleLoginReq {});
    let resp = h.auth_client.start_google_login(req).await?.into_inner();

    let state_cookie = secure_cookie("google_state".to_string(), resp.state);
    let code_cookie = secure_cookie("google_code_verifier".to_string(), resp.code_verifier);
    let jar = jar.add(state_cookie).add(code_cookie);

    let resp = Json(StartGoogleLoginResp {
        state: String::new(),
        code_verifier: String::new(),
        authorization_url: resp.authorization_url,
    });

    Ok((jar, resp).into_response())
}

#[derive(Deserialize)]
pub struct GoogleCallbackQuery {
    state: String,
    code: String,
}

#[debug_handler]
#[instrument(skip(h, query), err)]
pub async fn handle_google_callback(
    State(mut h): State<Handler>,
    Query(query): Query<GoogleCallbackQuery>,
    jar: CookieJar,
) -> Result<Redirect, OAuthError> {
    let stored_state =
        get_cookie_value(&jar, "google_state").map_err(OAuthError::ReadCookieError)?;
    let code_verifier =
        get_cookie_value(&jar, "google_code_verifier").map_err(OAuthError::ReadCookieError)?;

    if query.state != stored_state {
        return Err(OAuthError::StateMismatch);
    }

    let req = Request::new(HandleGoogleCallbackReq {
        state: query.state,
        code: query.code,
        code_verifier,
    });
    h.auth_client.handle_google_callback(req).await?;

    Ok(Redirect::to("/"))
}

// ----------------------------------------
//         AUTHENTICATED ENDPOINTS
// ----------------------------------------

#[debug_handler]
#[instrument(skip(h), err)]
pub async fn get_current_user(
    State(mut h): State<Handler>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<GetUserResp>, GatewayError> {
    let session_token = bearer.token().to_string();
    let validate_ression_req = Request::new(ValidateSessionReq {
        token: session_token,
    });

    let validate_ression_resp = h.auth_client.validate_session(validate_ression_req).await?;
    let user_id = validate_ression_resp.into_inner().user_id;

    let get_user_req = Request::new(GetUserReq { id: user_id });
    let get_user_resp = h.user_client.get_user(get_user_req).await?;

    Ok(Json(get_user_resp.into_inner()))
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

#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("gRPC request failed: {0}")]
    RequestError(#[from] Status),
    #[error("state mismatch in oauth flow")]
    StateMismatch,
    #[error("read cookie error: {0}")]
    ReadCookieError(#[from] CookieError),
}

impl IntoResponse for OAuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::RequestError(e) => (
                grpc_to_http_status(e.code()),
                Self::RequestError(e).to_string(),
            ),
            Self::StateMismatch => (StatusCode::UNAUTHORIZED, Self::StateMismatch.to_string()),
            Self::ReadCookieError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Self::ReadCookieError(e).to_string(),
            ),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}
