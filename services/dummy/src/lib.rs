pub mod proto;

use crate::proto::{
    CreateEntityReq, CreateEntityResp, GetEntityReq, GetEntityResp,
    api_service_client::ApiServiceClient,
};
use shared::{middleware::tracing::TracingServiceClient, patched_host};
use std::{error::Error, str::FromStr as _};
use tonic::{
    Request, Response, Status,
    transport::{Channel, Endpoint},
};

pub const GRPC_PORT: u16 = 50051;
pub const SERVICE_NAME: &str = "dummy";

#[derive(Clone)]
pub struct DummyClient(ApiServiceClient<TracingServiceClient<Channel>>);

impl DummyClient {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let host = patched_host(String::from(SERVICE_NAME));
        let endpoint = Endpoint::from_str(&format!("http://{host}:{GRPC_PORT}"))?;
        let channel = endpoint.connect().await?;
        let client = TracingServiceClient::new(channel);
        let client = ApiServiceClient::new(client);

        Ok(Self(client))
    }

    pub async fn get_entity(
        &mut self,
        req: Request<GetEntityReq>,
    ) -> Result<Response<GetEntityResp>, Status> {
        self.0.get_entity(req).await
    }

    pub async fn create_entity(
        &mut self,
        req: Request<CreateEntityReq>,
    ) -> Result<Response<CreateEntityResp>, Status> {
        self.0.create_entity(req).await
    }
}
