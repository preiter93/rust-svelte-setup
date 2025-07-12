mod handler;
mod utils;
use common_utils::{http::middleware::add_middleware, tracing::tracer::init_tracer};
use handler::Handler;

use crate::handler::{create_session, list_users};
use axum::{
    Router,
    routing::{get, post},
};
use std::error::Error;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

const SERVICE_NAME: &'static str = "gateway";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let tracer = init_tracer(SERVICE_NAME)?;

    let handler = Handler::new().await?;
    let mut router = Router::new()
        .route("/session", post(create_session))
        .route("/user", get(list_users))
        .with_state(handler)
        .layer(CorsLayer::very_permissive());
    router = add_middleware(router);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, router).await.unwrap();

    tracer.shutdown()?;

    Ok(())
}
