//! Unit tests for the gateway's auth, JWT, and routing logic.

use axum::http::StatusCode;
use gateway::config::Config;
use gateway::error::{AppError, ProblemDetails};
use gateway::security::jwt::{issue_user_token, verify_internal_token, verify_token, KeyManager};

// ─── Config ────────────────────────────────────────────────────────────────
#[test]
fn config_defaults_when_env_unset() {
    // No env vars -- we don't want to fail; this verifies the fallback chain.
    let cfg = Config::from_env();
    assert!(cfg.is_ok());
    let cfg = cfg.unwrap();
    assert!(!cfg.jwt_issuer.is_empty());
    assert!(!cfg.jwt_audience.is_empty());
    assert!(cfg.services.contains_key("room-service"));
    assert!(cfg.services.contains_key("doc-service"));
    assert!(cfg.services.contains_key("latex-service"));
    assert!(cfg.rate_limit.contains_key("/api/v1/room-service"));
}

#[test]
fn config_bind_addr_includes_host_and_port() {
    let cfg = Config::from_env().unwrap();
    let addr = cfg.bind_addr();
    assert!(addr.contains(':'));
}

// ─── KeyManager + JWT round-trip ───────────────────────────────────────────
#[test]
fn keymanager_generates_with_kid() {
    let km = KeyManager::new().expect("KeyManager");
    assert_eq!(km.kid(), "ed-gateway-1");
    let jwk = km.public_jwk();
    assert_eq!(jwk["kty"], "RSA");
    assert_eq!(jwk["alg"], "RS256");
    assert_eq!(jwk["use"], "sig");
    assert!(jwk["n"].as_str().unwrap().len() > 100);
}

#[test]
fn keymanager_generates_unique_keys() {
    let km1 = KeyManager::new().unwrap();
    let km2 = KeyManager::new().unwrap();
    assert_ne!(km1.pub_n_b64(), km2.pub_n_b64());
}

#[test]
fn user_token_round_trip() {
    let km = KeyManager::new().unwrap();
    let token = issue_user_token(&km, "ed-gateway", "ed-services", "user-123",
        vec!["user".into()], vec!["rooms:read".into()], 60).unwrap();
    let claims = verify_token(&token, "ed-gateway", "ed-services").unwrap();
    assert_eq!(claims.sub, "user-123");
    assert_eq!(claims.iss, "ed-gateway");
    assert_eq!(claims.aud, "ed-services");
    assert_eq!(claims.roles, vec!["user".to_string()]);
    assert_eq!(claims.scopes, vec!["rooms:read".to_string()]);
}

#[test]
fn token_rejected_with_wrong_audience() {
    let km = KeyManager::new().unwrap();
    let token = issue_user_token(&km, "ed-gateway", "ed-services", "u",
        vec![], vec![], 60).unwrap();
    let res = verify_token(&token, "ed-gateway", "wrong-audience");
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(matches!(err, AppError::Unauthorized(_)));
}

#[test]
fn token_rejected_with_wrong_issuer() {
    let km = KeyManager::new().unwrap();
    let token = issue_user_token(&km, "ed-gateway", "ed-services", "u",
        vec![], vec![], 60).unwrap();
    let res = verify_token(&token, "wrong-issuer", "ed-services");
    assert!(res.is_err());
}

#[test]
fn token_rejected_when_tampered() {
    let km = KeyManager::new().unwrap();
    let token = issue_user_token(&km, "ed-gateway", "ed-services", "u",
        vec![], vec![], 60).unwrap();
    let mut parts: Vec<&str> = token.split('.').collect();
    let last = parts[2].to_string();
    let replaced = if last.ends_with('A') { format!("{}B", &last[..last.len()-1]) } else { format!("{}A", &last[..last.len()-1]) };
    parts[2] = &replaced;
    let bad = parts.join(".");
    let res = verify_token(&bad, "ed-gateway", "ed-services");
    assert!(res.is_err());
}

#[test]
fn expired_token_rejected() {
    let km = KeyManager::new().unwrap();
    let token = issue_user_token(&km, "ed-gateway", "ed-services", "u",
        vec![], vec![], -10).unwrap();
    let res = verify_token(&token, "ed-gateway", "ed-services");
    assert!(res.is_err());
}

// ─── Internal token (HS256) ───────────────────────────────────────────────
#[test]
fn internal_token_round_trip() {
    let secret = "unit-test-internal-secret";
    let token = gateway::security::jwt::issue_internal_token(secret, "ed-gateway", "room-service", 60).unwrap();
    let claims = verify_internal_token(secret, "ed-gateway", &token).unwrap();
    assert_eq!(claims.sub, "service:room-service");
    assert!(claims.roles.contains(&"service".to_string()));
    assert!(claims.scopes.contains(&"internal".to_string()));
}

#[test]
fn internal_token_rejected_with_wrong_secret() {
    let token = gateway::security::jwt::issue_internal_token("a", "ed-gateway", "x", 60).unwrap();
    let res = verify_internal_token("b", "ed-gateway", &token);
    assert!(res.is_err());
}

// ─── AppError -> ProblemDetails ───────────────────────────────────────────
#[test]
fn app_error_status_mapping() {
    assert_eq!(AppError::NotFound { what: "x".into() }.status(), StatusCode::NOT_FOUND);
    assert_eq!(AppError::BadRequest("x".into()).status(), StatusCode::BAD_REQUEST);
    assert_eq!(AppError::Unauthorized("x".into()).status(), StatusCode::UNAUTHORIZED);
    assert_eq!(AppError::Forbidden("x".into()).status(), StatusCode::FORBIDDEN);
    assert_eq!(AppError::Validation("x".into()).status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(AppError::Conflict("x".into()).status(), StatusCode::CONFLICT);
    assert_eq!(AppError::Upstream("x".into()).status(), StatusCode::BAD_GATEWAY);
    assert_eq!(AppError::Internal("x".into()).status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(AppError::RateLimited { retry_after_secs: 60 }.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[test]
fn app_error_problem_carries_instance() {
    let e = AppError::NotFound { what: "room".into() };
    let p = e.to_problem(Some("/rooms/123".into()));
    assert_eq!(p.status, 404);
    assert_eq!(p.title, "Not found");
    assert_eq!(p.instance.as_deref(), Some("/rooms/123"));
    assert!(p.kind.starts_with("https://"));
}

#[test]
fn app_error_problem_serialises() {
    let p = ProblemDetails {
        kind: "about:blank#404".into(),
        title: "Not found".into(),
        status: 404,
        detail: Some("x".into()),
        instance: None,
    };
    let j = serde_json::to_string(&p).unwrap();
    let back: ProblemDetails = serde_json::from_str(&j).unwrap();
    assert_eq!(back.status, 404);
    assert!(j.contains("\"type\":"));
}
