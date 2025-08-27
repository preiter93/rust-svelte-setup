use crate::error::ApiError;
use crate::error::OAuthError;
use crate::service::create_user_if_not_found;
use crate::utils::GITHUB_STATE;
use crate::utils::GOOGLE_CODE_VERIFIER;
use crate::utils::GOOGLE_STATE;
use crate::utils::OauthCookieJar;
use auth::AuthClient;
use auth::proto::HandleGithubCallbackReq;
use auth::proto::StartGithubLoginReq;
use auth::proto::{
    CreateSessionReq, DeleteSessionReq, HandleGoogleCallbackReq, StartGoogleLoginReq,
};
use axum::Extension;
use axum::body::Body;
use axum::extract::Query;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::http::header::LOCATION;
use axum::{Json, extract::State, response::Response};
use axum_macros::debug_handler;
use serde::Deserialize;
use shared::cookie::ResponseCookies;
use shared::cookie::create_expired_cookie;
use shared::cookie::create_oauth_cookie;
use shared::cookie::create_session_token_cookie;
use shared::cookie::extract_session_token_cookie;
use shared::session::SessionState;
use tonic::{Code, Request};
use tracing::instrument;
use user::proto::OauthProvider;
use user::{
    UserClient,
    proto::{GetUserIdFromOauthIdReq, GetUserReq, GetUserResp},
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
    Extension(SessionState { user_id }): Extension<SessionState>,
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
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let Some(cookie_header) = headers.get("cookie") else {
        return Err(ApiError::Unauthenticated);
    };
    let Some(token) = extract_session_token_cookie(cookie_header) else {
        return Err(ApiError::Unauthenticated);
    };

    let req = Request::new(DeleteSessionReq {
        token: token.clone(),
    });
    h.auth_client.delete_session(req).await?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .with_cookie(create_session_token_cookie(token))
        .body(Body::empty())?;

    Ok(response)
}

// ----------------------------------------
//           OAUTH ENDPOINTS
// ----------------------------------------

/// Initiates the Google OAuth login flow. Does not require authentication.
#[debug_handler]
#[instrument(skip(h), err)]
pub async fn start_google_login(State(mut h): State<Handler>) -> Result<Response, ApiError> {
    let req = Request::new(StartGoogleLoginReq {});
    let resp = h.auth_client.start_google_login(req).await?.into_inner();

    let response = Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header(LOCATION, &resp.authorization_url)
        .with_cookies([
            create_oauth_cookie(GOOGLE_STATE, resp.state),
            create_oauth_cookie(GOOGLE_CODE_VERIFIER, resp.code_verifier),
        ])
        .body(Body::empty())?;

    Ok(response)
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
    headers: HeaderMap,
) -> Result<Response, OAuthError> {
    let jar = OauthCookieJar::from_headers(&headers)?;
    let stored_state = jar.extract(GOOGLE_STATE)?;
    let code_verifier = jar.extract(GOOGLE_CODE_VERIFIER)?;

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

    let user_req = Request::new(GetUserIdFromOauthIdReq {
        oauth_id: google_id.clone(),
        provider: OauthProvider::Google.into(),
    });
    let user_resp = h.user_client.get_user_id_from_oauth_id(user_req).await;
    let user_id = match user_resp {
        Ok(resp) => resp.into_inner().id,
        Err(ref status) if status.code() == Code::NotFound => {
            create_user_if_not_found(&mut h.user_client, google_id, String::new(), name, email)
                .await?
        }
        Err(err) => return Err(OAuthError::RequestError(err)),
    };

    let session_req = Request::new(CreateSessionReq { user_id });
    let session_resp = h.auth_client.create_session(session_req).await?;
    let session_token = session_resp.into_inner().token;

    let response = Response::builder()
        .status(StatusCode::OK)
        .with_cookies([
            create_session_token_cookie(session_token),
            create_expired_cookie(GOOGLE_STATE),
            create_expired_cookie(GOOGLE_CODE_VERIFIER),
        ])
        .body(Body::empty())?;

    Ok(response)
}

/// Initiates the Github OAuth login flow. Does not require authentication.
#[debug_handler]
#[instrument(skip(h), err)]
pub async fn start_github_login(State(mut h): State<Handler>) -> Result<Response, ApiError> {
    let req = Request::new(StartGithubLoginReq {});
    let resp = h.auth_client.start_github_login(req).await?.into_inner();

    let response = Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header(LOCATION, &resp.authorization_url)
        .with_cookies([create_oauth_cookie(GITHUB_STATE, resp.state)])
        .body(Body::empty())?;

    Ok(response)
}

#[derive(Deserialize)]
pub struct GithubCallbackQuery {
    state: String,
    code: String,
}

/// Handles the Github OAuth callback, creates a session and logs the user in.
/// Does not require authentication.
#[debug_handler]
#[instrument(skip(h, query), err)]
pub async fn handle_github_callback(
    State(mut h): State<Handler>,
    Query(query): Query<GithubCallbackQuery>,
    headers: HeaderMap,
) -> Result<Response, OAuthError> {
    let jar = OauthCookieJar::from_headers(&headers)?;
    let stored_state = jar.extract(GITHUB_STATE)?;

    if query.state != stored_state {
        return Err(OAuthError::StateMismatch);
    }

    let callback_req = Request::new(HandleGithubCallbackReq {
        state: query.state,
        code: query.code,
    });
    let callback_resp = h.auth_client.handle_github_callback(callback_req).await?;
    let callback_data = callback_resp.into_inner();

    let github_id = callback_data.github_id.to_string();
    let name = callback_data.name.to_string();
    let email = callback_data.email.to_string();

    let user_req = Request::new(GetUserIdFromOauthIdReq {
        oauth_id: github_id.clone(),
        provider: OauthProvider::Github.into(),
    });
    let user_resp = h.user_client.get_user_id_from_oauth_id(user_req).await;
    let user_id = match user_resp {
        Ok(resp) => resp.into_inner().id,
        Err(ref status) if status.code() == Code::NotFound => {
            create_user_if_not_found(&mut h.user_client, String::new(), github_id, name, email)
                .await?
        }
        Err(err) => return Err(OAuthError::RequestError(err)),
    };

    let session_req = Request::new(CreateSessionReq { user_id });
    let session_resp = h.auth_client.create_session(session_req).await?;
    let session_token = session_resp.into_inner().token;

    let response = Response::builder()
        .status(StatusCode::OK)
        .with_cookies([
            create_session_token_cookie(session_token),
            create_expired_cookie(GITHUB_STATE),
        ])
        .body(Body::empty())?;

    Ok(response)
}
