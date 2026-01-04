use std::error::Error;

use super::config::PGConfig;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::NoTls;

/// Create a PostgreSQL connection pool.
///
/// # Errors
///
/// Returns an error if the connection pool cannot be created.
pub fn connect(cfg: &PGConfig) -> Result<Pool, Box<dyn Error>> {
    let mut pg = tokio_postgres::Config::new();
    pg.dbname(&cfg.dbname)
        .user(&cfg.user)
        .password(&cfg.password)
        .host(&cfg.host)
        .port(cfg.port);

    let manager = Manager::from_config(
        pg,
        NoTls,
        ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        },
    );

    Pool::builder(manager)
        .build()
        .map_err(|e| format!("failed to connect to postgres: {e}").into())
}
