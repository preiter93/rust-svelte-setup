//! # Session-based authentication:
//! - The user logs in with a username and password
//! - The server authenticates the user and generates a session token
//! - The session token is stored in the database together with user info
//! - The token is sent to the client and stored in a cookie or local storage
//! - For requests the client sends the session token
//! - The server fetches user id from the token via the database and authorizes the user
//!
//! # Further readings
//! <https://lucia-auth.com/sessions/basic>
use std::marker::PhantomData;

use crate::{
    db::DBClient,
    oauth::{github::GithubOAuth, google::GoogleOAuth},
    proto::{
        CreateSessionReq, CreateSessionResp, DeleteSessionReq, DeleteSessionResp,
        GetOauthAccountReq, GetOauthAccountResp, HandleOauthCallbackReq, HandleOauthCallbackResp,
        LinkOauthAccountReq, LinkOauthAccountResp, StartOauthLoginReq, StartOauthLoginResp,
        ValidateSessionReq, ValidateSessionResp, auth_service_server::AuthService,
    },
};
use common::{Now, SystemNow};
use oauth::RandomSource;
use tonic::{Request, Response, Status};
use tracing::instrument;

#[derive(Clone)]
pub struct Server<D, R, N> {
    pub db: D,
    pub google: GoogleOAuth<R>,
    pub github: GithubOAuth<R>,
    pub(crate) _now: PhantomData<N>,
}

impl<D, R> Server<D, R, SystemNow> {
    pub fn new(db: D, google: GoogleOAuth<R>, github: GithubOAuth<R>) -> Self {
        Self {
            db,
            google,
            github,
            _now: PhantomData,
        }
    }
}

pub(crate) type SessionToken = String;

#[tonic::async_trait]
impl<D, R, N> AuthService for Server<D, R, N>
where
    D: DBClient,
    R: RandomSource + Clone,
    N: Now,
{
    #[instrument(skip_all, fields(user_id), err)]
    async fn create_session(
        &self,
        req: Request<CreateSessionReq>,
    ) -> Result<Response<CreateSessionResp>, Status> {
        self.create_session(req).await
    }

    #[instrument(skip_all, fields(user_id), err)]
    async fn validate_session(
        &self,
        req: Request<ValidateSessionReq>,
    ) -> Result<Response<ValidateSessionResp>, Status> {
        self.validate_session(req).await
    }

    #[instrument(skip_all, fields(user_id), err)]
    async fn delete_session(
        &self,
        req: Request<DeleteSessionReq>,
    ) -> Result<Response<DeleteSessionResp>, Status> {
        self.delete_session(req).await
    }

    #[instrument(skip_all, fields(user_id), err)]
    async fn start_oauth_login(
        &self,
        req: Request<StartOauthLoginReq>,
    ) -> Result<Response<StartOauthLoginResp>, Status> {
        self.start_oauth_login(req).await
    }

    #[instrument(skip_all, fields(user_id), err)]
    async fn handle_oauth_callback(
        &self,
        req: Request<HandleOauthCallbackReq>,
    ) -> Result<Response<HandleOauthCallbackResp>, Status> {
        self.handle_oauth_callback(req).await
    }

    #[instrument(skip_all, fields(user_id), err)]
    async fn link_oauth_account(
        &self,
        req: Request<LinkOauthAccountReq>,
    ) -> Result<Response<LinkOauthAccountResp>, Status> {
        self.link_oauth_account(req).await
    }

    #[instrument(skip_all, fields(user_id), err)]
    async fn get_oauth_account(
        &self,
        req: Request<GetOauthAccountReq>,
    ) -> Result<Response<GetOauthAccountResp>, Status> {
        self.get_oauth_account(req).await
    }
}
