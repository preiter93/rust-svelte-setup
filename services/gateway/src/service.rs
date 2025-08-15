use crate::{
    error::{ApiError, OAuthError},
    utils::SESSION_TOKEN_KEY,
};
use auth::{AuthClient, proto::ValidateSessionReq};
use axum_extra::extract::CookieJar;
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

/// Validates the sessoin token from the cookie.
///
/// Returns unauthenticated if
/// - there is no session token in the cookie jar
/// - the session token isn not valid
pub(crate) async fn validate_session_from_cookie(
    auth_client: &mut AuthClient,
    jar: &CookieJar,
) -> Result<String, ApiError> {
    let session_token = jar
        .get(SESSION_TOKEN_KEY)
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| ApiError::Unauthenticated)?;

    let validate_req = Request::new(ValidateSessionReq {
        token: session_token,
    });

    let validate_resp = auth_client
        .validate_session(validate_req)
        .await
        .map_err(|_| ApiError::Unauthenticated)?;

    Ok(validate_resp.into_inner().user_id)
}

/// Returns the session token from the cookie.
pub(crate) fn get_session_token_from_cookie(jar: &CookieJar) -> Result<String, ApiError> {
    jar.get(SESSION_TOKEN_KEY)
        .map(|cookie| cookie.value().to_string())
        .ok_or(ApiError::Unauthenticated)
}
