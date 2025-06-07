/// MCP (Model Context Protocol) integration for container-use tools
/// 
/// This module provides safe abstractions for interacting with container-use MCP tools
/// within the ladder-rs rating system.

use std::collections::HashMap;
use crate::error::LadderError;

/// Configuration for MCP container-use integration
#[derive(Debug, Clone)]
pub struct McpConfig {
    pub environment_id: String,
    pub allowed_operations: Vec<McpOperation>,
    pub timeout_ms: u64,
}

/// Available MCP operations for container-use
#[derive(Debug, Clone, PartialEq)]
pub enum McpOperation {
    FileRead,
    FileWrite,
    FileDelete,
    FileList,
    RunCommand,
    EnvironmentUpdate,
    Checkpoint,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            environment_id: String::new(),
            allowed_operations: vec![
                McpOperation::FileRead,
                McpOperation::FileList,
                McpOperation::RunCommand,
            ],
            timeout_ms: 30000,
        }
    }
}

/// MCP Client for container-use operations
pub struct McpClient {
    config: McpConfig,
    environment_id: String,
}

impl McpClient {
    /// Create a new MCP client with the given configuration
    pub fn new(config: McpConfig, environment_id: String) -> Self {
        Self {
            config,
            environment_id,
        }
    }

    /// Check if an operation is allowed by the current configuration
    pub fn is_operation_allowed(&self, operation: &McpOperation) -> bool {
        self.config.allowed_operations.contains(operation)
    }

    /// Get the current environment ID
    pub fn environment_id(&self) -> &str {
        &self.environment_id
    }

    /// Validate environment access for rating operations
    pub fn validate_environment_access(&self) -> Result<(), LadderError> {
        if self.environment_id.is_empty() {
            return Err(LadderError::InvalidInput("Environment ID cannot be empty".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_config_default() {
        let config = McpConfig::default();
        assert_eq!(config.timeout_ms, 30000);
        assert!(config.allowed_operations.contains(&McpOperation::FileRead));
    }

    #[test]
    fn test_mcp_client_operation_allowed() {
        let config = McpConfig::default();
        let client = McpClient::new(config, "test-env".to_string());
        
        assert!(client.is_operation_allowed(&McpOperation::FileRead));
        assert!(!client.is_operation_allowed(&McpOperation::FileWrite));
    }

    #[test]
    fn test_environment_validation() {
        let config = McpConfig::default();
        let client = McpClient::new(config, "test-env".to_string());
        
        assert!(client.validate_environment_access().is_ok());
        
        let empty_client = McpClient::new(McpConfig::default(), String::new());
        assert!(empty_client.validate_environment_access().is_err());
    }
}