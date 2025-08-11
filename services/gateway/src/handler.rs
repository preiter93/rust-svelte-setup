use auth::AuthClient;
use auth::proto::{
    CreateSessionReq, CreateSessionResp, DeleteSessionReq, HandleGoogleCallbackReq,
    StartGoogleLoginReq,
};
use axum::extract::Query;
use axum::response::Redirect;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::service::{
    create_user_if_not_found, get_session_token_from_cookie, validate_session_from_cookie,
};
use crate::utils::{
    CookieError, build_oauth_cookie, build_session_token_cookie, extract_cookie,
    grpc_to_http_status,
};
use axum_extra::extract::CookieJar;
use axum_macros::debug_handler;
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;
use tonic::{Code, Request, Status};
use tracing::instrument;
use user::{
    UserClient,
    proto::{
        CreateUserReq, CreateUserResp, GetUserIdFromGoogleIdReq, GetUserIdFromGoogleIdResp,
        GetUserReq, GetUserResp,
    },
};

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
//         SESSION ENDPOINTS
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
pub async fn delete_session(
    State(mut h): State<Handler>,
    jar: CookieJar,
) -> Result<Response, GatewayError> {
    let token = get_session_token_from_cookie(&jar)?;
    let req = Request::new(DeleteSessionReq {
        token: token.clone(),
    });
    h.auth_client.delete_session(req).await?;

    let jar = jar.remove(build_session_token_cookie(token));

    Ok(jar.into_response())
}

// ----------------------------------------
//         USER ENDPOINTS
// ----------------------------------------

/// Creates a new user.
/// Does not require authentication.
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

/// Retrieves a user ID by Google ID.
/// Does not require authentication.
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

/// Gets the current authenticated user.
#[debug_handler]
#[instrument(skip(h), err)]
pub async fn get_current_user(
    State(mut h): State<Handler>,
    jar: CookieJar,
) -> Result<Json<GetUserResp>, GatewayError> {
    let user_id = validate_session_from_cookie(&mut h.auth_client, &jar).await?;

    let get_user_req = Request::new(GetUserReq { id: user_id });
    let get_user_resp = h.user_client.get_user(get_user_req).await?;

    Ok(Json(get_user_resp.into_inner()))
}

// ----------------------------------------
//           OAUTH ENDPOINTS
// ----------------------------------------

/// Initiates the Google OAuth login flow
/// Does not require authentication.
#[debug_handler]
#[instrument(skip(h), err)]
pub async fn start_google_login(
    State(mut h): State<Handler>,
    jar: CookieJar,
) -> Result<Response, GatewayError> {
    let resp = h
        .auth_client
        .start_google_login(Request::new(StartGoogleLoginReq {}))
        .await?
        .into_inner();

    let jar = jar
        .add(build_oauth_cookie("google_state", resp.state))
        .add(build_oauth_cookie(
            "google_code_verifier",
            resp.code_verifier,
        ));

    let redirect = Redirect::temporary(&resp.authorization_url);

    Ok((jar, redirect).into_response())
}

#[derive(Deserialize)]
pub struct GoogleCallbackQuery {
    state: String,
    code: String,
}

/// Handles the Google OAuth callback, creates a session and logs the user in.
/// Does not require authentication.
#[debug_handler]
#[instrument(skip(h, query), err)]
pub async fn handle_google_callback(
    State(mut h): State<Handler>,
    Query(query): Query<GoogleCallbackQuery>,
    jar: CookieJar,
) -> Result<Response, OAuthError> {
    let stored_state = extract_cookie(&jar, "google_state")?;
    let code_verifier = extract_cookie(&jar, "google_code_verifier")?;

    if query.state != stored_state {
        return Err(OAuthError::StateMismatch);
    }

    let callback_req = Request::new(HandleGoogleCallbackReq {
        state: query.state,
        code: query.code,
        code_verifier: code_verifier.clone(),
    });
    let callback_resp = h.auth_client.handle_google_callback(callback_req).await?;
    let callback_data = callback_resp.into_inner();

    let google_id = callback_data.google_id.to_string();
    let name = callback_data.name.to_string();
    let email = callback_data.email.to_string();

    let user_req = Request::new(GetUserIdFromGoogleIdReq {
        google_id: google_id.clone(),
    });
    let user_resp = h.user_client.get_user_id_from_google_id(user_req).await;
    let user_id = match user_resp {
        Ok(resp) => resp.into_inner().id,
        Err(ref status) if status.code() == Code::NotFound => {
            create_user_if_not_found(&mut h.user_client, google_id, name, email).await?
        }
        Err(err) => return Err(OAuthError::RequestError(err)),
    };

    let session_req = Request::new(CreateSessionReq { user_id });
    let session_resp = h.auth_client.create_session(session_req).await?;
    let session_token = session_resp.into_inner().token;

    let jar = jar.add(build_session_token_cookie(session_token));

    let jar = jar
        .remove(build_oauth_cookie("google_state", stored_state))
        .remove(build_oauth_cookie("google_code_verifier", code_verifier));

    Ok(jar.into_response())
}

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("unauthenticated:")]
    Unauthenticated,
    #[error("gRPC request failed: {0}")]
    RequestError(#[from] Status),
    #[error("failed to serialize response: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl IntoResponse for GatewayError {
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

#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("gRPC request failed: {0}")]
    RequestError(#[from] Status),
    #[error("state mismatch in oauth flow")]
    StateMismatch,
    #[error("internal error: {0}")]
    InternalError(String),
}

impl IntoResponse for OAuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::RequestError(e) => (
                grpc_to_http_status(e.code()),
                Self::RequestError(e).to_string(),
            ),
            Self::StateMismatch => (StatusCode::UNAUTHORIZED, Self::StateMismatch.to_string()),
            Self::InternalError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Self::InternalError(e).to_string(),
            ),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

impl From<CookieError> for OAuthError {
    fn from(value: CookieError) -> Self {
        Self::InternalError(value.to_string())
    }
}
