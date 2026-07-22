use std::sync::Once;
use tracing_subscriber::{prelude::*, EnvFilter};
static INIT: Once = Once::new();

/// Initialize tracing exactly once per process. Issue #213: the previous
/// implementation stacked two `fmt` layers which caused every log line
/// to be emitted twice (once in plain text, once in JSON). The
/// service-name filter is now merged into the single base filter.
pub fn init_tracing(service_name: &str, json: bool) {
    INIT.call_once(|| {
        let svc = service_name.replace('-', "_");
        let base = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        let with_svc = base.add_directive(format!("{}=debug", svc).parse().unwrap());

        let fmt_layer = if json {
            tracing_subscriber::fmt::layer().json().boxed()
        } else {
            tracing_subscriber::fmt::layer().boxed()
        };

        tracing_subscriber::registry()
            .with(with_svc)
            .with(fmt_layer)
            .init();
    });
}

pub mod correlation {
    use http::HeaderName;
    use uuid::Uuid;
    pub const CORRELATION_HEADER: HeaderName = HeaderName::from_static("x-correlation-id");
    pub type CorrelationId = String;
    pub fn new() -> CorrelationId { Uuid::new_v4().to_string() }
    pub fn from_headers<'a>(headers: impl IntoIterator<Item = &'a http::HeaderValue>) -> Option<CorrelationId> {
        headers.into_iter().filter_map(|v| v.to_str().ok()).find(|s| !s.is_empty()).map(|s| s.to_string())
    }
}