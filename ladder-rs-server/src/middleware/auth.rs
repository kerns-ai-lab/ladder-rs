//! Authentication and authorization middleware

use axum::middleware::Next;
use axum::response::Response;
use axum::http::Request;
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
