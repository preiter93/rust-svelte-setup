#![allow(dead_code)]
use crate::{
    db::PostgresDBClient,
    handler::Handler,
    oauth::{config::OauthConfig, github::GithubOAuth, google::GoogleOAuth},
    proto::auth_service_server::AuthServiceServer,
};
use auth::{GRPC_PORT, SERVICE_NAME};
use dotenv::dotenv;
use setup::{middleware::TracingGrpcServiceLayer, tracing::init_tracer};
use std::error::Error;

pub(crate) mod create_session;
pub(crate) mod db;
pub(crate) mod delete_session;
pub(crate) mod error;
pub(crate) mod get_oauth_account;
pub(crate) mod handle_oauth_callback;
pub(crate) mod handler;
pub(crate) mod link_oauth_account;
pub(crate) mod oauth;
#[allow(clippy::all)]
pub(crate) mod proto;
pub(crate) mod start_oauth_login;
pub(crate) mod utils;
pub(crate) mod validate_session;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let tracer = init_tracer(SERVICE_NAME)?;

    let pg_cfg = database::PGConfig::from_env(SERVICE_NAME)?;
    let pool = database::connect(&pg_cfg)?;
    database::run_migrations!(pool, "./migrations");

    let oauth_cfg = OauthConfig::from_env();
    let handler = Handler::new(
        PostgresDBClient::new(pool),
        GoogleOAuth::from_config(&oauth_cfg),
        GithubOAuth::from_config(&oauth_cfg),
    );

    let address = format!("0.0.0.0:{GRPC_PORT}").parse()?;
    let service = AuthServiceServer::new(handler);

    println!("listening on :{GRPC_PORT}");
    let mut server = tonic::transport::Server::builder().layer(TracingGrpcServiceLayer);
    server.add_service(service).serve(address).await.unwrap();

    tracer.shutdown()?;

    Ok(())
}
