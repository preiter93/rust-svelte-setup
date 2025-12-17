use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod, tokio_postgres};
use refinery::Runner;
use std::error::Error;
use std::ops::DerefMut;
use std::path::Path;
use testcontainers::ContainerAsync;
use testcontainers::{
    GenericImage, ImageExt,
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
};
use tokio::sync::OnceCell;
use tonic::{Code, Response, Status};

/// Represents a test database running in a container.
struct TestDb {
    /// The underlying PostgreSQL container.
    postgres: ContainerAsync<GenericImage>,
}

/// A global singleton holding the test database.
/// OnceCell ensures the DB is started only once across all tests.
static TEST_DB: OnceCell<TestDb> = OnceCell::const_new();

/// Returns a connection pool to the test database.
///
/// If the test database hasn’t been started yet, it will start it first.
pub async fn get_test_db(
    service_name: &str,
    migrations: impl AsRef<Path>,
) -> Result<Pool, Box<dyn Error>> {
    let db = TEST_DB
        .get_or_init(|| async { start_test_db(service_name, migrations).await.unwrap() })
        .await;
    let pool = create_connection_pool(service_name, &db.postgres).await?;
    Ok(pool)
}

/// Shutdown postgres container when the process exits.
///
/// Note:
/// A static OnceCell does not automatically Drop when the program is
/// terminated. That means test containers won’t be cleaned up
/// automatically thus we explicitly stop the postgres container here.
///
/// For more context, see:  
/// <https://github.com/testcontainers/testcontainers-rs/issues/707>
#[dtor::dtor]
fn on_shutdown() {
    let Some(test_db) = TEST_DB.get() else {
        return;
    };
    let container_id = test_db.postgres.id();

    std::process::Command::new("docker")
        .args(["container", "rm", "-f", container_id])
        .output()
        .expect("failed to stop testcontainer");
}

async fn start_test_db(
    service_name: &str,
    migrations: impl AsRef<Path>,
) -> Result<TestDb, Box<dyn Error>> {
    let pg_port = 5432;
    let postgres = GenericImage::new("postgres", "latest")
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
        .expect("Failed to start postgres");

    let pool = create_connection_pool(service_name, &postgres).await?;

    let mut connection = pool.get().await?;
    let migrations = refinery::load_sql_migrations(migrations)?;
    let _ = Runner::new(&migrations)
        .run_async(connection.deref_mut().deref_mut())
        .await?;

    Ok(TestDb { postgres })
}

async fn create_connection_pool(
    service_name: &str,
    postgres: &ContainerAsync<GenericImage>,
) -> Result<Pool, Box<dyn Error>> {
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

    Ok(pool)
}

/// Asserts that a gRPC response matches the expected result.
pub fn assert_response<T: PartialEq + std::fmt::Debug>(
    got: Result<Response<T>, Status>,
    want: Result<T, Code>,
) {
    match (got, want) {
        (Ok(got), Ok(want)) => assert_eq!(got.into_inner(), want),
        (Err(got), Err(want)) => assert_eq!(got.code(), want),
        (Ok(got), Err(want)) => panic!("left: {got:?}\nright: {want}"),
        (Err(got), Ok(want)) => panic!("left: {got}\nright: {want:?}"),
    }
}
