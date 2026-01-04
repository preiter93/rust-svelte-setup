#![allow(dead_code)]
use crate::{
    db::PostgresDBClient,
    handler::Handler,
    oauth::{config::OauthConfig, github::GithubOAuth, google::GoogleOAuth},
    proto::api_service_server::ApiServiceServer,
};
use auth::{GRPC_PORT, SERVICE_NAME};
use dotenv::dotenv;
use setup::{middleware::TracingGrpcServiceLayer, tracing::init_tracer};
use std::error::Error;
use tonic::transport::Server;

pub(crate) mod db;
pub(crate) mod error;
pub(crate) mod handler;
pub(crate) mod oauth;
#[allow(clippy::all)]
pub(crate) mod proto;
pub(crate) mod utils;

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
    let service = ApiServiceServer::new(handler);

    println!("listening on :{GRPC_PORT}");
    let mut server = Server::builder().layer(TracingGrpcServiceLayer);
    server.add_service(service).serve(address).await.unwrap();

    tracer.shutdown()?;

    Ok(())
}
