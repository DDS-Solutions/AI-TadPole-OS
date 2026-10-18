//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Startup / Tracing & Telemetry
//! - **Primary Entrypoints**: `init_tracing`, `RedactingSpanExporter`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initializes the global telemetry and tracing ecosystem.
///
/// ### Log Format
/// - Default: human-readable console output via `tracing_subscriber::fmt`
/// - JSON: Set `OTEL_STDOUT_EXPORTER=json` for machine-parseable newline-delimited JSON
/// - OTel OTLP: Set `OTEL_EXPORTER_OTLP_ENDPOINT` for Jaeger/Tempo export
/// - OTel Stdout: Set `OTEL_STDOUT_EXPORTER=true` for OTel stdout span export
pub fn init_tracing(disable_otel: bool) {
    crate::telemetry::init_prometheus_metrics();

    let otel_env = std::env::var("OTEL_STDOUT_EXPORTER")
        .unwrap_or_default()
        .to_lowercase();
    let use_json_logs = otel_env == "json";
    let enable_stdout_otel = otel_env == "true";

    // Build the OTel provider (type-erased via Option so it can be moved
    // into exactly one match arm without the compiler needing to unify
    // the different Layered<fmt::Layer<_, JsonFields, ...>> vs
    // Layered<fmt::Layer<_, DefaultFields, ...>> subscriber types).
    let otel_provider: Option<SdkTracerProvider> = if !disable_otel {
        let provider = if enable_stdout_otel {
            SdkTracerProvider::builder()
                .with_simple_exporter(RedactingSpanExporter::new(
                    opentelemetry_stdout::SpanExporter::default(),
                ))
                .build()
        } else {
            use opentelemetry_otlp::WithExportConfig;
            let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4317".to_string());
            match opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint.clone())
                .build()
            {
                Ok(exp) => SdkTracerProvider::builder()
                    .with_batch_exporter(RedactingSpanExporter::new(exp))
                    .build(),
                Err(e) => {
                    eprintln!(
                        "⚠️  [Telemetry] OTLP failed ({}): {}. Using stdout.",
                        endpoint, e
                    );
                    SdkTracerProvider::builder()
                        .with_simple_exporter(RedactingSpanExporter::new(
                            opentelemetry_stdout::SpanExporter::default(),
                        ))
                        .build()
                }
            }
        };
        Some(provider)
    } else {
        None
    };

    // Each arm creates its own otel_layer so Rust never needs to unify
    // the concrete Layered<fmt::Layer<_, JsonFields, …>> vs DefaultFields types.
    match (use_json_logs, otel_provider) {
        (true, Some(provider)) => {
            let otel = tracing_opentelemetry::layer().with_tracer(provider.tracer("tadpole-os"));
            let _ = tracing_subscriber::registry()
                .with(tracing_subscriber::EnvFilter::from_default_env())
                .with(tracing_subscriber::fmt::layer().json())
                .with(otel)
                .with(crate::telemetry::TelemetryLayer::new())
                .try_init();
        }
        (false, Some(provider)) => {
            let otel = tracing_opentelemetry::layer().with_tracer(provider.tracer("tadpole-os"));
            let _ = tracing_subscriber::registry()
                .with(tracing_subscriber::EnvFilter::from_default_env())
                .with(tracing_subscriber::fmt::layer())
                .with(otel)
                .with(crate::telemetry::TelemetryLayer::new())
                .try_init();
        }
        (true, None) => {
            let _ = tracing_subscriber::registry()
                .with(tracing_subscriber::EnvFilter::from_default_env())
                .with(tracing_subscriber::fmt::layer().json())
                .with(crate::telemetry::TelemetryLayer::new())
                .try_init();
        }
        (false, None) => {
            let _ = tracing_subscriber::registry()
                .with(tracing_subscriber::EnvFilter::from_default_env())
                .with(tracing_subscriber::fmt::layer())
                .with(crate::telemetry::TelemetryLayer::new())
                .try_init();
        }
    }
}

/// A wrapper for OpenTelemetry SpanExporters that redacts sensitive information.
#[derive(Debug)]
pub struct RedactingSpanExporter<E> {
    inner: E,
    redactor: crate::secret_redactor::SecretRedactor,
}

impl<E> RedactingSpanExporter<E> {
    pub fn new(inner: E) -> Self {
        Self {
            inner,
            redactor: crate::secret_redactor::SecretRedactor::from_env(),
        }
    }
}

impl<E: opentelemetry_sdk::trace::SpanExporter> opentelemetry_sdk::trace::SpanExporter
    for RedactingSpanExporter<E>
{
    fn export(
        &self,
        mut batch: Vec<opentelemetry_sdk::trace::SpanData>,
    ) -> impl std::future::Future<Output = opentelemetry_sdk::error::OTelSdkResult> + Send {
        for span in &mut batch {
            // 1. Redact span name
            span.name = std::borrow::Cow::Owned(self.redactor.redact(&span.name));

            // 2. Redact span attributes (e.g. db.statement, http.request.body, etc.)
            for kv in &mut span.attributes {
                if let opentelemetry::Value::String(ref s) = kv.value {
                    let redacted = self.redactor.redact(s.as_str());
                    kv.value = opentelemetry::Value::String(redacted.into());
                }
            }

            // 3. Redact events (accessing internal Vec of Event)
            for event in &mut span.events.events {
                event.name = std::borrow::Cow::Owned(self.redactor.redact(&event.name));
                for kv in &mut event.attributes {
                    if let opentelemetry::Value::String(ref s) = kv.value {
                        let redacted = self.redactor.redact(s.as_str());
                        kv.value = opentelemetry::Value::String(redacted.into());
                    }
                }
            }
        }
        self.inner.export(batch)
    }

    fn shutdown(&self) -> Result<(), opentelemetry_sdk::error::OTelSdkError> {
        self.inner.shutdown()
    }

    fn force_flush(&self) -> Result<(), opentelemetry_sdk::error::OTelSdkError> {
        self.inner.force_flush()
    }
}
