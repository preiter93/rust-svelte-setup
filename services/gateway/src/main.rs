mod handler;
mod utils;
use handler::Handler;

use crate::handler::create_session;
use axum::{Router, routing::post};
use std::error::Error;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let handler = Handler::new().await?;
    let app = Router::new()
        .route("/session", post(create_session))
        .with_state(handler)
        .layer(CorsLayer::very_permissive());

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
