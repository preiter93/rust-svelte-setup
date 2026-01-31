// This file is generated.
use crate::GRPC_PORT;
use crate::SERVICE_NAME;
use crate::proto::GetEntityReq;
use crate::proto::GetEntityResp;
use crate::proto::dummy_service_client::DummyServiceClient;
use setup::{middleware::tracing::TracingServiceClient, patched_host};
use std::{error::Error, str::FromStr as _};
use tonic::transport::{Channel, Endpoint};
use tonic::{Request, Response, Status, async_trait};

#[derive(Clone)]
pub struct DummyClient(DummyServiceClient<TracingServiceClient<Channel>>);

impl DummyClient {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let host = patched_host(String::from(SERVICE_NAME));
        let endpoint = Endpoint::from_str(&format!("http://{host}:{GRPC_PORT}"))?;
        let channel = endpoint.connect().await?;
        let client = TracingServiceClient::new(channel);
        let client = DummyServiceClient::new(client);

        Ok(Self(client))
    }
}

#[rustfmt::skip]
#[async_trait]
pub trait IDummyClient: Send + Sync + 'static {
    async fn get_entity(&self, req: Request<GetEntityReq>) -> Result<Response<GetEntityResp>, Status>;
}

#[rustfmt::skip]
#[async_trait]
impl IDummyClient for DummyClient {
    async fn get_entity(&self, req: Request<GetEntityReq>) -> Result<Response<GetEntityResp>, Status> {
        self.0.clone().get_entity(req).await
    }
}

#[cfg(feature = "testutils")]
pub mod testutils {
    use super::*;
    use tokio::sync::Mutex;
    use tonic::{Request, Response, Status};

    #[rustfmt::skip]
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

    #[rustfmt::skip]
    #[async_trait]
    impl IDummyClient for MockDummyClient {
        async fn get_entity(&self, req: Request<GetEntityReq>) -> Result<Response<GetEntityResp>, Status> {
            *self.get_entity_req.lock().await = Some(req.into_inner());
            self.get_entity_resp.lock().await.take().unwrap().map(Response::new)
        }
    }
}
