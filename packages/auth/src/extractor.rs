use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{header::AUTHORIZATION, request::Parts},
};
use crate::verifier::JwtVerifier;
use ed_domain::UserId;
use crate::current_user::CurrentUser;

pub struct CurrentUserExtractor(pub CurrentUser);

#[async_trait]
impl<S> FromRequestParts<S> for CurrentUserExtractor
where
    S: Send + Sync,
    JwtVerifier: FromRef<S>,
{
    type Rejection = axum::response::Response;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let verifier = JwtVerifier::from_ref(state);
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or_else(|| {
                axum::response::IntoResponse::into_response((
                    axum::http::StatusCode::UNAUTHORIZED,
                    axum::Json(serde_json::json!({"error": "missing bearer"})),
                ))
            })?;
        let claims = verifier
            .verify(token)
            .map_err(|e| {
                axum::response::IntoResponse::into_response((
                    axum::http::StatusCode::UNAUTHORIZED,
                    axum::Json(serde_json::json!({"error": format!("invalid token: {e}")})),
                ))
            })?;
        let id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| {
            axum::response::IntoResponse::into_response((
                axum::http::StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({"error": "sub is not a UUID"})),
            ))
        })?;
        Ok(CurrentUserExtractor(CurrentUser {
            id: UserId::from(id),
            email: None,
            roles: claims.roles,
            scopes: claims.scopes,
            correlation_id: claims.correlation_id,
        }))
    }
}
