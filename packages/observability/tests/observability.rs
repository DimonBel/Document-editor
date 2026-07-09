//! Unit tests for `ed-observability`: tracing init smoke + correlation ID helpers.

use ed_observability::correlation;

#[test]
fn correlation_id_is_a_uuid_string() {
    let id = correlation::new();
    // 8-4-4-4-12 = 36 chars (UUID v4 canonical form)
    assert_eq!(id.len(), 36);
    assert_eq!(id.chars().filter(|c| *c == '-').count(), 4);
}

#[test]
fn correlation_ids_are_unique() {
    let a = correlation::new();
    let b = correlation::new();
    assert_ne!(a, b);
}

#[test]
fn correlation_from_headers_picks_first_nonempty() {
    use http::HeaderValue;
    let headers = vec![HeaderValue::from_static(""), HeaderValue::from_static("trace-1")];
    assert_eq!(correlation::from_headers(headers.iter()), Some("trace-1".to_string()));
}

#[test]
fn correlation_from_headers_returns_none_when_empty() {
    use http::HeaderValue;
    let headers: Vec<HeaderValue> = vec![];
    assert_eq!(correlation::from_headers(headers.iter()), None);
}

#[test]
fn correlation_header_name_is_correct() {
    assert_eq!(correlation::CORRELATION_HEADER.as_str(), "x-correlation-id");
}

// init_tracing uses a Once -- calling twice must be safe and idempotent
#[test]
fn init_tracing_is_idempotent() {
    ed_observability::init_tracing("test-svc", true);
    ed_observability::init_tracing("test-svc", true);  // no panic
}
