//! Authentication state: user credentials (argon2id-hashed passwords) +
//! refresh-token tracking backed by Redis.
//!
//! In a production deployment this would live in a Postgres `users` table,
//! but the same trait surface (`UserStore`) is preserved.

use std::collections::HashMap;
use std::sync::Arc;

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use parking_lot::RwLock;
use redis::AsyncCommands;
use subtle::ConstantTimeEq;

use crate::error::AppError;

/// A user record with an Argon2id-hashed password.
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub scopes: Vec<String>,
    pub password_hash: String,
}

#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, AppError>;
}

/// In-memory user store seeded from config.
///
/// The default user is created at startup if `SEED_USERNAME` and
/// `SEED_PASSWORD_HASH` (a pre-hashed Argon2id string) are set, OR if
/// `SEED_USERNAME` + `SEED_PASSWORD` are set (the latter hashes at
/// startup and prints the hash for the operator to persist).
pub struct InMemoryUserStore {
    by_username: Arc<RwLock<HashMap<String, User>>>,
    by_id: Arc<RwLock<HashMap<String, User>>>,
}

impl InMemoryUserStore {
    pub fn new() -> Self {
        Self {
            by_username: Arc::new(RwLock::new(HashMap::new())),
            by_id: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub fn insert(&self, user: User) {
        self.by_id.write().insert(user.id.clone(), user.clone());
        self.by_username.write().insert(user.username.clone(), user);
    }
}

#[async_trait::async_trait]
impl UserStore for InMemoryUserStore {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        Ok(self.by_username.read().get(username).cloned())
    }
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, AppError> {
        // Issue #203: previously returned the first user regardless of id.
        Ok(self.by_id.read().get(id).cloned())
    }
}

/// Verify a plaintext password against an Argon2id hash.
pub fn verify_password(password: &str, hash: &str) -> Result<(), AppError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("password hash parse: {e}")))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| AppError::Unauthorized("invalid credentials".into()))
}

/// Hash a plaintext password with a random salt.
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(format!("hash: {e}")))
}

/// Constant-time string comparison.
pub fn ct_eq(a: &str, b: &str) -> bool {
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

/// Refresh-token store backed by Redis.
///
/// Each refresh token is a random 32-byte URL-safe string; the corresponding
/// `jti` key in Redis carries the user-id and is removed on use (rotation).
pub struct RefreshTokenStore {
    pub redis: deadpool_redis::Pool,
}

impl RefreshTokenStore {
    pub async fn issue(&self, user_id: &str) -> Result<String, AppError> {
        let token = random_token();
        let mut conn = self.redis.get().await
            .map_err(|e| AppError::Internal(format!("redis: {e}")))?;
        // Issue #251: never use the raw token as the Redis key. A
        // database dump leaks all active sessions; hash with SHA-256
        // and use the prefix `refresh:` + hash as the lookup key.
        let key = format!("refresh:{}", sha256_hex(&token));
        let _: () = conn.set_ex(&key, user_id, 60 * 60 * 24 * 30).await
            .map_err(|e| AppError::Internal(format!("redis: {e}")))?;
        Ok(token)
    }
    pub async fn consume(&self, token: &str) -> Result<Option<String>, AppError> {
        let mut conn = self.redis.get().await
            .map_err(|e| AppError::Internal(format!("redis: {e}")))?;
        let key = format!("refresh:{}", sha256_hex(token));
        let user_id: Option<String> = redis::cmd("GETDEL")
            .arg(&key)
            .query_async(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("redis: {e}")))?;
        Ok(user_id)
    }
}

fn sha256_hex(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex::encode(h.finalize())
}

fn random_token() -> String {
    use rand::RngCore;
    use base64::Engine;
    let mut buf = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut buf);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf)
}
