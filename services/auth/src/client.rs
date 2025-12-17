// This file is generated.
use crate::GRPC_PORT;
use crate::SERVICE_NAME;
use crate::proto::CreateSessionReq;
use crate::proto::CreateSessionResp;
use crate::proto::DeleteSessionReq;
use crate::proto::DeleteSessionResp;
use crate::proto::GetOauthAccountReq;
use crate::proto::GetOauthAccountResp;
use crate::proto::HandleOauthCallbackReq;
use crate::proto::HandleOauthCallbackResp;
use crate::proto::LinkOauthAccountReq;
use crate::proto::LinkOauthAccountResp;
use crate::proto::StartOauthLoginReq;
use crate::proto::StartOauthLoginResp;
use crate::proto::ValidateSessionReq;
use crate::proto::ValidateSessionResp;
use crate::proto::api_service_client::ApiServiceClient;
use setup::{middleware::tracing::TracingServiceClient, patched_host};
use std::{error::Error, str::FromStr as _};
use tonic::transport::{Channel, Endpoint};
use tonic::{Request, Response, Status, async_trait};

#[derive(Clone)]
pub struct AuthClient(ApiServiceClient<TracingServiceClient<Channel>>);

impl AuthClient {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let host = patched_host(String::from(SERVICE_NAME));
        let endpoint = Endpoint::from_str(&format!("http://{host}:{GRPC_PORT}"))?;
        let channel = endpoint.connect().await?;
        let client = TracingServiceClient::new(channel);
        let client = ApiServiceClient::new(client);

        Ok(Self(client))
    }
}

#[rustfmt::skip]
#[async_trait]
pub trait IAuthClient: Send + Sync + 'static {
    async fn create_session(&self, req: Request<CreateSessionReq>) -> Result<Response<CreateSessionResp>, Status>;
    async fn validate_session(&self, req: Request<ValidateSessionReq>) -> Result<Response<ValidateSessionResp>, Status>;
    async fn delete_session(&self, req: Request<DeleteSessionReq>) -> Result<Response<DeleteSessionResp>, Status>;
    async fn start_oauth_login(&self, req: Request<StartOauthLoginReq>) -> Result<Response<StartOauthLoginResp>, Status>;
    async fn handle_oauth_callback(&self, req: Request<HandleOauthCallbackReq>) -> Result<Response<HandleOauthCallbackResp>, Status>;
    async fn link_oauth_account(&self, req: Request<LinkOauthAccountReq>) -> Result<Response<LinkOauthAccountResp>, Status>;
    async fn get_oauth_account(&self, req: Request<GetOauthAccountReq>) -> Result<Response<GetOauthAccountResp>, Status>;
}

#[rustfmt::skip]
#[async_trait]
impl IAuthClient for AuthClient {
    async fn create_session(&self, req: Request<CreateSessionReq>) -> Result<Response<CreateSessionResp>, Status> {
        self.0.clone().create_session(req).await
    }
    async fn validate_session(&self, req: Request<ValidateSessionReq>) -> Result<Response<ValidateSessionResp>, Status> {
        self.0.clone().validate_session(req).await
    }
    async fn delete_session(&self, req: Request<DeleteSessionReq>) -> Result<Response<DeleteSessionResp>, Status> {
        self.0.clone().delete_session(req).await
    }
    async fn start_oauth_login(&self, req: Request<StartOauthLoginReq>) -> Result<Response<StartOauthLoginResp>, Status> {
        self.0.clone().start_oauth_login(req).await
    }
    async fn handle_oauth_callback(&self, req: Request<HandleOauthCallbackReq>) -> Result<Response<HandleOauthCallbackResp>, Status> {
        self.0.clone().handle_oauth_callback(req).await
    }
    async fn link_oauth_account(&self, req: Request<LinkOauthAccountReq>) -> Result<Response<LinkOauthAccountResp>, Status> {
        self.0.clone().link_oauth_account(req).await
    }
    async fn get_oauth_account(&self, req: Request<GetOauthAccountReq>) -> Result<Response<GetOauthAccountResp>, Status> {
        self.0.clone().get_oauth_account(req).await
    }
}

#[cfg(feature = "testutils")]
pub mod testutils {
    use super::*;
    use tokio::sync::Mutex;
    use tonic::{Request, Response, Status};

