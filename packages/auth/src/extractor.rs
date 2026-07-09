use axum::{async_trait, extract::FromRequestParts, http::{header::AUTHORIZATION, request::Parts}};
use crate::current_user::CurrentUser;
use crate::error::AuthError;
use crate::verifier::JwtVerifier;
pub struct CurrentUserExtractor(pub CurrentUser);
#[async_trait]
impl<S> FromRequestParts<S> for CurrentUserExtractor
where S: Send + Sync, JwtVerifier: axum::extract::FromRef<S> {
    type Rejection = AuthError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let verifier = JwtVerifier::from_ref(state);
        let token = parts.headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or_else(|| AuthError::InvalidToken("missing bearer".into()))?;
        let claims = verifier.verify(token).map_err(|e| AuthError::InvalidToken(e.to_string()))?;
        let id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken("sub is not a UUID".into()))?;
        Ok(CurrentUserExtractor(CurrentUser {
            id: id.into(), email: None, roles: claims.roles, scopes: claims.scopes, correlation_id: claims.correlation_id,
        }))
    }
}
