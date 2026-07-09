//! Unit tests for `ed-auth`: JWT verifier + CurrentUser + Scope/Roles.

use ed_auth::{Role, Scope};
use ed_domain::UserId;

#[test]
fn role_as_str_matches() {
    assert_eq!(Role::User.as_str(), "user");
    assert_eq!(Role::Admin.as_str(), "admin");
    assert_eq!(Role::Service.as_str(), "service");
}

#[test]
fn scope_rejects_empty() {
    assert!(Scope::parse("").is_err());
}

#[test]
fn scope_accepts_nonempty() {
    let s = Scope::parse("rooms:read").unwrap();
    assert_eq!(s.as_str(), "rooms:read");
}

#[test]
fn current_user_role_and_scope_checks() {
    let u = ed_auth::CurrentUser {
        id: UserId::new(),
        email: Some("u@example.com".into()),
        roles: vec!["user".into(), "admin".into()],
        scopes: vec!["rooms:read".into(), "rooms:write".into()],
        correlation_id: None,
    };
    assert!(u.has_role("user"));
    assert!(u.has_role("admin"));
    assert!(!u.has_role("service"));
    assert!(u.has_scope("rooms:read"));
    assert!(u.has_scope("rooms:write"));
    assert!(!u.has_scope("documents:read"));
}

// JWT verifier round-trip via HS256
#[test]
fn jwt_verifier_hs256_round_trip() {
    use ed_auth::{Claims, JwtVerifier};
    use jsonwebtoken::{encode, EncodingKey, Header};
    use chrono::{Utc, Duration};

    let secret = b"unit-test-secret";
    let verifier = JwtVerifier::new_from_secret(secret, "ed-gateway", "ed-services");

    let exp = (Utc::now() + Duration::minutes(5)).timestamp() as usize;
    let iat = Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: UserId::new().to_string(),
        iss: "ed-gateway".into(),
        aud: "ed-services".into(),
        exp, iat,
        roles: vec!["user".into()],
        scopes: vec!["rooms:read".into()],
        correlation_id: Some("trace-1".into()),
    };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret)).unwrap();

    let verified = verifier.verify(&token).unwrap();
    assert_eq!(verified.sub, claims.sub);
    assert_eq!(verified.iss, claims.iss);
    assert_eq!(verified.aud, claims.aud);
    assert_eq!(verified.roles, claims.roles);
    assert_eq!(verified.scopes, claims.scopes);
    assert_eq!(verified.correlation_id.as_deref(), Some("trace-1"));
}

#[test]
fn jwt_verifier_rejects_wrong_secret() {
    use ed_auth::{Claims, JwtVerifier};
    use jsonwebtoken::{encode, EncodingKey, Header};
    use chrono::{Utc, Duration};

    let token_secret = b"a";
    let verifier = JwtVerifier::new_from_secret(b"b", "ed-gateway", "ed-services");

    let exp = (Utc::now() + Duration::minutes(5)).timestamp() as usize;
    let claims = Claims {
        sub: "x".into(), iss: "ed-gateway".into(), aud: "ed-services".into(),
        exp, iat: Utc::now().timestamp() as usize,
        roles: vec![], scopes: vec![], correlation_id: None,
    };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(token_secret)).unwrap();

    assert!(verifier.verify(&token).is_err());
}

#[test]
fn jwt_verifier_rejects_expired_token() {
    use ed_auth::{Claims, JwtVerifier};
    use jsonwebtoken::{encode, EncodingKey, Header};
    use chrono::{Utc, Duration};

    let secret = b"s";
    let verifier = JwtVerifier::new_from_secret(secret, "ed-gateway", "ed-services");

    let exp = (Utc::now() - Duration::minutes(5)).timestamp() as usize;
    let claims = Claims {
        sub: "x".into(), iss: "ed-gateway".into(), aud: "ed-services".into(),
        exp, iat: Utc::now().timestamp() as usize,
        roles: vec![], scopes: vec![], correlation_id: None,
    };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret)).unwrap();

    assert!(verifier.verify(&token).is_err());
}

#[test]
fn roles_unique_dedupes() {
    use ed_auth::{Claims, JwtVerifier};
    let v = JwtVerifier::new_from_secret(b"s", "iss", "aud");
    let c = Claims {
        sub: "s".into(), iss: "iss".into(), aud: "aud".into(),
        exp: 0, iat: 0,
        roles: vec!["user".into(), "user".into(), "admin".into()],
        scopes: vec![], correlation_id: None,
    };
    let uniq = v.roles_unique(&c);
    assert_eq!(uniq.len(), 2);
}
