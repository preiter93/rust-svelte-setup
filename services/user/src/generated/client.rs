// This file is generated.
use crate::proto::CreateUserReq;
use crate::proto::CreateUserResp;
use crate::proto::GetUserReq;
use crate::proto::GetUserResp;
use crate::proto::api_service_client::ApiServiceClient;
use setup::{middleware::tracing::TracingServiceClient, patched_host};
use std::{error::Error, str::FromStr as _};
use tonic::transport::{Channel, Endpoint};
use tonic::{Request, Response, Status, async_trait};

pub const GRPC_PORT: u16 = 50051;
pub const SERVICE_NAME: &str = "user";

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
}

#[async_trait]
pub trait IUserClient {
    #[rustfmt::skip]
    async fn create_user(&self, req: Request<CreateUserReq>) -> Result<Response<CreateUserResp>, Status>;
    #[rustfmt::skip]
    async fn get_user(&self, req: Request<GetUserReq>) -> Result<Response<GetUserResp>, Status>;
}

#[async_trait]
impl IUserClient for UserClient {
    #[rustfmt::skip]
    async fn create_user(&self, req: Request<CreateUserReq>) -> Result<Response<CreateUserResp>, Status> {
        self.0.clone().create_user(req).await
    }
    #[rustfmt::skip]
    async fn get_user(&self, req: Request<GetUserReq>) -> Result<Response<GetUserResp>, Status> {
        self.0.clone().get_user(req).await
    }
}

#[cfg(feature = "test-utils")]
pub mod test_utils {
    use super::*;
    use tokio::sync::Mutex;
    use tonic::{Request, Response, Status};

    pub struct MockUserClient {
        pub create_user_req: Mutex<Option<CreateUserReq>>,
        pub create_user_resp: Mutex<Option<Result<CreateUserResp, Status>>>,
        pub get_user_req: Mutex<Option<GetUserReq>>,
        pub get_user_resp: Mutex<Option<Result<GetUserResp, Status>>>,
    }

    impl Default for MockUserClient {
        fn default() -> Self {
            Self {
                create_user_req: Mutex::new(None),
                create_user_resp: Mutex::new(None),
                get_user_req: Mutex::new(None),
                get_user_resp: Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl IUserClient for MockUserClient {
        #[rustfmt::skip]
        async fn create_user(&self, req: Request<CreateUserReq>) -> Result<Response<CreateUserResp>, Status> {
            *self.create_user_req.lock().await = Some(req.into_inner());
            self.create_user_resp.lock().await.take().unwrap().map(Response::new)
        }
        #[rustfmt::skip]
        async fn get_user(&self, req: Request<GetUserReq>) -> Result<Response<GetUserResp>, Status> {
            *self.get_user_req.lock().await = Some(req.into_inner());
            self.get_user_resp.lock().await.take().unwrap().map(Response::new)
        }
    }
}
