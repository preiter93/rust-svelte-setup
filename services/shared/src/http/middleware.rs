use axum::Router;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::tracing::request_span::{accept_trace, new_request_span, record_trace_id};

pub fn add_middleware(router: Router) -> Router {
    router.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http().make_span_with(new_request_span))
            .map_request(accept_trace)
            .map_request(record_trace_id),
    )
}
