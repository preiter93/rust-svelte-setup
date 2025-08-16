use axum::Router;
use http::Request;
use opentelemetry::propagation::Injector;
use opentelemetry::{global, trace::TraceContextExt as _};
use opentelemetry_http::HeaderExtractor;
use tonic::{
    Status,
    metadata::{MetadataKey, MetadataMap, MetadataValue},
    service::Interceptor,
    transport::Server,
};
use tower::{
    ServiceBuilder,
    layer::util::{Identity, Stack},
    util::MapRequestLayer,
};
use tower_http::trace::{GrpcMakeClassifier, TraceLayer};
use tracing::{Span, field, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

/// Adds default tracing middleware to a grpc server.
pub fn add_tracing_middleware_for_grpc<B, L>(
    server: Server<L>,
) -> Server<Stack<MiddlewareStack<B>, L>> {
    let service_builder = ServiceBuilder::new()
        .layer(TraceLayer::new_for_grpc().make_span_with(MakeSpan))
        .map_request(propagate_trace_context::<B> as MiddlewareFn<B>)
        .map_request(record_trace_id::<B> as MiddlewareFn<B>);
    server.layer(service_builder)
}

/// Adds default tracing middleware to a http router.
pub fn add_tracing_middleware_for_http(router: Router) -> Router {
    router.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http().make_span_with(MakeSpan))
            .map_request(propagate_trace_context)
            .map_request(record_trace_id),
    )
}

/// Propagates trace context between service boundaries.
///
/// Associate the current span with the trace of the request.
fn propagate_trace_context<B>(request: http::Request<B>) -> http::Request<B> {
    let parent_context = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(request.headers()))
    });
    Span::current().set_parent(parent_context);

    request
}

/// Records the trace ID of the given request as `trace_id` in the current span.
///
/// This should be applied after [`propagate_trace_context`].
fn record_trace_id<B>(request: http::Request<B>) -> http::Request<B> {
    let span = Span::current();

    let trace_id = span.context().span().span_context().trace_id();
    span.record("trace_id", trace_id.to_string());

    request
}

/// The way [`Span`]s will be created for [`Trace`].
#[derive(Debug, Clone)]
pub struct MakeSpan;

impl<B> tower_http::trace::MakeSpan<B> for MakeSpan {
    /// Creates a new tracing span for an incoming request.
    fn make_span(&mut self, _: &Request<B>) -> Span {
        info_span!("request", trace_id = field::Empty)
    }
}

/// A gRPC interceptor that injects the current trace context into request metadata.
#[derive(Clone)]
pub struct GrpcServiceInterceptor;

impl Interceptor for GrpcServiceInterceptor {
    /// Injects the current span's context into the request's metadata for trace propagation.
    fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        global::get_text_map_propagator(|propagator| {
            let context = Span::current().context();
            propagator.inject_context(&context, &mut MetadataInjector(request.metadata_mut()));
        });

        Ok(request)
    }
}

/// Adapter for injecting trace context into gRPC metadata.
struct MetadataInjector<'a>(&'a mut MetadataMap);

impl<'a> Injector for MetadataInjector<'a> {
    /// Inserts a key-value pair into the gRPC metadata.
    fn set(&mut self, key: &str, value: String) {
        if let (Ok(key), Ok(value)) = (
            MetadataKey::from_bytes(key.as_bytes()),
            MetadataValue::try_from(value.as_str()),
        ) {
            self.0.insert(key, value);
        }
    }
}

type MiddlewareFn<B> = fn(Request<B>) -> Request<B>;

/// This type abomination is necessary to type the response of
/// [`add_tracing_middleware_for_grpc`] correctly.
type MiddlewareStack<B> = ServiceBuilder<
    Stack<
        MapRequestLayer<MiddlewareFn<B>>, // record trace id
        Stack<
            MapRequestLayer<MiddlewareFn<B>>, // accept trace
            Stack<
                TraceLayer<GrpcMakeClassifier, MakeSpan>, // trace layer
                Identity,
            >,
        >,
    >,
>;
