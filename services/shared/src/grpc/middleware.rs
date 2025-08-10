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
use tracing::{Span, field, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

type MiddlewareFn<B> = fn(Request<B>) -> Request<B>;
type MakeSpanFn<B> = fn(&Request<B>) -> Span;

type MiddlewareStack<B> = ServiceBuilder<
    Stack<
        MapRequestLayer<MiddlewareFn<B>>, // record trace id
        Stack<
            MapRequestLayer<MiddlewareFn<B>>, // accept trace
            Stack<
                TraceLayer<GrpcMakeClassifier, MakeSpanFn<B>>, // trace layer
                Identity,
            >,
        >,
    >,
>;

pub fn add_middleware<B, L>(router: Server<L>) -> Server<Stack<MiddlewareStack<B>, L>> {
    let service_builder: MiddlewareStack<B> = ServiceBuilder::new()
        .layer(TraceLayer::new_for_grpc().make_span_with(new_request_span as MakeSpanFn<B>))
        .map_request(accept_trace::<B> as MiddlewareFn<B>)
        .map_request(record_trace_id::<B> as MiddlewareFn<B>);
    router.layer(service_builder)
}

/// Creates a new tracing span for an incoming request.
fn new_request_span<B>(_: &http::Request<B>) -> Span {
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
