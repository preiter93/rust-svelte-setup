use std::{convert::Infallible, net::SocketAddr};

use axum::response::IntoResponse;
use http::Request;
use tonic::{body::Body, server::NamedService, transport::Server};
use tower::{Service, ServiceBuilder};
use tower_http::trace::TraceLayer;

use crate::tracing::request_span::{accept_trace, new_request_span, record_trace_id};

pub async fn serve<S>(svc: S, addr: SocketAddr) -> Result<(), tonic::transport::Error>
where
    S: Service<Request<Body>, Error = Infallible> + NamedService + Clone + Send + Sync + 'static,
    S::Response: IntoResponse,
    S::Future: Send + 'static,
{
    Server::builder()
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_grpc().make_span_with(new_request_span))
                .map_request(accept_trace)
                .map_request(record_trace_id),
        )
        .add_service(svc)
        .serve(addr)
        .await
}
