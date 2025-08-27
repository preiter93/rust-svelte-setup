use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod, tokio_postgres};
use refinery::Runner;
use std::error::Error;
use std::ops::DerefMut;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock, Weak};
use testcontainers::ContainerAsync;
use testcontainers::{
    GenericImage, ImageExt,
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
};

pub struct TestDb {
    pub pool: Pool,
    pub postgres: ContainerAsync<GenericImage>,
}

static TEST_DB: OnceLock<Mutex<Weak<TestDb>>> = OnceLock::new();

pub async fn get_test_db(
    service_name: &str,
    migrations: impl AsRef<Path>,
) -> Result<Arc<TestDb>, Box<dyn Error>> {
    let mut guard = TEST_DB
        .get_or_init(|| Mutex::new(Weak::new()))
        .lock()
        .unwrap();

    if let Some(test_db) = guard.upgrade() {
        return Ok(test_db);
    }

    let postgres = start_postgres().await;
    let host = postgres.get_host().await?;
    let port = postgres.get_host_port_ipv4(5432).await?;

    let mut config = tokio_postgres::Config::new();
    config
        .dbname(format!("{service_name}_db"))
        .user("postgres")
        .password("postgres")
        .host(host.to_string())
        .port(port);

    let pool = Pool::builder(Manager::from_config(
        config,
        tokio_postgres::NoTls,
        ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        },
    ))
    .build()
    .map_err(|e| format!("failed to connect to db: {e}"))?;

    let mut connection = pool.get().await?;
    let migrations = refinery::load_sql_migrations(migrations)?;
    let _ = Runner::new(&migrations)
        .run_async(connection.deref_mut().deref_mut())
        .await?;

    let test_db = Arc::new(TestDb { pool, postgres });
    *guard = Arc::downgrade(&test_db);

    Ok(test_db)
}

async fn start_postgres() -> ContainerAsync<GenericImage> {
    let pg_port = 5432;
    GenericImage::new("postgres", "latest")
        .with_exposed_port(ContainerPort::Tcp(pg_port))
        .with_wait_for(WaitFor::message_on_stdout(
            "database system is ready to accept connections",
        ))
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_network("shared_network")
        .with_copy_to(
            "/docker-entrypoint-initdb.d/init.sql",
            include_bytes!("../../../../infrastructure/db/init.sql").to_vec(),
        )
        .with_env_var("PGPORT", pg_port.to_string())
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .with_env_var("POSTGRES_DB", "postgres")
        .start()
        .await
        .expect("Failed to start postgres")
}
