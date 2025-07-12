use std::error::Error;

use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::SpanExporter;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::{Resource, propagation::TraceContextPropagator};
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

/// Initializes OpenTelemetry tracing.
///
/// It allows tracing spans to be exported to backends like Jaeger.
pub fn init_tracer(service_name: &'static str) -> Result<SdkTracerProvider, Box<dyn Error>> {
    let span_exporter = SpanExporter::builder().with_tonic().build()?;
    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(Resource::builder().with_service_name(service_name).build())
        .with_batch_exporter(span_exporter)
        .build();

    global::set_text_map_propagator(TraceContextPropagator::new());
    global::set_tracer_provider(tracer_provider.clone());

    let tracer = tracer_provider.tracer(service_name);
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    tracing_subscriber::registry().with(telemetry).init();

    Ok(tracer_provider)
}
