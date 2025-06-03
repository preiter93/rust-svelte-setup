use deadpool_postgres::Pool;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    connect_to_db()?;
    println!("Hello, world!");
    Ok(())
}

pub fn connect_to_db() -> anyhow::Result<deadpool_postgres::Pool> {
    use deadpool_postgres::{Manager, ManagerConfig, RecyclingMethod};
    use tokio_postgres::{Config, NoTls};

    let pg_dbname = std::env::var("PG_DBNAME").expect("PG_DBNAME must be set");
    let pg_password = std::env::var("PG_PASSWORD").expect("PG_PASSWORD must be set");
    let pg_user = std::env::var("PG_USER").expect("PG_USER must be set");
    let pg_host = std::env::var("PG_HOST").expect("PG_HOST must be set");
    let pg_port_str = std::env::var("PG_PORT").expect("PG_PORT must be set");
    let pg_port = pg_port_str.parse::<u16>().expect("failed to parse PG_PORT");

    let mut pg_config = Config::new();
    pg_config
        .dbname(pg_dbname)
        .user(pg_user)
        .password(pg_password)
        .host(pg_host)
        .port(pg_port);
    let manager_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };
    let manager = Manager::from_config(pg_config, NoTls, manager_config);
    let pool = Pool::builder(manager).build()?;
    Ok(pool)
}
