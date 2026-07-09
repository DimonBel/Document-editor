//! Unit tests for `ed-cache`: types and trait surface (no live Redis required).

use ed_cache::{RateLimitDecision, Session};
use uuid::Uuid;

#[test]
fn session_id_is_uuid_string() {
    let id = Session::new_id();
    assert_eq!(id.len(), 36);
    let parsed = Uuid::parse_str(&id).unwrap();
    assert_eq!(parsed.to_string(), id);
}

#[test]
fn session_ids_are_unique() {
    let a = Session::new_id();
    let b = Session::new_id();
    assert_ne!(a, b);
}

#[test]
fn rate_limit_decision_is_copy() {
    // Compile-time check that the enum derives Copy
    let a = RateLimitDecision::Allow;
    let b = a;
    let _ = (a, b);
    assert!(matches!(b, RateLimitDecision::Allow));
}

#[test]
fn cache_error_converts_from_json() {
    use ed_cache::CacheError;
    let bad = serde_json::from_str::<serde_json::Value>("not-json");
    let err: CacheError = bad.unwrap_err().into();
    let s = format!("{err}");
    assert!(s.contains("json"));
}
