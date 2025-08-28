use crate::error::ApiError;
use crate::error::OAuthError;
use crate::service::create_user_if_not_found;
use crate::utils::OAUTH_CODE_VERIFIER;
use crate::utils::OAUTH_STATE;
use crate::utils::OauthCookieJar;
use auth::AuthClient;
use auth::proto::OauthProvider;
use auth::proto::{CreateSessionReq, DeleteSessionReq, HandleOauthCallbackReq, StartOauthLoginReq};
use axum::Extension;
use axum::body::Body;
use axum::extract::Path;
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
use shared::cookie::expire_session_token_cookie;
use shared::cookie::extract_session_token_cookie;
use shared::session::SessionState;
use tonic::{Code, Request};
use tracing::instrument;
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
        .with_cookie(expire_session_token_cookie())
        .body(Body::empty())?;

    Ok(response)
}

/// Initiates the OAuth login flow. Does not require authentication.
#[debug_handler]
#[instrument(skip(h), err)]
pub async fn start_oauth_login(
    Path(provider): Path<String>,
    State(mut h): State<Handler>,
) -> Result<Response, ApiError> {
    let provider = match provider.as_ref() {
        "google" => OauthProvider::Google,
        "github" => OauthProvider::Github,
        _ => OauthProvider::Unspecified,
    };
    let req = Request::new(StartOauthLoginReq {
        provider: provider.into(),
    });
    let resp = h.auth_client.start_oauth_login(req).await?.into_inner();

    let response = Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header(LOCATION, &resp.authorization_url)
        .with_cookies([
            create_oauth_cookie(OAUTH_STATE, resp.state),
            create_oauth_cookie(OAUTH_CODE_VERIFIER, resp.code_verifier),
        ])
        .body(Body::empty())?;

    Ok(response)
}

#[derive(Deserialize)]
pub struct OauthCallbackQuery {
    state: String,
    code: String,
}

/// Handles the OAuth callback, creates a session and logs the user in.
/// Does not require authentication.
#[debug_handler]
#[instrument(skip(h, query), err)]
pub async fn handle_oauth_callback(
    Path(provider): Path<String>,
    State(mut h): State<Handler>,
    Query(query): Query<OauthCallbackQuery>,
    headers: HeaderMap,
) -> Result<Response, OAuthError> {
    let provider = match provider.as_ref() {
        "google" => OauthProvider::Google,
        "github" => OauthProvider::Github,
        _ => OauthProvider::Unspecified,
    };

    let jar = OauthCookieJar::from_headers(&headers)?;
    let stored_state = jar.extract(OAUTH_STATE)?;
    let code_verifier = jar.extract(OAUTH_CODE_VERIFIER)?;

    if query.state != stored_state {
        return Err(OAuthError::StateMismatch);
    }

    let callback_req = Request::new(HandleOauthCallbackReq {
        provider: provider.into(),
        code: query.code,
        code_verifier: code_verifier.clone(),
    });
    let callback_resp = h.auth_client.handle_oauth_callback(callback_req).await?;
    let callback_data = callback_resp.into_inner();

    let oauth_id = callback_data.id.to_string();
    let name = callback_data.name.to_string();
    let email = callback_data.email.to_string();

    let user_req = Request::new(GetUserIdFromOauthIdReq {
        oauth_id: oauth_id.clone(),
        provider: provider.into(),
    });
    let user_resp = h.user_client.get_user_id_from_oauth_id(user_req).await;
    let user_id = match user_resp {
        Ok(resp) => resp.into_inner().id,
        Err(ref status) if status.code() == Code::NotFound => {
            // TODO: Do not safe oauth id on user table
            create_user_if_not_found(&mut h.user_client, oauth_id, String::new(), name, email)
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
            create_expired_cookie(OAUTH_STATE),
            create_expired_cookie(OAUTH_CODE_VERIFIER),
        ])
        .body(Body::empty())?;

    Ok(response)
}
