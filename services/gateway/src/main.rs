mod handler;

use crate::handler::get_session;
use axum::{Router, routing::get};
use std::error::Error;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct Server {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = Server {};
    let app = Router::new()
        .route("/session", get(get_session))
        .with_state(server)
        .layer(CorsLayer::very_permissive());

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
