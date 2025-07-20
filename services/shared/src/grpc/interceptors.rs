use opentelemetry::global;
use opentelemetry::propagation::Injector;
use tonic::{
    Request, Status,
    metadata::{MetadataKey, MetadataMap, MetadataValue},
    service::Interceptor,
};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

#[derive(Clone)]
pub struct GrpcServiceInterceptor;

impl Interceptor for GrpcServiceInterceptor {
    /// Propagate the current trace context by injecting it into the request's metadata.
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        global::get_text_map_propagator(|propagator| {
            let context = Span::current().context();
            propagator.inject_context(&context, &mut MetadataInjector(request.metadata_mut()));
        });

        Ok(request)
    }
}

/// An adapter that injects OpenTelemetry trace context into gRPC metadata.
struct MetadataInjector<'a>(&'a mut MetadataMap);

impl<'a> Injector for MetadataInjector<'a> {
    /// Attempts to insert opentelemetry trace context into the gRPC requests metadata.
    fn set(&mut self, key: &str, value: String) {
        if let (Ok(key), Ok(value)) = (
            MetadataKey::from_bytes(key.as_bytes()),
            MetadataValue::try_from(value.as_str()),
        ) {
            self.0.insert(key, value);
        }
    }
}
