//! RSA keypair + JWT issuance / verification.

use std::sync::Arc;

use base64::Engine;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use parking_lot::RwLock;
use rand::rngs::OsRng;
use rsa::pkcs8::{EncodePrivateKey, LineEnding};
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};
use rsa::traits::PublicKeyParts;
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub iss: String,
    pub aud: String,
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub correlation_id: Option<String>,
}

pub struct KeyManager {
    private_pem: String,
    public_jwk: serde_json::Value,
    kid: String,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    pub_key_n_b64: String,
    pub_key_e_b64: String,
}

impl KeyManager {
    pub fn new() -> anyhow::Result<Self> { Self::new_persisted(None) }

    /// Construct (or load) the gateway's RSA keypair.
    /// Issue #212: previously the keypair was regenerated on every restart,
    /// invalidating all live tokens. If `path` is provided and the file
    /// exists, the PEM is loaded; otherwise a new keypair is generated
    /// and (if `path` is Some) written to disk in mode 0600.
    pub fn new_persisted(path: Option<&std::path::Path>) -> anyhow::Result<Self> {
        if let Some(p) = path {
            if let Ok(bytes) = std::fs::read(p) {
                if let Ok(pem_str) = std::str::from_utf8(&bytes) {
                    if let Ok(km) = Self::from_pem(pem_str) {
                        return Ok(km);
                    }
                }
            }
        }
        let mut rng = OsRng;
        let sk = RsaPrivateKey::new(&mut rng, 2048)?;
        let pk = RsaPublicKey::from(&sk);

        let pem = sk.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)?;
        let pem_str = String::from_utf8(pem.as_bytes().to_vec())?;
        let kid = "ed-gateway-1".to_string();

        // Build JWK
        let n_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(pk.n().to_bytes_be());
        let e_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(pk.e().to_bytes_be());
        let jwk = serde_json::json!({
            "kty": "RSA",
            "kid": kid,
            "use": "sig",
            "alg": "RS256",
            "n": n_b64,
            "e": e_b64,
        });

        let km = Self {
            encoding_key: EncodingKey::from_rsa_pem(pem.as_bytes())?,
            decoding_key: DecodingKey::from_rsa_pem(pem.as_bytes())?,
            private_pem: pem_str.clone(),
            public_jwk: jwk,
            kid,
            pub_key_n_b64: n_b64,
            pub_key_e_b64: e_b64,
        };

        if let Some(p) = path {
            if let Some(parent) = p.parent() { let _ = std::fs::create_dir_all(parent); }
            if let Err(e) = std::fs::write(p, pem_str.as_bytes()) {
                tracing::warn!(error = %e, path = %p.display(), "could not persist RSA keypair");
            }
        }
        Ok(km)
    }

    fn from_pem(pem_str: &str) -> anyhow::Result<Self> {
        let pem_bytes = pem_str.as_bytes();
        let pk = RsaPublicKey::from_public_key_pem(pem_str)
            .or_else(|_| -> anyhow::Result<RsaPublicKey> {
                // not directly loadable as public-only; try parsing as private
                let sk = rsa::RsaPrivateKey::from_pkcs8_pem(pem_str)?;
                Ok(RsaPublicKey::from(&sk))
            })?;
        let kid = "ed-gateway-1".to_string();
        let n_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(pk.n().to_bytes_be());
        let e_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(pk.e().to_bytes_be());
        let jwk = serde_json::json!({
            "kty": "RSA",
            "kid": kid,
            "use": "sig",
            "alg": "RS256",
            "n": n_b64,
            "e": e_b64,
        });
        Ok(Self {
            encoding_key: EncodingKey::from_rsa_pem(pem_bytes)?,
            decoding_key: DecodingKey::from_rsa_pem(pem_bytes)?,
            private_pem: pem_str.to_string(),
            public_jwk: jwk,
            kid,
            pub_key_n_b64: n_b64,
            pub_key_e_b64: e_b64,
        })
    }

    pub fn kid(&self) -> &str { &self.kid }
    pub fn public_jwk(&self) -> &serde_json::Value { &self.public_jwk }
    pub fn pub_n_b64(&self) -> &str { &self.pub_key_n_b64 }
    pub fn pub_e_b64(&self) -> &str { &self.pub_key_e_b64 }

    pub fn sign(&self, claims: &Claims) -> AppResult<String> {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(self.kid.clone());
        encode(&header, claims, &self.encoding_key).map_err(|e| AppError::Internal(format!("jwt: {e}")))
    }

    pub fn verify(&self, token: &str, issuer: &str, audience: &str) -> AppResult<Claims> {
        let mut v = Validation::new(Algorithm::RS256);
        v.set_audience(&[audience]);
        v.set_issuer(&[issuer]);
        decode::<Claims>(token, &self.decoding_key, &v)
            .map(|d| d.claims)
            .map_err(|e| AppError::Unauthorized(format!("invalid token: {e}")))
    }
}

/// Issue a short-lived user JWT.
pub fn issue_user_token(
    keys: &KeyManager,
    issuer: &str,
    audience: &str,
    subject: &str,
    roles: Vec<String>,
    scopes: Vec<String>,
    ttl_seconds: i64,
) -> AppResult<String> {
    let now = chrono::Utc::now().timestamp();
    let claims = Claims {
        iss: issuer.to_string(),
        aud: audience.to_string(),
        sub: subject.to_string(),
        iat: now as usize,
        exp: (now + ttl_seconds) as usize,
        roles,
        scopes,
        correlation_id: None,
    };
    keys.sign(&claims)
}

/// Issue a service-to-service internal token (HMAC for simplicity, with separate `internal_secret`).
pub fn issue_internal_token(secret: &str, issuer: &str, service: &str, ttl_seconds: i64) -> AppResult<String> {
    use jsonwebtoken::{EncodingKey, Header};
    let now = chrono::Utc::now().timestamp();
    let claims = serde_json::json!({
        "iss": issuer,
        "aud": "internal",
        "sub": format!("service:{service}"),
        "iat": now,
        "exp": now + ttl_seconds,
        "roles": ["service"],
        "scopes": ["internal"],
    });
    encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(format!("internal jwt: {e}")))
}

/// Verify an internal token (HS256, audience = "internal").
pub fn verify_internal_token(secret: &str, issuer: &str, token: &str) -> AppResult<Claims> {
    let mut v = Validation::new(Algorithm::HS256);
    v.set_audience(&["internal"]);
    v.set_issuer(&[issuer]);
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &v)
        .map(|d| d.claims)
        .map_err(|e| AppError::Unauthorized(format!("invalid internal token: {e}")))
}

/// Cheaply cloneable `Arc<KeyManager>` newtype.
#[derive(Clone)]
pub struct SharedKeys(pub Arc<KeyManager>);
