use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, pub iss: String, pub aud: String,
    pub exp: usize, pub iat: usize,
    #[serde(default)] pub roles: Vec<String>,
    #[serde(default)] pub scopes: Vec<String>,
    #[serde(default)] pub correlation_id: Option<String>,
}
pub struct JwtVerifier { pub(crate) decoding_key: DecodingKey, pub(crate) issuer: String, pub(crate) audience: String, pub(crate) algorithm: Algorithm }
impl JwtVerifier {
    pub fn new_from_secret(secret: &[u8], issuer: impl Into<String>, audience: impl Into<String>) -> Self {
        Self { decoding_key: DecodingKey::from_secret(secret), issuer: issuer.into(), audience: audience.into(), algorithm: Algorithm::HS256 }
    }
    pub fn new_from_rsa_pem(pem: &[u8], issuer: impl Into<String>, audience: impl Into<String>) -> Self {
        Self { decoding_key: DecodingKey::from_rsa_pem(pem).expect("invalid RSA PEM"), issuer: issuer.into(), audience: audience.into(), algorithm: Algorithm::RS256 }
    }
    pub fn verify(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let mut v = Validation::new(self.algorithm.clone());
        v.set_audience(&[self.audience.clone()]);
        v.set_issuer(&[self.issuer.clone()]);
        decode::<Claims>(token, &self.decoding_key, &v).map(|d| d.claims)
    }
    pub fn roles_unique(&self, c: &Claims) -> HashSet<String> { c.roles.iter().cloned().collect() }
}
