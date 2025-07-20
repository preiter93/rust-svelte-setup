use opentelemetry::{global, trace::TraceContextExt as _};
use opentelemetry_http::HeaderExtractor;
use tracing::{Span, field, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

/// Creates a new tracing span for an incoming request.
pub fn new_request_span<B>(_: &http::Request<B>) -> Span {
    // let headers = request.headers();
    // info_span!("request", ?headers, trace_id = field::Empty)
    info_span!("request", trace_id = field::Empty)
}

/// Propagates trace context between service boundaries.
///
/// Associate the current span with the open telemetry trace of the given request.
pub fn accept_trace<B>(request: http::Request<B>) -> http::Request<B> {
    let parent_context = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(request.headers()))
    });
    Span::current().set_parent(parent_context);

    request
}

/// Records the open telemetry trace ID of the given request as "trace_id" in the current span.
pub fn record_trace_id<B>(request: http::Request<B>) -> http::Request<B> {
    let span = Span::current();

    let trace_id = span.context().span().span_context().trace_id();
    span.record("trace_id", trace_id.to_string());

    request
}
