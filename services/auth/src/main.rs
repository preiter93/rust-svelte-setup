#![allow(dead_code)]
use crate::{db::DBCLient, handler::Handler, proto::api_service_server::ApiServiceServer};
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use dotenv::dotenv;
use std::error::Error;

mod db;
mod handler;
#[allow(clippy::all)]
pub mod proto;
mod utils;

const GRPC_PORT: &str = "50051";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let cfg = Config::from_env();

    let pool = connect_to_db(&cfg)?;
    run_db_migrations(&pool).await?;

    let server = Handler {
        db: DBCLient::new(pool),
    };

    let addr = format!("0.0.0.0:{GRPC_PORT}").parse()?;
    let svc = ApiServiceServer::new(server);
    println!("listening on :{GRPC_PORT}");
    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr)
        .await
        .expect("Failed to run gRPC server");

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
        std::env::var(key).expect(&format!("{key} must be set"))
    }

    pub fn from_env() -> Self {
        let pg_port_str = Self::must_get_env("PG_PORT");
        Self {
            pg_dbname: Self::must_get_env("PG_DBNAME"),
            pg_password: Self::must_get_env("PG_PASSWORD"),
            pg_user: Self::must_get_env("PG_USER"),
            pg_host: if std::env::var("LOCAL").unwrap_or_default() == "true" {
                Self::must_get_env("DB_HOST_LOCAL")
            } else {
                Self::must_get_env("DB_HOST_REMOVE")
            },
            pg_port: pg_port_str.parse().expect("failed to parse PG_PORT"),
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

async fn run_db_migrations(pool: &Pool) -> std::result::Result<(), Box<dyn Error>> {
    use std::ops::DerefMut;

    refinery::embed_migrations!("migrations");
    let mut conn = pool.get().await?;
    let client = conn.deref_mut().deref_mut();
    let migration_report = migrations::runner().run_async(client).await?;

    for migration in migration_report.applied_migrations() {
        println!(
            "Migration Applied: V{}_{}",
            migration.version(),
            migration.name(),
        );
    }

    Ok(())
}