    #[rustfmt::skip]
    pub struct MockAuthClient {
        pub create_session_req: Mutex<Option<CreateSessionReq>>,
        pub create_session_resp: Mutex<Option<Result<CreateSessionResp, Status>>>,
        pub validate_session_req: Mutex<Option<ValidateSessionReq>>,
        pub validate_session_resp: Mutex<Option<Result<ValidateSessionResp, Status>>>,
        pub delete_session_req: Mutex<Option<DeleteSessionReq>>,
        pub delete_session_resp: Mutex<Option<Result<DeleteSessionResp, Status>>>,
        pub start_oauth_login_req: Mutex<Option<StartOauthLoginReq>>,
        pub start_oauth_login_resp: Mutex<Option<Result<StartOauthLoginResp, Status>>>,
        pub handle_oauth_callback_req: Mutex<Option<HandleOauthCallbackReq>>,
        pub handle_oauth_callback_resp: Mutex<Option<Result<HandleOauthCallbackResp, Status>>>,
        pub link_oauth_account_req: Mutex<Option<LinkOauthAccountReq>>,
        pub link_oauth_account_resp: Mutex<Option<Result<LinkOauthAccountResp, Status>>>,
        pub get_oauth_account_req: Mutex<Option<GetOauthAccountReq>>,
        pub get_oauth_account_resp: Mutex<Option<Result<GetOauthAccountResp, Status>>>,
    }

    impl Default for MockAuthClient {
        fn default() -> Self {
            Self {
                create_session_req: Mutex::new(None),
                create_session_resp: Mutex::new(None),
                validate_session_req: Mutex::new(None),
                validate_session_resp: Mutex::new(None),
                delete_session_req: Mutex::new(None),
                delete_session_resp: Mutex::new(None),
                start_oauth_login_req: Mutex::new(None),
                start_oauth_login_resp: Mutex::new(None),
                handle_oauth_callback_req: Mutex::new(None),
                handle_oauth_callback_resp: Mutex::new(None),
                link_oauth_account_req: Mutex::new(None),
                link_oauth_account_resp: Mutex::new(None),
                get_oauth_account_req: Mutex::new(None),
                get_oauth_account_resp: Mutex::new(None),
            }
        }
    }

    #[rustfmt::skip]
    #[async_trait]
    impl IAuthClient for MockAuthClient {
        async fn create_session(&self, req: Request<CreateSessionReq>) -> Result<Response<CreateSessionResp>, Status> {
            *self.create_session_req.lock().await = Some(req.into_inner());
            self.create_session_resp.lock().await.take().unwrap().map(Response::new)
        }
        async fn validate_session(&self, req: Request<ValidateSessionReq>) -> Result<Response<ValidateSessionResp>, Status> {
            *self.validate_session_req.lock().await = Some(req.into_inner());
            self.validate_session_resp.lock().await.take().unwrap().map(Response::new)
        }
        async fn delete_session(&self, req: Request<DeleteSessionReq>) -> Result<Response<DeleteSessionResp>, Status> {
            *self.delete_session_req.lock().await = Some(req.into_inner());
            self.delete_session_resp.lock().await.take().unwrap().map(Response::new)
        }
        async fn start_oauth_login(&self, req: Request<StartOauthLoginReq>) -> Result<Response<StartOauthLoginResp>, Status> {
            *self.start_oauth_login_req.lock().await = Some(req.into_inner());
            self.start_oauth_login_resp.lock().await.take().unwrap().map(Response::new)
        }
        async fn handle_oauth_callback(&self, req: Request<HandleOauthCallbackReq>) -> Result<Response<HandleOauthCallbackResp>, Status> {
            *self.handle_oauth_callback_req.lock().await = Some(req.into_inner());
            self.handle_oauth_callback_resp.lock().await.take().unwrap().map(Response::new)
        }
        async fn link_oauth_account(&self, req: Request<LinkOauthAccountReq>) -> Result<Response<LinkOauthAccountResp>, Status> {
            *self.link_oauth_account_req.lock().await = Some(req.into_inner());
            self.link_oauth_account_resp.lock().await.take().unwrap().map(Response::new)
        }
        async fn get_oauth_account(&self, req: Request<GetOauthAccountReq>) -> Result<Response<GetOauthAccountResp>, Status> {
            *self.get_oauth_account_req.lock().await = Some(req.into_inner());
            self.get_oauth_account_resp.lock().await.take().unwrap().map(Response::new)
        }
    }
}
