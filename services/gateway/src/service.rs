use crate::error::OAuthError;
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
