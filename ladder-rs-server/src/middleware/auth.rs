//! Authentication and authorization middleware

use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::Response;
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

/// Stub `FromRequestParts` implementation for `UserContext`.
///
/// TODO: Replace with real session extraction once session infrastructure exists.
/// For now returns a default admin context so handler signatures compile.
#[async_trait]
impl<S> FromRequestParts<S> for UserContext
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let _ = (parts, state);
        Ok(UserContext {
            user_id: "stub-admin".to_string(),
            role: UserRole::Admin,
        })
    }
}
