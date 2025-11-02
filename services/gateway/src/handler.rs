use crate::error::{ApiError, OAuthError};
use crate::utils::{OAUTH_CODE_VERIFIER, OAUTH_STATE, OauthCookieJar, parse_provider};
use auth::proto::{
    CreateSessionReq, DeleteSessionReq, HandleOauthCallbackReq, LinkOauthAccountReq,
    StartOauthLoginReq,
};
use auth::{AuthClient, IAuthClient};
use axum::{
    Extension, Json,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header::LOCATION},
    response::Response,
};
use axum_macros::debug_handler;
use serde::Deserialize;
use setup::cookie::{
    ResponseCookies, create_expired_oauth_cookie, create_oauth_cookie, create_session_token_cookie,
    expire_session_token_cookie, extract_session_token_cookie,
};
use setup::session::SessionState;
use tonic::{Code, Request, Status};
use tracing::instrument;
use user::IUserClient;
use user::proto::CreateUserReq;
use user::{
    UserClient,
    proto::{GetUserReq, GetUserResp},
};

#[derive(Clone)]
pub(crate) struct Handler {
    auth_client: AuthClient,
    user_client: UserClient,
}

impl Handler {
    pub(crate) async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let auth_client = AuthClient::new()
            .await
            .map_err(|e| format!("AuthClient initialization failed: {}", e))?;

        let user_client = UserClient::new()
            .await
            .map_err(|e| format!("UserClient initialization failed: {}", e))?;

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
    State(h): State<Handler>,
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
    State(h): State<Handler>,
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
    State(h): State<Handler>,
) -> Result<Response, ApiError> {
    let provider = parse_provider(provider);
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
    State(h): State<Handler>,
    Query(query): Query<OauthCallbackQuery>,
    headers: HeaderMap,
) -> Result<Response, OAuthError> {
    let provider = parse_provider(provider);

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

    let account_id = callback_data.account_id;
    let name = callback_data.provider_user_name;
    let email = callback_data.provider_user_email;

    let mut user_id = callback_data.user_id;
    if user_id.is_empty() {
        let req = Request::new(CreateUserReq { name, email });
        let resp = h.user_client.create_user(req).await?;
        let user = resp.into_inner().user.ok_or_else(|| {
            OAuthError::RequestError(Status::new(Code::Internal, "failed to create user"))
        })?;
        user_id = user.id;

        let req = Request::new(LinkOauthAccountReq {
            account_id,
            user_id: user_id.clone(),
        });
        let _ = h.auth_client.link_oauth_account(req).await?;
    }

    let session_req = Request::new(CreateSessionReq { user_id });
    let session_resp = h.auth_client.create_session(session_req).await?;
    let session_token = session_resp.into_inner().token;

    let response = Response::builder()
        .status(StatusCode::OK)
        .with_cookies([
            create_session_token_cookie(session_token),
            create_expired_oauth_cookie(OAUTH_STATE),
            create_expired_oauth_cookie(OAUTH_CODE_VERIFIER),
        ])
        .body(Body::empty())?;

    Ok(response)
}
