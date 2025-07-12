pub mod proto;

use crate::proto::{CreateSessionReq, CreateSessionResp, api_service_client::ApiServiceClient};
use common_utils::grpc::interceptors::GrpcServiceInterceptor;
use std::{error::Error, str::FromStr as _};
use tonic::{
    Request, Response, Status,
    service::interceptor::InterceptedService,
    transport::{Channel, Endpoint},
};

#[derive(Clone)]
pub struct AuthClient(ApiServiceClient<InterceptedService<Channel, GrpcServiceInterceptor>>);

impl AuthClient {
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

    pub async fn create_session(
        &mut self,
        req: Request<CreateSessionReq>,
    ) -> Result<Response<CreateSessionResp>, Status> {
        self.0.create_session(req).await
    }
}
