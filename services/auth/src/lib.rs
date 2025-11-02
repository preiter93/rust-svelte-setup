pub mod proto;

use crate::proto::{
    CreateSessionReq, CreateSessionResp, DeleteSessionReq, DeleteSessionResp, ValidateSessionReq,
    ValidateSessionResp, api_service_client::ApiServiceClient,
};
use crate::proto::{
    GetOauthAccountReq, GetOauthAccountResp, HandleOauthCallbackReq, HandleOauthCallbackResp,
    LinkOauthAccountReq, LinkOauthAccountResp, StartOauthLoginReq, StartOauthLoginResp,
};
use setup::middleware::auth::AuthenticatedSession;
use setup::middleware::tracing::TracingServiceClient;
use setup::patched_host;
use setup::{
    middleware::{SessionAuthClient, auth::AuthenticateSessionErr},
    session::SessionState,
};
use std::{error::Error, str::FromStr};
use tonic::{
    Code, Request, Response, Status, async_trait,
    transport::{Channel, Endpoint},
};

pub const GRPC_PORT: u16 = 50051;
pub const SERVICE_NAME: &str = "auth";

#[derive(Clone)]
pub struct AuthClient(ApiServiceClient<TracingServiceClient<Channel>>);

impl AuthClient {
    /// Creates a new [`AuthClient`].
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
pub trait IAuthClient: Send + Sync + 'static {
    async fn create_session(
        &self,
        req: Request<CreateSessionReq>,
    ) -> Result<Response<CreateSessionResp>, Status>;

    async fn delete_session(
        &self,
        req: Request<DeleteSessionReq>,
    ) -> Result<Response<DeleteSessionResp>, Status>;

    async fn validate_session(
        &self,
        req: Request<ValidateSessionReq>,
    ) -> Result<Response<ValidateSessionResp>, Status>;

    async fn start_oauth_login(
        &self,
        req: Request<StartOauthLoginReq>,
    ) -> Result<Response<StartOauthLoginResp>, Status>;

    async fn handle_oauth_callback(
        &self,
        req: Request<HandleOauthCallbackReq>,
    ) -> Result<Response<HandleOauthCallbackResp>, Status>;

    async fn link_oauth_account(
        &self,
        req: Request<LinkOauthAccountReq>,
    ) -> Result<Response<LinkOauthAccountResp>, Status>;

    async fn get_oauth_account(
        &self,
        req: Request<GetOauthAccountReq>,
    ) -> Result<Response<GetOauthAccountResp>, Status>;
}

#[async_trait]
impl IAuthClient for AuthClient {
    /// Creates a new user session.
    async fn create_session(
        &self,
        req: Request<CreateSessionReq>,
    ) -> Result<Response<CreateSessionResp>, Status> {
        self.0.clone().create_session(req).await
    }

    /// Deletes a user session.
    async fn delete_session(
        &self,
        req: Request<DeleteSessionReq>,
    ) -> Result<Response<DeleteSessionResp>, Status> {
        self.0.clone().delete_session(req).await
    }

    /// Validates a session token and returns the session state.
    async fn validate_session(
        &self,
        req: Request<ValidateSessionReq>,
    ) -> Result<Response<ValidateSessionResp>, Status> {
        self.0.clone().validate_session(req).await
    }

    /// Starts a oauth login flow and returns the redirect URL.
    async fn start_oauth_login(
        &self,
        req: Request<StartOauthLoginReq>,
    ) -> Result<Response<StartOauthLoginResp>, Status> {
        self.0.clone().start_oauth_login(req).await
    }

    /// Handles OAuth callback and finalizes login.
    async fn handle_oauth_callback(
        &self,
        req: Request<HandleOauthCallbackReq>,
    ) -> Result<Response<HandleOauthCallbackResp>, Status> {
        self.0.clone().handle_oauth_callback(req).await
    }

    /// Links a user id to an oauth token.
    async fn link_oauth_account(
        &self,
        req: Request<LinkOauthAccountReq>,
    ) -> Result<Response<LinkOauthAccountResp>, Status> {
        self.0.clone().link_oauth_account(req).await
    }

