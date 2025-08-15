use crate::error::ApiError;
use auth::AuthClient;
use auth::proto::{
    CreateSessionReq, DeleteSessionReq, HandleGoogleCallbackReq, StartGoogleLoginReq,
};
use axum::Extension;
use axum::extract::Query;
use axum::response::Redirect;
use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use shared::id::UserId;

use crate::error::OAuthError;
use crate::service::{create_user_if_not_found, get_session_token_from_cookie};
use crate::utils::{build_oauth_cookie, build_session_token_cookie, extract_cookie};
use axum_extra::extract::CookieJar;
use axum_macros::debug_handler;
use serde::Deserialize;
use tonic::{Code, Request};
use tracing::instrument;
use user::{
    UserClient,
    proto::{GetUserIdFromGoogleIdReq, GetUserReq, GetUserResp},
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
//         USER ENDPOINTS
// ----------------------------------------

/// Gets the current authenticated user.
#[debug_handler]
#[instrument(skip(h), err)]
pub async fn get_current_user(
    State(mut h): State<Handler>,
    Extension(UserId(user_id)): Extension<UserId>,
) -> Result<Json<GetUserResp>, ApiError> {
    let req = Request::new(GetUserReq { id: user_id });
    let resp = h.user_client.get_user(req).await?;
    Ok(Json(resp.into_inner()))
}

/// Logs the current authenticated user out.
#[debug_handler]
#[instrument(skip(h), err)]
pub async fn logout_user(
    State(mut h): State<Handler>,
    jar: CookieJar,
) -> Result<Response, ApiError> {
    let token = get_session_token_from_cookie(&jar)?;
    let req = Request::new(DeleteSessionReq {
        token: token.clone(),
    });
    h.auth_client.delete_session(req).await?;

    let jar = jar.remove(build_session_token_cookie(token));

    Ok(jar.into_response())
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
) -> Result<Response, ApiError> {
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
