pub mod client;
pub mod proto;

use crate::client::{AuthClient, IAuthClient};
use crate::proto::ValidateSessionReq;
use setup::middleware::SessionAuthClient;
use setup::{
    middleware::auth::{AuthenticateSessionErr, AuthenticatedSession},
    session::SessionState,
};
use tonic::async_trait;
use tonic::{Code, Request};

pub const GRPC_PORT: u16 = 50051;
pub const SERVICE_NAME: &str = "auth";

#[async_trait]
impl SessionAuthClient for AuthClient {
    async fn authenticate_session(
        &mut self,
        token: &str,
    ) -> Result<AuthenticatedSession, AuthenticateSessionErr> {
        let req = Request::new(ValidateSessionReq {
            token: token.to_string(),
        });
        let resp = self
            .validate_session(req)
            .await
            .map_err(|e| match e.code() {
                Code::Internal => AuthenticateSessionErr::Internal,
                _ => AuthenticateSessionErr::Unauthenticated,
            })?;
        let resp = resp.into_inner();

        Ok(AuthenticatedSession {
            session_state: SessionState::new(resp.user_id),
            should_refresh_cookie: resp.should_refresh_cookie,
        })
    }
}
