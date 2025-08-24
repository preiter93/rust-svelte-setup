pub mod proto;

use crate::proto::{
    CreateUserReq, CreateUserResp, GetUserIdFromGoogleIdReq, GetUserIdFromGoogleIdResp, GetUserReq,
    GetUserResp, api_service_client::ApiServiceClient,
};
use shared::{middleware::tracing::TracingServiceClient, patched_host};
use std::{error::Error, str::FromStr as _};
use tonic::{
    Request, Response, Status,
    transport::{Channel, Endpoint},
};

const GRPC_PORT: &str = "50052";
pub const SERVICE_NAME: &'static str = "user";

#[derive(Clone)]
pub struct UserClient(ApiServiceClient<TracingServiceClient<Channel>>);

impl UserClient {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let host = patched_host(String::from(SERVICE_NAME));
        let endpoint = Endpoint::from_str(&format!("http://{host}:{GRPC_PORT}"))?;
        let channel = endpoint.connect().await?;
        let client = TracingServiceClient::new(channel);
        let client = ApiServiceClient::new(client);

        Ok(Self(client))
    }

    pub async fn get_user(
        &mut self,
        req: Request<GetUserReq>,
    ) -> Result<Response<GetUserResp>, Status> {
        self.0.get_user(req).await
    }

    pub async fn get_user_id_from_google_id(
        &mut self,
        req: Request<GetUserIdFromGoogleIdReq>,
    ) -> Result<Response<GetUserIdFromGoogleIdResp>, Status> {
        self.0.get_user_id_from_google_id(req).await
    }

    pub async fn create_user(
        &mut self,
        req: Request<CreateUserReq>,
    ) -> Result<Response<CreateUserResp>, Status> {
        self.0.create_user(req).await
    }
}
