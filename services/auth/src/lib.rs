pub mod proto;

use crate::proto::{
    CreateSessionReq, CreateSessionResp, DeleteSessionReq, DeleteSessionResp,
    HandleGoogleCallbackReq, HandleGoogleCallbackResp, StartGoogleLoginReq, StartGoogleLoginResp,
    ValidateSessionReq, ValidateSessionResp, api_service_client::ApiServiceClient,
};
use shared::{
    middleware::{GrpcServiceInterceptor, SessionValidator, auth::ValidateSessionErr},
    session::SessionState,
};
use std::{error::Error, str::FromStr};
use tonic::{
    Code, Request, Response, Status, async_trait,
    service::interceptor::InterceptedService,
    transport::{Channel, Endpoint},
};

#[derive(Clone)]
pub struct AuthClient(ApiServiceClient<InterceptedService<Channel, GrpcServiceInterceptor>>);

impl AuthClient {
    /// Creates a new [`AuthClient`].
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let endpoint_url = if std::env::var("LOCAL").unwrap_or_default() == "true" {
            "http://localhost:50051"
        } else {
            "http://auth:50051"
        };
        let endpoint = Endpoint::from_str(endpoint_url)?;
        let channel = endpoint.connect().await?;
        let client = ApiServiceClient::with_interceptor(channel, GrpcServiceInterceptor {});
        Ok(Self(client))
    }

    /// Creates a new user session.
    pub async fn create_session(
        &mut self,
        req: Request<CreateSessionReq>,
    ) -> Result<Response<CreateSessionResp>, Status> {
        self.0.create_session(req).await
    }

    /// Deletes a user session.
    pub async fn delete_session(
        &mut self,
        req: Request<DeleteSessionReq>,
    ) -> Result<Response<DeleteSessionResp>, Status> {
        self.0.delete_session(req).await
    }

    /// Validates a session token and returns the session state.
    pub async fn validate_session(
        &mut self,
        req: Request<ValidateSessionReq>,
    ) -> Result<Response<ValidateSessionResp>, Status> {
        self.0.validate_session(req).await
    }

    /// Starts a google login flow and returns the redirect URL.
    pub async fn start_google_login(
        &mut self,
        req: Request<StartGoogleLoginReq>,
    ) -> Result<Response<StartGoogleLoginResp>, Status> {
        self.0.start_google_login(req).await
    }

    /// Handles google's OAuth callback and finalizes login.
    pub async fn handle_google_callback(
        &mut self,
        req: Request<HandleGoogleCallbackReq>,
    ) -> Result<Response<HandleGoogleCallbackResp>, Status> {
        self.0.handle_google_callback(req).await
    }
}

#[async_trait]
impl SessionValidator for AuthClient {
    async fn validate_session(
        &mut self,
        token: String,
    ) -> Result<SessionState, ValidateSessionErr> {
        let req = Request::new(ValidateSessionReq { token });
        let resp = self
            .validate_session(req)
            .await
            .map_err(|e| match e.code() {
                Code::Internal => ValidateSessionErr::Internal,
                _ => ValidateSessionErr::Unauthenticated,
            })?;

        Ok(SessionState::new(resp.into_inner().user_id))
    }
}
