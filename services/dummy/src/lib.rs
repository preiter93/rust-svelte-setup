pub mod proto;

use crate::proto::{GetEntityReq, GetEntityResp, api_service_client::ApiServiceClient};
use setup::{middleware::tracing::TracingServiceClient, patched_host};
use std::{error::Error, str::FromStr as _};
use tonic::{
    Request, Response, Status, async_trait,
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
}

#[async_trait]
pub trait IDummyClient {
    async fn get_entity(
        &self,
        req: Request<GetEntityReq>,
    ) -> Result<Response<GetEntityResp>, Status>;
}

#[async_trait]
impl IDummyClient for DummyClient {
    async fn get_entity(
        &self,
        req: Request<GetEntityReq>,
    ) -> Result<Response<GetEntityResp>, Status> {
        self.0.clone().get_entity(req).await
    }
}

#[cfg(feature = "test-utils")]
pub mod test_utils {
    use super::*;
    use tokio::sync::Mutex;
    use tonic::{Request, Response, Status};

    pub struct MockDummyClient {
        pub get_entity_req: Mutex<Option<GetEntityReq>>,
        pub get_entity_resp: Mutex<Option<Result<GetEntityResp, Status>>>,
    }

    impl Default for MockDummyClient {
        fn default() -> Self {
            Self {
                get_entity_req: Mutex::new(None),
                get_entity_resp: Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl IDummyClient for MockDummyClient {
        async fn get_entity(
            &self,
            req: Request<GetEntityReq>,
        ) -> Result<Response<GetEntityResp>, Status> {
            *self.get_entity_req.lock().await = Some(req.into_inner());

            self.get_entity_resp
                .lock()
                .await
                .take()
                .unwrap()
                .map(Response::new)
        }
    }
}
