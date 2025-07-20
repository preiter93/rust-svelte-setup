use http::Request;
use opentelemetry::{global, trace::TraceContextExt as _};
use opentelemetry_http::HeaderExtractor;
use tonic::transport::Server;
use tower::{
    ServiceBuilder,
    layer::util::{Identity, Stack},
    util::MapRequestLayer,
};
use tower_http::trace::{GrpcMakeClassifier, TraceLayer};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

type TracerStack = Stack<TraceLayer<GrpcMakeClassifier>, Identity>;
type MiddlewareFn<B> = fn(Request<B>) -> Request<B>;

type MiddlewareStack<B> = ServiceBuilder<
    Stack<MapRequestLayer<MiddlewareFn<B>>, Stack<MapRequestLayer<MiddlewareFn<B>>, TracerStack>>,
>;

pub fn add_middleware<B, L>(router: Server<L>) -> Server<Stack<MiddlewareStack<B>, L>> {
    let service_builder: MiddlewareStack<B> = ServiceBuilder::new()
        .layer(TraceLayer::new_for_grpc())
        .map_request(accept_trace::<B> as fn(Request<B>) -> Request<B>)
        .map_request(record_trace_id::<B> as fn(Request<B>) -> Request<B>);
    let router = router.layer(service_builder);
    router
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
