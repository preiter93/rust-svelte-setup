pub mod cookie;
pub mod db;
pub mod middleware;
pub mod session;
#[cfg(feature = "test-utils")]
pub mod test_utils;
pub mod tracing;

pub fn patched_host<S: Into<String>>(host: S) -> String {
    let host = host.into();
    let app_env = std::env::var("APP_ENV").unwrap_or_default();
    match app_env.as_str() {
        "local" => "localhost".to_string(),
        "integration-test" => format!("{host}-integration-test"),
        _ => host,
    }
}
