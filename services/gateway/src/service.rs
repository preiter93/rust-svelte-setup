use crate::error::{ApiError, OAuthError};
use axum_extra::extract::CookieJar;
use shared::session::SESSION_TOKEN_COOKIE_KEY;
use tonic::{Code, Request, Status};
use user::{UserClient, proto::CreateUserReq};

/// Creates a user if it does not exist yet.
pub(crate) async fn create_user_if_not_found(
    user_client: &mut UserClient,
    google_id: String,
    name: String,
    email: String,
) -> Result<String, OAuthError> {
    let req = Request::new(CreateUserReq {
        google_id,
        name,
        email,
    });
    let resp = user_client.create_user(req).await?;
    let user = resp.into_inner().user.ok_or_else(|| {
        let not_found_err = Status::new(Code::NotFound, "no user found");
        OAuthError::RequestError(not_found_err)
    })?;
    Ok(user.id)
}

/// Returns the session token from the cookie.
pub(crate) fn get_session_token_from_cookie(jar: &CookieJar) -> Result<String, ApiError> {
    jar.get(SESSION_TOKEN_COOKIE_KEY)
        .map(|cookie| cookie.value().to_string())
        .ok_or(ApiError::Unauthenticated)
}
