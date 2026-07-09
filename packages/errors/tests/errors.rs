//! Unit tests for `ed-errors`: AppError -> ProblemDetails mapping.

use ed_errors::{AppError, ProblemDetails};
use ed_domain::DomainError;

#[test]
fn validation_maps_to_422() {
    let e = AppError::Validation("nope".into());
    assert_eq!(e.http_status(), 422);
    let p = e.to_problem(None);
    assert_eq!(p.status, 422);
    assert!(p.title.contains("Validation"));
}

#[test]
fn not_found_maps_to_404() {
    let e = AppError::NotFound;
    assert_eq!(e.http_status(), 404);
    let p = e.to_problem(Some("/rooms/123".into()));
    assert_eq!(p.status, 404);
    assert_eq!(p.instance.as_deref(), Some("/rooms/123"));
}

#[test]
fn infra_maps_to_502() {
    let e = AppError::Infra("db down".into());
    assert_eq!(e.http_status(), 502);
    let p = e.to_problem(None);
    assert!(p.detail.unwrap().contains("db down"));
}

#[test]
fn broker_maps_to_502() {
    let e = AppError::Broker("rabbit down".into());
    assert_eq!(e.http_status(), 502);
}

#[test]
fn internal_maps_to_500() {
    let e = AppError::Internal("boom".into());
    assert_eq!(e.http_status(), 500);
    let p = e.to_problem(None);
    assert!(p.title.contains("Internal"));
}

#[test]
fn domain_error_propagates_status() {
    let cases: &[(DomainError, u16)] = &[
        (DomainError::NotFound { entity: "x".into(), id: "1".into() }, 404),
        (DomainError::Validation("x".into()), 422),
        (DomainError::Conflict("x".into()), 409),
        (DomainError::Unauthorized("x".into()), 401),
        (DomainError::Forbidden("x".into()), 403),
        (DomainError::Invariant("x".into()), 400),
    ];
    for (d, expected) in cases {
        let e: AppError = d.clone().into();
        assert_eq!(e.http_status(), *expected, "for {d:?}");
    }
}

#[test]
fn problem_details_serialises_to_json() {
    let p = ProblemDetails {
        kind: "about:blank#404".into(),
        title: "Not found".into(),
        status: 404,
        detail: Some("room 1 missing".into()),
        instance: Some("/rooms/1".into()),
    };
    let j = serde_json::to_string(&p).unwrap();
    let back: ProblemDetails = serde_json::from_str(&j).unwrap();
    assert_eq!(back.status, 404);
    assert_eq!(back.title, "Not found");
    assert_eq!(back.detail.as_deref(), Some("room 1 missing"));
}

#[test]
fn problem_details_uses_type_field_not_r#type() {
    // serde(rename = "type") on a Rust field named `kind` -- ensure the wire key is "type"
    let p = ProblemDetails { kind: "about:blank#400".into(), title: "t".into(), status: 400, detail: None, instance: None };
    let j = serde_json::to_string(&p).unwrap();
    assert!(j.contains("\"type\":"));
    assert!(!j.contains("\"kind\":"));
}
