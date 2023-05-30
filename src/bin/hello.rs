use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use opentelemetry::{global, sdk::trace as sdktrace, trace::TraceError};
use opentelemetry_aws::XrayPropagator;
use opentelemetry_http::HeaderExtractor;
use std::{thread, time::Duration};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let parent_context = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(event.headers()))
    });

    let span = tracing::info_span!("nested");
    span.set_parent(parent_context);

    span.in_scope(|| {
        thread::sleep(Duration::from_millis(50));

        let who = event
            .query_string_parameters_ref()
            .and_then(|params| params.first("name"))
            .unwrap_or("world");
        let message = format!("Hello {who}, this is an AWS Lambda HTTP request");

        let resp = Response::builder()
            .status(200)
            .header("content-type", "text/html")
            .body(message.into())
            .map_err(Box::new)?;

        Ok(resp)
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let tracer = init_tracer()?;
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let format = tracing_subscriber::fmt::format().without_time();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .event_format(format);

    tracing_subscriber::Registry::default()
        .with(tracing_subscriber::EnvFilter::try_new("info")?)
        .with(fmt_layer)
        .with(telemetry)
        .try_init()?;

    let res = run(service_fn(function_handler)).await;

    global::shutdown_tracer_provider();

    res
}

fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    global::set_text_map_propagator(XrayPropagator::new());

    let otlp_exporter = opentelemetry_otlp::new_exporter().tonic();

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(
            sdktrace::config()
                .with_sampler(sdktrace::Sampler::AlwaysOn)
                .with_id_generator(sdktrace::XrayIdGenerator::default()),
        )
        .with_exporter(otlp_exporter)
        .install_simple()
}
