use std::{env, error::Error};

#[derive(Debug)]
pub struct PGConfig {
    pub(super) dbname: String,
    pub(super) user: String,
    pub(super) password: String,
    pub(super) host: String,
    pub(super) port: u16,
}

impl PGConfig {
    /// Load PostgreSQL configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if required environment variables are missing
    /// or if `PG_PORT` cannot be parsed.
    pub fn from_env(service_name: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            dbname: format!("{service_name}_db"),
            user: env::var("PG_USER")?,
            password: env::var("PG_PASSWORD")?,
            host: patched_host(env::var("PG_HOST")?),
            port: env::var("PG_PORT")?.parse::<u16>()?,
        })
    }
}

fn patched_host<S: Into<String>>(host: S) -> String {
    let host = host.into();
    let app_env = std::env::var("APP_ENV").unwrap_or_default();
    match app_env.as_str() {
        "local" => "localhost".to_string(),
        "integration-test" => format!("{host}-integration-test"),
        _ => host,
    }
}
