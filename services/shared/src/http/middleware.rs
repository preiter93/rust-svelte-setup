use axum::Router;
use opentelemetry::{global, trace::TraceContextExt as _};
use opentelemetry_http::HeaderExtractor;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{Span, field, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

pub fn add_middleware(router: Router) -> Router {
    router.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http().make_span_with(new_request_span))
            .map_request(accept_trace)
            .map_request(record_trace_id),
    )
}

/// Creates a new tracing span for an incoming request.
fn new_request_span<B>(_: &http::Request<B>) -> Span {
    // let headers = request.headers();
    // info_span!("request", ?headers, trace_id = field::Empty)
    info_span!("request", trace_id = field::Empty)
}

/// Propagates trace context between service boundaries.
///
/// Associate the current span with the open telemetry trace of the given request.
fn accept_trace<B>(request: http::Request<B>) -> http::Request<B> {
    let parent_context = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(request.headers()))
    });
    Span::current().set_parent(parent_context);

    request
}

/// Records the open telemetry trace ID of the given request as "trace_id" in the current span.
fn record_trace_id<B>(request: http::Request<B>) -> http::Request<B> {
    let span = Span::current();

    let trace_id = span.context().span().span_context().trace_id();
    span.record("trace_id", trace_id.to_string());

    request
}
