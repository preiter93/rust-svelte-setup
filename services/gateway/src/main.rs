mod error;
mod handler;
mod utils;

use crate::handler::{
    Handler, get_current_user, handle_oauth_callback, logout_user, start_oauth_login,
};
use auth::AuthClient;
use axum::{
    Router,
    http::{
        HeaderValue, Method,
        header::{AUTHORIZATION, CONTENT_TYPE},
    },
    routing::{get, post},
};
use gateway::{HTTP_PORT, SERVICE_NAME};
use shared::middleware::{TracingHttpServiceLayer, auth::SessionAuthLayer};
use shared::tracing::init_tracer;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tracer = init_tracer(SERVICE_NAME)?;

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(vec![AUTHORIZATION, CONTENT_TYPE]);

    let auth_client = AuthClient::new().await?;

    let handler = Handler::new().await?;
    let mut router = Router::new()
        .route("/logout", post(logout_user))
        .route("/user/me", get(get_current_user))
        .route("/auth/{provider}/login", get(start_oauth_login))
        .route("/auth/{provider}/callback", get(handle_oauth_callback))
        .with_state(handler);
    router = router.layer(SessionAuthLayer::new(
        auth_client.clone(),
        vec![
            String::from("/auth/google/login"),
            String::from("/auth/google/callback"),
            String::from("/auth/github/login"),
            String::from("/auth/github/callback"),
        ],
    ));
    router = router.layer(cors).layer(TracingHttpServiceLayer);

    let address = format!("0.0.0.0:{HTTP_PORT}");

    let listener = TcpListener::bind(address).await?;
    println!("listening on :{}", listener.local_addr()?);

    axum::serve(listener, router).await?;

    tracer.shutdown()?;

    Ok(())
}
