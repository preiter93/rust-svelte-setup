use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock, Weak},
    time::Duration,
};
use testcontainers::{core::ContainerPort, runners::AsyncRunner};
use tokio::io::AsyncBufReadExt;

use testcontainers::{ContainerAsync, GenericImage, ImageExt, core::WaitFor};
use tokio::{io::BufReader, time::timeout};

#[allow(dead_code)]
pub(crate) struct TestContainers {
    pub(crate) postgres: ContainerAsync<GenericImage>,
    pub(crate) auth: ContainerAsync<GenericImage>,
    pub(crate) user: ContainerAsync<GenericImage>,
    pub(crate) gateway: ContainerAsync<GenericImage>,
}

/// [`TESTCONTAINERS`] ensures that containers are shared across all integration tests.
/// It uses a [`Weak`] reference so that [`TestContainers`] are automatically dropped
/// and Docker containers are cleaned up after all tests have completed.
static TESTCONTAINERS: OnceLock<Mutex<Weak<TestContainers>>> = OnceLock::new();

pub async fn get_test_containers() -> Arc<TestContainers> {
    let mut guard = TESTCONTAINERS
        .get_or_init(|| Mutex::new(Weak::new()))
        .lock()
        .unwrap();

    if let Some(container) = guard.upgrade() {
        return container;
    }

    let container = Arc::new(TestContainers::init().await);
    *guard = Arc::downgrade(&container);
    container
}

impl TestContainers {
    async fn init() -> Self {
        let pg_port = 5432;
        let pg_host = "db";
        let pg_port_str = pg_port.to_string();

        let postgres = run_postgres(pg_host, pg_port).await;
        let auth = run_auth_service(pg_host, &pg_port_str).await;
        let user = run_user_service(pg_host, &pg_port_str).await;
        let gateway = run_gateway_service(pg_host, &pg_port_str).await;

        TestContainers {
            postgres,
            auth,
            user,
            gateway,
        }
    }

    pub async fn gateway_uri(&self) -> String {
        format!(
            "http://{}:{}",
            self.gateway_host().await,
            self.gateway_port().await
        )
    }

    async fn gateway_host(&self) -> String {
        self.gateway.get_host().await.unwrap().to_string()
    }

    async fn gateway_port(&self) -> u16 {
        let port = gateway::HTTP_PORT;
        self.gateway.get_host_port_ipv4(port).await.unwrap()
    }
}

async fn run_postgres(pg_host: &str, pg_port: u16) -> ContainerAsync<GenericImage> {
    GenericImage::new("postgres", "latest")
        .with_exposed_port(ContainerPort::Tcp(pg_port))
        .with_wait_for(WaitFor::message_on_stdout(
            "database system is ready to accept connections",
        ))
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_network("shared_network")
        .with_container_name(format!("{pg_host}-integration-test"))
        .with_copy_to(
            "/docker-entrypoint-initdb.d/init.sql",
            include_bytes!("../../../../infrastructure/db/init.sql").to_vec(),
        )
        .with_env_var("APP_ENV", "integration-test")
        .with_env_var("PGPORT", pg_port.to_string())
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .with_env_var("POSTGRES_DB", "postgres")
        .start()
        .await
        .expect("Failed to start postgres")
}

async fn run_auth_service(pg_host: &str, pg_port: &str) -> ContainerAsync<GenericImage> {
    let mut auth_env_vars = HashMap::new();
    auth_env_vars.insert("GOOGLE_CLIENT_ID", "test");
    auth_env_vars.insert("GOOGLE_CLIENT_SECRET", "test");
    auth_env_vars.insert("GOOGLE_REDIRECT_URI", "test");
    let exposed_port = Some(auth::GRPC_PORT);
    run_service_container("auth", pg_host, pg_port, auth_env_vars, exposed_port).await
}

async fn run_user_service(pg_host: &str, pg_port: &str) -> ContainerAsync<GenericImage> {
    let exposed_port = Some(user::GRPC_PORT);
    run_service_container("user", pg_host, pg_port, HashMap::new(), exposed_port).await
}

async fn run_gateway_service(pg_host: &str, pg_port: &str) -> ContainerAsync<GenericImage> {
    let exposed_port = Some(gateway::HTTP_PORT);
    run_service_container("gateway", pg_host, pg_port, HashMap::new(), exposed_port).await
}

async fn run_service_container(
    service_name: &str,
    pg_host: &str,
    pg_port: &str,
    env_vars: HashMap<&'static str, &'static str>,
    exposed_port: Option<u16>,
) -> ContainerAsync<GenericImage> {
    let mut container =
        GenericImage::new(format!("services_{service_name}"), String::from("latest"));
    if let Some(exposed_port) = exposed_port {
        container = container.with_exposed_port(ContainerPort::Tcp(exposed_port));
    }
    let mut container_request = container
        .with_wait_for(WaitFor::message_on_stdout("listening on"))
        .with_container_name(format!("{service_name}-integration-test"))
        .with_network("shared_network")
        .with_env_var("APP_ENV", "integration-test")
        .with_env_var("PG_PORT", pg_port)
        .with_env_var("PG_HOST", pg_host)
        .with_env_var("PG_USER", "postgres")
        .with_env_var("PG_PASSWORD", "postgres")
        .with_env_var("PG_DBNAME", format!("{service_name}_db"));

    for (name, value) in env_vars {
        container_request = container_request.with_env_var(name, value);
    }

    let container = container_request
        .start()
        .await
        .expect(&format!("failed to start {service_name} service"));

    // read_startup_logs(&container, service_name).await;

    container
}

#[allow(dead_code)]
async fn read_startup_logs(container: &ContainerAsync<GenericImage>, service_name: &str) {
    let mut stdout = BufReader::new(container.stdout(true)).lines();
    let mut stderr = BufReader::new(container.stderr(true)).lines();

    // Read logs for up to 5 seconds
    let _ = timeout(Duration::from_secs(5), async {
        loop {
            tokio::select! {
                Ok(Some(line)) = stdout.next_line() => {
                    println!("[{service_name}] STDOUT: {line}");
                }
                Ok(Some(line)) = stderr.next_line() => {
                    println!("[{service_name}] STDERR: {line}");
                }
                else => break,
            }
        }
    })
    .await;
}
