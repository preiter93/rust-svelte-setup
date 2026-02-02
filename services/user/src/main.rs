pub mod create_user;
pub mod db;
pub mod error;
pub mod get_user;
pub mod handler;
#[allow(clippy::all)]
pub mod proto;

#[cfg(test)]
mod fixture;

use crate::{handler::Handler, proto::user_service_server::UserServiceServer};
use common::UuidV4Generator;
use db::PostgresDBClient;
use dotenv::dotenv;
use setup::{middleware::TracingGrpcServiceLayer, tracing::init_tracer};
use std::error::Error;
use user::{GRPC_PORT, SERVICE_NAME};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let tracer = init_tracer(SERVICE_NAME)?;

    let pg_cfg = database::PGConfig::from_env(SERVICE_NAME)?;
    let pool = database::connect(&pg_cfg)?;
    database::run_migrations!(pool, "./migrations");

    let handler = Handler {
        db: PostgresDBClient::new(pool),
        uuid: UuidV4Generator,
    };

    let addr = format!("0.0.0.0:{GRPC_PORT}").parse()?;
    let svc = UserServiceServer::new(handler);

    println!("listening on :{GRPC_PORT}");
    let mut server = tonic::transport::Server::builder().layer(TracingGrpcServiceLayer);
    server.add_service(svc).serve(addr).await.unwrap();

    tracer.shutdown()?;

    Ok(())
}
