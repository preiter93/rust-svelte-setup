pub mod testcontainers;

use std::{error::Error, str::FromStr};

use auth::proto::{CreateSessionReq, auth_service_client::AuthServiceClient as AuthClient};
use axum::http::{HeaderMap, HeaderValue};
use reqwest::header::COOKIE;
use tonic::{Request, transport::Endpoint};
use user::proto::{CreateUserReq, User, user_service_client::UserServiceClient as UserClient};

use crate::utils::testcontainers::TestContainers;

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct AuthenticatedUser {
    pub(crate) user: User,
    pub(crate) token: String,
}

impl AuthenticatedUser {
    pub(crate) fn get_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            HeaderValue::from_str(&format!("session_token={}", self.token)).unwrap(),
        );
        headers
    }
}

pub(crate) async fn create_authenticated_user(
    containers: &TestContainers,
) -> Result<AuthenticatedUser, Box<dyn Error>> {
    let host = containers.user.get_host().await.unwrap();

    let port = containers.auth.get_host_port_ipv4(auth::GRPC_PORT).await;
    let endpoint = Endpoint::from_str(&format!("http://{host}:{}", port.unwrap()))?;
    let channel = endpoint.connect().await?;
    let mut auth_client = AuthClient::new(channel);

    let port = containers.user.get_host_port_ipv4(user::GRPC_PORT).await;
    let endpoint = Endpoint::from_str(&format!("http://{host}:{}", port.unwrap()))?;
    let channel = endpoint.connect().await?;
    let mut user_client = UserClient::new(channel);

    let req = Request::new(CreateUserReq {
        name: "integration-test-name".to_string(),
        email: "integration-test-email".to_string(),
    });
    let resp = user_client.create_user(req).await?;
    let user = resp.into_inner().user.unwrap();

    let req = Request::new(CreateSessionReq {
        user_id: user.id.clone(),
    });
    let resp = auth_client.create_session(req).await?;
    let token = resp.into_inner().token;

    Ok(AuthenticatedUser { user, token })
}
