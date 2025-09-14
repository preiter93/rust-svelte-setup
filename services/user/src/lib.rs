pub mod proto;

use crate::proto::{
    CreateUserReq, CreateUserResp, GetUserReq, GetUserResp, api_service_client::ApiServiceClient,
};
use shared::{middleware::tracing::TracingServiceClient, patched_host};
use std::{error::Error, str::FromStr as _};
use tonic::{
    Request, Response, Status, async_trait,
    transport::{Channel, Endpoint},
};

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
    async fn get_user(&self, req: Request<GetUserReq>) -> Result<Response<GetUserResp>, Status>;

    async fn create_user(
        &self,
        req: Request<CreateUserReq>,
    ) -> Result<Response<CreateUserResp>, Status>;
}

#[async_trait]
impl IUserClient for UserClient {
    async fn get_user(&self, req: Request<GetUserReq>) -> Result<Response<GetUserResp>, Status> {
        self.0.clone().get_user(req).await
    }

    async fn create_user(
        &self,
        req: Request<CreateUserReq>,
    ) -> Result<Response<CreateUserResp>, Status> {
        self.0.clone().create_user(req).await
    }
}

#[cfg(feature = "test-utils")]
pub mod test_utils {
    use super::*;
    use tokio::sync::Mutex;
    use tonic::{Request, Response, Status};

    pub struct MockUserClient {
        pub get_user_req: Mutex<Option<GetUserReq>>,
        pub get_user_resp: Mutex<Option<Result<GetUserResp, Status>>>,
        pub create_user_req: Mutex<Option<CreateUserReq>>,
        pub create_user_resp: Mutex<Option<Result<CreateUserResp, Status>>>,
    }

    impl Default for MockUserClient {
        fn default() -> Self {
            Self {
                get_user_req: Mutex::new(None),
                get_user_resp: Mutex::new(None),
                create_user_req: Mutex::new(None),
                create_user_resp: Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl IUserClient for MockUserClient {
        async fn get_user(
            &self,
            req: Request<GetUserReq>,
        ) -> Result<Response<GetUserResp>, Status> {
            *self.get_user_req.lock().await = Some(req.into_inner());

            self.get_user_resp
                .lock()
                .await
                .take()
                .unwrap()
                .map(Response::new)
        }

        async fn create_user(
            &self,
            req: Request<CreateUserReq>,
        ) -> Result<Response<CreateUserResp>, Status> {
            *self.create_user_req.lock().await = Some(req.into_inner());

            self.create_user_resp
                .lock()
                .await
                .take()
                .unwrap()
                .map(Response::new)
        }
    }
}
