//! Authentication and authorization middleware

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::fmt;

/// Authentication layer for extracting and validating user sessions
pub struct AuthLayer;

impl AuthLayer {
    /// Validate user authentication and extract role
    pub fn validate() -> Self {
        AuthLayer
    }
}

impl fmt::Debug for AuthLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthLayer").finish()
    }
}

/// User context extracted from authenticated session
#[derive(Debug, Clone)]
pub struct UserContext {
    pub user_id: String,
    pub role: UserRole,
}

/// Fix 2 — C2: FromRequestParts for UserContext
///
/// Extracts the authenticated user from the request extensions.  The auth
/// middleware must have inserted a `UserContext` into `request.extensions`
/// before this extractor is called.  If no `UserContext` is present the
/// request is rejected with 401 Unauthorized.
impl<S> FromRequestParts<S> for UserContext
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<UserContext>()
            .cloned()
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    axum::Json(serde_json::json!({
                        "error": "Unauthorized",
                        "error_code": "UNAUTHORIZED",
                    })),
                )
                    .into_response()
            })
    }
}

/// User role for authorization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRole {
    Admin,
    Operator,
    Viewer,
}

impl UserRole {
    /// Check if this role is allowed to perform admin operations
    pub fn is_admin(&self) -> bool {
        *self == UserRole::Admin
    }
}
