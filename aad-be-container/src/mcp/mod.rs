//! Unified Model Context Protocol (MCP) server module for Agent-As-Data.
//!
//! Follows the unified PolecatWorks MCP Server Pattern (`src/mcp/{mod.rs, rpc.rs, server.rs}`)
//! with centralized `McpConfig`, typed `McpError`, and instrumented JSON-RPC 2.0 dispatch.

use serde::{Deserialize, Serialize};

pub mod rpc;
pub mod server;

/// Typed error enum for MCP configuration validation and tool execution.
#[derive(thiserror::Error, Debug)]
pub enum McpError {
    #[error("MCP server_name cannot be empty")]
    EmptyServerName,
    #[error("MCP instructions cannot be empty")]
    EmptyInstructions,
    #[error("Tool '{0}' not found")]
    ToolNotFound(String),
    #[error("Invalid arguments for tool '{0}': {1}")]
    InvalidArguments(String, String),
    #[error("{0}")]
    Execution(String),
    #[error("JSON serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Configuration for the in-process Agent-As-Data MCP server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpConfig {
    /// Whether the HTTP MCP JSON-RPC endpoints (`/mcp`, `{api_prefix}/mcp`, `/api/v1/mcp`) are enabled.
    pub enabled: bool,
    /// MCP `serverInfo.name` advertised during `initialize`.
    pub server_name: String,
    /// Human-readable server instructions returned during `initialize`.
    pub instructions: String,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            server_name: "aad-be-mcp".to_string(),
            instructions: "Agent-As-Data (AAD) Unified MCP Server exposing Agents, Skills, Traits, MCP Tool Registry, Semantic Context Search, Knowledge Base & Graph, Workbenches/Memory, and Execution.".to_string(),
        }
    }
}

impl McpConfig {
    /// Fail-fast validation of MCP configuration when enabled.
    pub fn validate(&self) -> Result<(), McpError> {
        if self.enabled {
            if self.server_name.trim().is_empty() {
                return Err(McpError::EmptyServerName);
            }
            if self.instructions.trim().is_empty() {
                return Err(McpError::EmptyInstructions);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_mcp_config() {
        let config = McpConfig::default();
        assert!(config.enabled);
        assert_eq!(config.server_name, "aad-be-mcp");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_empty_server_name() {
        let mut config = McpConfig::default();
        config.server_name = "   ".to_string();
        assert!(matches!(config.validate(), Err(McpError::EmptyServerName)));
    }

    #[test]
    fn test_empty_instructions() {
        let mut config = McpConfig::default();
        config.instructions = "   ".to_string();
        assert!(matches!(
            config.validate(),
            Err(McpError::EmptyInstructions)
        ));
    }

    #[test]
    fn test_disabled_skips_validation() {
        let config = McpConfig {
            enabled: false,
            server_name: "".to_string(),
            instructions: "".to_string(),
        };
        assert!(config.validate().is_ok());
    }
}
