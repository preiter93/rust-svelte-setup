use http::{Request, Response};
use opentelemetry::{global, trace::TraceContextExt as _};
use opentelemetry_http::{HeaderExtractor, HeaderInjector};
use std::task::{Context, Poll};
use tower::{Layer, Service, ServiceBuilder};
use tower_http::classify::{GrpcErrorsAsFailures, ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{Trace, TraceLayer};
use tracing::{Span, field, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

type GrpcTraceService<S> =
    Trace<TracePropagationService<S>, SharedClassifier<GrpcErrorsAsFailures>, MakeSpan>;

type HttpTraceService<S> =
    Trace<TracePropagationService<S>, SharedClassifier<ServerErrorsAsFailures>, MakeSpan>;

// A gRPC tracing layer. Extracts trace context and starts a span per request.
#[derive(Clone)]
pub struct TracingGrpcServiceLayer;

impl<S> Layer<S> for TracingGrpcServiceLayer {
    type Service = GrpcTraceService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_grpc().make_span_with(MakeSpan)) // creates request span
            .layer(TracePropagationLayer::new()) // extracts trace context and sets trace id
            .service(inner)
    }
}

// A HTTP tracing layer. Extracts trace context and starts a span per request.
#[derive(Clone)]
pub struct TracingHttpServiceLayer;

impl<S> Layer<S> for TracingHttpServiceLayer {
    type Service = HttpTraceService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http().make_span_with(MakeSpan)) // creates request span
            .layer(TracePropagationLayer::new()) // extracts trace context and sets trace id
            .service(inner)
    }
}

/// Layer that propagates trace context from incoming requests to downstream services.
#[derive(Clone)]
pub struct TracePropagationLayer;

impl TracePropagationLayer {
    fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for TracePropagationLayer {
    type Service = TracePropagationService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TracePropagationService { inner }
    }
}

/// Service that propagates trace context from incoming requests to downstream services
#[derive(Clone)]
pub struct TracePropagationService<S> {
    inner: S,
}

impl<S, ReqBody, RespBody> Service<Request<ReqBody>> for TracePropagationService<S>
where
    S: Service<Request<ReqBody>, Response = Response<RespBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // Extract the incoming trace context from the request headers
        let parent_context = global::get_text_map_propagator(|propagator| {
            propagator.extract(&HeaderExtractor(req.headers()))
        });

        // Make the current span a child of the extracted context
        let span = Span::current();
        span.set_parent(parent_context);

        // Sets the trace ID in the current span
        let trace_id = span.context().span().span_context().trace_id();
        span.record("trace_id", trace_id.to_string());

        self.inner.call(req)
    }
}

/// The way [`Span`]s will be created for [`Trace`].
#[derive(Debug, Clone)]
pub struct MakeSpan;

impl<B> tower_http::trace::MakeSpan<B> for MakeSpan {
    /// Creates a new tracing span for an incoming request.
    fn make_span(&mut self, req: &Request<B>) -> Span {
        info_span!("request", uri = %req.uri(), trace_id = field::Empty)
    }
}

/// A client side interceptor that injects the current trace context into outgoing HTTP requests.
#[derive(Clone, Copy)]
pub struct TracingServiceClient<S> {
    inner: S,
}

impl<S> TracingServiceClient<S> {
    /// Creates a new [`TracingServiceClient`].
    pub fn new(service: S) -> Self {
        Self { inner: service }
    }
}

impl<S, ReqBody> Service<Request<ReqBody>> for TracingServiceClient<S>
where
    S: Service<Request<ReqBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: http::Request<ReqBody>) -> Self::Future {
        global::get_text_map_propagator(|propagator| {
            let context = Span::current().context();
            propagator.inject_context(&context, &mut HeaderInjector(req.headers_mut()));
        });

        self.inner.call(req)
    }
}
