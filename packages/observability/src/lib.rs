use std::sync::Once;
use tracing_subscriber::{prelude::*, EnvFilter};
static INIT: Once = Once::new();
pub fn init_tracing(service_name: &str, json: bool) {
    INIT.call_once(|| {
        let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,ed_*=debug"));
        let fmt_layer = if json { tracing_subscriber::fmt::layer().json().boxed() } else { tracing_subscriber::fmt::layer().boxed() };
        let filter_svc = EnvFilter::new(format!("info,{}=debug", service_name.replace('-', "_")));
        let svc_layer = tracing_subscriber::fmt::layer().with_filter(filter_svc).boxed();
        tracing_subscriber::registry().with(env_filter).with(fmt_layer).with(svc_layer).init();
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
