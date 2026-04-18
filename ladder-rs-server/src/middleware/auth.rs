//! Authentication and authorization middleware

use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use std::fmt;

use crate::error::ServerError;

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

#[async_trait]
impl<S> FromRequestParts<S> for UserContext
where
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the Authorization header and validate the token.
        // In the stub implementation, a missing or invalid header returns 401.
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if auth_header.is_empty() {
            return Err(ServerError::Unauthorized);
        }

        // Strip "Bearer " prefix if present
        let token = auth_header.strip_prefix("Bearer ").unwrap_or(auth_header);

        if token.is_empty() {
            return Err(ServerError::Unauthorized);
        }

        // Stub: treat any non-empty bearer token as a valid viewer session.
        // Real implementation would validate against session store / JWT.
        Ok(UserContext {
            user_id: token.to_string(),
            role: UserRole::Viewer,
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