    /// Fetch an oauth access token from a user id and a provider.
    async fn get_oauth_account(
        &self,
        req: Request<GetOauthAccountReq>,
    ) -> Result<Response<GetOauthAccountResp>, Status> {
        self.0.clone().get_oauth_account(req).await
    }
}

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

#[cfg(feature = "test-utils")]
pub mod test_utils {
    use tokio::sync::Mutex;

    use super::*;

    #[derive(Default)]
    pub struct MockAuthClient {
        pub create_session_req: Mutex<Option<CreateSessionReq>>,
        pub create_session_resp: Mutex<Option<Result<CreateSessionResp, Status>>>,

        pub delete_session_req: Mutex<Option<DeleteSessionReq>>,
        pub delete_session_resp: Mutex<Option<Result<DeleteSessionResp, Status>>>,

        pub validate_session_req: Mutex<Option<ValidateSessionReq>>,
        pub validate_session_resp: Mutex<Option<Result<ValidateSessionResp, Status>>>,

        pub start_oauth_login_req: Mutex<Option<StartOauthLoginReq>>,
        pub start_oauth_login_resp: Mutex<Option<Result<StartOauthLoginResp, Status>>>,

        pub handle_oauth_callback_req: Mutex<Option<HandleOauthCallbackReq>>,
        pub handle_oauth_callback_resp: Mutex<Option<Result<HandleOauthCallbackResp, Status>>>,

        pub link_oauth_account_req: Mutex<Option<LinkOauthAccountReq>>,
        pub link_oauth_account_resp: Mutex<Option<Result<LinkOauthAccountResp, Status>>>,

        pub get_oauth_account_req: Mutex<Option<GetOauthAccountReq>>,
        pub get_oauth_account_resp: Mutex<Option<Result<GetOauthAccountResp, Status>>>,
    }

    #[async_trait]
    impl IAuthClient for MockAuthClient {
        async fn create_session(
            &self,
            req: Request<CreateSessionReq>,
        ) -> Result<Response<CreateSessionResp>, Status> {
            *self.create_session_req.lock().await = Some(req.into_inner());
            self.create_session_resp
                .lock()
                .await
                .take()
                .unwrap()
                .map(Response::new)
        }

        async fn delete_session(
            &self,
            req: Request<DeleteSessionReq>,
        ) -> Result<Response<DeleteSessionResp>, Status> {
            *self.delete_session_req.lock().await = Some(req.into_inner());
            self.delete_session_resp
                .lock()
                .await
                .take()
                .unwrap()
                .map(Response::new)
        }

        async fn validate_session(
            &self,
            req: Request<ValidateSessionReq>,
        ) -> Result<Response<ValidateSessionResp>, Status> {
            *self.validate_session_req.lock().await = Some(req.into_inner());
            self.validate_session_resp
                .lock()
                .await
                .take()
                .unwrap()
                .map(Response::new)
        }

        async fn start_oauth_login(
            &self,
            req: Request<StartOauthLoginReq>,
        ) -> Result<Response<StartOauthLoginResp>, Status> {
            *self.start_oauth_login_req.lock().await = Some(req.into_inner());
            self.start_oauth_login_resp
                .lock()
                .await
                .take()
                .unwrap()
                .map(Response::new)
        }

        async fn handle_oauth_callback(
            &self,
            req: Request<HandleOauthCallbackReq>,
        ) -> Result<Response<HandleOauthCallbackResp>, Status> {
            *self.handle_oauth_callback_req.lock().await = Some(req.into_inner());
            self.handle_oauth_callback_resp
                .lock()
                .await
                .take()
                .unwrap()
                .map(Response::new)
        }

        async fn link_oauth_account(
            &self,
            req: Request<LinkOauthAccountReq>,
        ) -> Result<Response<LinkOauthAccountResp>, Status> {
            *self.link_oauth_account_req.lock().await = Some(req.into_inner());
            self.link_oauth_account_resp
                .lock()
                .await
                .take()
                .unwrap()
                .map(Response::new)
        }

        async fn get_oauth_account(
            &self,
            req: Request<GetOauthAccountReq>,
        ) -> Result<Response<GetOauthAccountResp>, Status> {
            *self.get_oauth_account_req.lock().await = Some(req.into_inner());
            self.get_oauth_account_resp
                .lock()
                .await
                .take()
                .unwrap()
                .map(Response::new)
        }
    }
}
