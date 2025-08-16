#![allow(dead_code)]
use crate::{
    db::DBCLient, handler::Handler, proto::api_service_server::ApiServiceServer, utils::GoogleOAuth,
};
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use dotenv::dotenv;
use shared::{
    middleware::add_tracing_middleware_for_grpc, run_db_migrations, tracing::tracer::init_tracer,
};
use std::error::Error;
use tonic::transport::Server;

mod db;
mod error;
mod handler;
#[allow(clippy::all)]
pub mod proto;
mod utils;

const GRPC_PORT: &str = "50051";
const SERVICE_NAME: &'static str = "auth";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let tracer = init_tracer(SERVICE_NAME)?;

    let cfg = Config::from_env();

    let pool = connect_to_db(&cfg)?;
    run_db_migrations!(pool, "./migrations");

    let server = Handler {
        db: DBCLient::new(pool),
        google: GoogleOAuth::new(
            cfg.google_client_id,
            cfg.google_client_secret,
            cfg.google_redirect_uri,
        ),
    };

    let addr = format!("0.0.0.0:{GRPC_PORT}").parse()?;
    let svc = ApiServiceServer::new(server);

    println!("listening on :{GRPC_PORT}");
    let server = Server::builder();
    let mut server = add_tracing_middleware_for_grpc(server);
    server
        .add_service(svc)
        .serve(addr)
        .await
        .expect("failed to run gRPC server");

    tracer.shutdown()?;

    Ok(())
}

struct Config {
    pg_dbname: String,
    pg_password: String,
    pg_user: String,
    pg_host: String,
    pg_port: u16,
    google_client_id: String,
    google_client_secret: String,
    google_redirect_uri: String,
}

impl Config {
    fn must_get_env(key: &str) -> String {
        std::env::var(key).expect(&format!("{key} must be set"))
    }

    pub fn from_env() -> Self {
        let pg_port_str = Self::must_get_env("PG_PORT");
        Self {
            pg_dbname: format!("{SERVICE_NAME}_db"),
            pg_password: Self::must_get_env("PG_PASSWORD"),
            pg_user: Self::must_get_env("PG_USER"),
            pg_host: if std::env::var("LOCAL").unwrap_or_default() == "true" {
                Self::must_get_env("PG_HOST_LOCAL")
            } else {
                Self::must_get_env("PG_HOST_REMOTE")
            },
            pg_port: pg_port_str.parse().expect("failed to parse PG_PORT"),
            google_client_id: Self::must_get_env("GOOGLE_CLIENT_ID"),
            google_client_secret: Self::must_get_env("GOOGLE_CLIENT_SECRET"),
            google_redirect_uri: Self::must_get_env("GOOGLE_REDIRECT_URI"),
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

    Ok(Pool::builder(manager).build()?)
}
