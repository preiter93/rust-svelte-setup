mod handler;
mod utils;
use shared::{http::middleware::add_middleware, tracing::tracer::init_tracer};
use handler::Handler;

use crate::handler::{create_session, create_user, get_current_user, get_user_id_by_google_id};
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

    // TODO add middleware to validate token

    let handler = Handler::new().await?;
    let mut router = Router::new()
        .route("/session", post(create_session))
        .route("/user", post(create_user))
        .route("/user/me", get(get_current_user))
        .route("/user/google/{id}", get(get_user_id_by_google_id))
        .with_state(handler)
        .layer(CorsLayer::very_permissive());
    router = add_middleware(router);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, router).await.unwrap();

    tracer.shutdown()?;

    Ok(())
}
