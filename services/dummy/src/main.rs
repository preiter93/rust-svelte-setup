pub mod db;
pub mod error;
pub mod get_entity;
pub mod handler;
#[allow(clippy::all)]
pub mod proto;
pub mod utils;

#[cfg(test)]
mod fixture;

use crate::{handler::Handler, proto::dummy_service_server::DummyServiceServer};
use common::UuidV4Generator;
use db::PostgresDBClient;
use dotenv::dotenv;
use dummy::{GRPC_PORT, SERVICE_NAME};
use setup::{middleware::TracingGrpcServiceLayer, tracing::init_tracer};
use std::error::Error;

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
    let svc = DummyServiceServer::new(handler);

    println!("listening on :{GRPC_PORT}");
    let mut server = tonic::transport::Server::builder().layer(TracingGrpcServiceLayer);
    server.add_service(svc).serve(addr).await.unwrap();

    tracer.shutdown()?;

    Ok(())
}
