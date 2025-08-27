#![allow(dead_code)]
use crate::{
    db::PostgresDBClient,
    handler::Handler,
    proto::api_service_server::ApiServiceServer,
    utils::{GithubOAuth, GoogleOAuth, StdRandomStringGenerator},
};
use auth::{GRPC_PORT, SERVICE_NAME};
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use dotenv::dotenv;
use shared::{
    middleware::TracingGrpcServiceLayer, patched_host, run_db_migrations, tracing::init_tracer,
};
use std::error::Error;
use tonic::transport::Server;

pub(crate) mod db;
pub(crate) mod error;
pub(crate) mod handler;
#[allow(clippy::all)]
pub(crate) mod proto;
pub(crate) mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let tracer = init_tracer(SERVICE_NAME)?;

    let cfg = Config::from_env();

    let pool = connect_to_db(&cfg)?;
    run_db_migrations!(pool, "./migrations");

    let server = Handler::new(
        PostgresDBClient::new(pool),
        GoogleOAuth::<StdRandomStringGenerator>::new(
            cfg.google_client_id,
            cfg.google_client_secret,
            cfg.google_redirect_uri,
        ),
        GithubOAuth::<StdRandomStringGenerator>::new(
            cfg.github_client_id,
            cfg.github_client_secret,
            cfg.github_redirect_uri,
        ),
    );

    let address = format!("0.0.0.0:{GRPC_PORT}").parse()?;
    let service = ApiServiceServer::new(server);

    println!("listening on :{GRPC_PORT}");
    let mut server = Server::builder().layer(TracingGrpcServiceLayer);
    server.add_service(service).serve(address).await.unwrap();

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
    github_client_id: String,
    github_client_secret: String,
    github_redirect_uri: String,
}

impl Config {
    fn must_get_env(key: &str) -> String {
        std::env::var(key).expect(&format!("{key} must be set"))
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
            google_client_id: Self::must_get_env("GOOGLE_CLIENT_ID"),
            google_client_secret: Self::must_get_env("GOOGLE_CLIENT_SECRET"),
            google_redirect_uri: Self::must_get_env("GOOGLE_REDIRECT_URI"),
            github_client_id: Self::must_get_env("GITHUB_CLIENT_ID"),
            github_client_secret: Self::must_get_env("GITHUB_CLIENT_SECRET"),
            github_redirect_uri: Self::must_get_env("GITHUB_REDIRECT_URI"),
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
