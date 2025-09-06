pub mod db;
pub mod error;
pub mod handler;
#[allow(clippy::all)]
pub mod proto;
pub mod utils;

use crate::{handler::Handler, utils::UuidV4Generator};
use db::PostgresDBClient;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use dotenv::dotenv;
use dummy::{GRPC_PORT, SERVICE_NAME};
use proto::api_service_server::ApiServiceServer;
use shared::{
    middleware::TracingGrpcServiceLayer, patched_host, run_db_migrations, tracing::init_tracer,
};
use std::error::Error;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let tracer = init_tracer(SERVICE_NAME)?;

    let cfg = Config::from_env();

    let pool = connect_to_db(&cfg)?;
    run_db_migrations!(pool, "./migrations");

    let server = Handler {
        db: PostgresDBClient::new(pool),
        uuid: UuidV4Generator,
    };

    let addr = format!("0.0.0.0:{GRPC_PORT}").parse()?;
    let svc = ApiServiceServer::new(server);

    println!("listening on :{GRPC_PORT}");
    let mut server = Server::builder().layer(TracingGrpcServiceLayer);
    server.add_service(svc).serve(addr).await.unwrap();

    tracer.shutdown()?;

    Ok(())
}

struct Config {
    pg_dbname: String,
    pg_password: String,
    pg_user: String,
    pg_host: String,
    pg_port: u16,
}

impl Config {
    fn must_get_env(key: &str) -> String {
        std::env::var(key).unwrap_or_else(|_| panic!("{key} must be set"))
    }

    pub fn from_env() -> Self {
        let pg_port = Self::must_get_env("PG_PORT")
            .parse()
            .expect("failed to parse PG_PORT");

        Self {
            pg_dbname: format!("{SERVICE_NAME}_db"),
            pg_password: Self::must_get_env("PG_PASSWORD"),
            pg_user: Self::must_get_env("PG_USER"),
            pg_host: patched_host(Self::must_get_env("PG_HOST")),
            pg_port,
        }
    }
}

fn connect_to_db(cfg: &Config) -> Result<Pool, Box<dyn Error>> {
    let mut pg_config = tokio_postgres::Config::new();
    pg_config
        .dbname(cfg.pg_dbname.clone())
        .user(cfg.pg_user.clone())
        .password(cfg.pg_password.clone())
        .host(cfg.pg_host.clone())
        .port(cfg.pg_port);

    let manager = Manager::from_config(
        pg_config,
        tokio_postgres::NoTls,
        ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        },
    );

    Ok(Pool::builder(manager)
        .build()
        .map_err(|e| format!("failed to connect to db: {e}"))?)
}
