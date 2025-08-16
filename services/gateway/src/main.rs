mod error;
mod handler;
mod service;
mod utils;
use auth::AuthClient;
use handler::Handler;
use shared::{
    middleware::AuthMiddleware, middleware::add_tracing_middleware_for_http,
    tracing::tracer::init_tracer,
};
use tower::ServiceBuilder;

use crate::handler::{get_current_user, handle_google_callback, logout_user, start_google_login};
use axum::{
    Router,
    http::{
        HeaderValue, Method,
        header::{AUTHORIZATION, CONTENT_TYPE},
    },
    routing::{get, post},
};
use std::error::Error;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

const SERVICE_NAME: &'static str = "gateway";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let tracer = init_tracer(SERVICE_NAME)?;

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(vec![AUTHORIZATION, CONTENT_TYPE]);

    let auth_client = AuthClient::new().await?;
    let auth_layer = move |inner| AuthMiddleware {
        inner,
        session_validator: auth_client.clone(),
    };

    let handler = Handler::new().await?;
    let mut router = Router::new()
        .route("/logout", post(logout_user))
        .route("/user/me", get(get_current_user))
        .route("/auth/google/login", get(start_google_login))
        .route("/auth/google/callback", get(handle_google_callback))
        .with_state(handler)
        .layer(cors);
    router = router.layer(ServiceBuilder::new().layer_fn(auth_layer));
    router = add_tracing_middleware_for_http(router);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, router).await.unwrap();

    tracer.shutdown()?;

    Ok(())
}
