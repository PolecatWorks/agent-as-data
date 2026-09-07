use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct HelloRequest {
    #[schemars(description = "The name of the user, persona, or entity to greet.")]
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct AadMcpServer {
    tool_router: ToolRouter<Self>,
}

impl AadMcpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

impl Default for AadMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl AadMcpServer {
    #[tool(description = "Generates a friendly greeting response for a specified user or entity name.")]
    pub fn hello(&self, Parameters(HelloRequest { name }): Parameters<HelloRequest>) -> Result<String, String> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err("Error: 'name' argument must be a non-empty string".to_string());
        }
        Ok(format!("Hello, {}!", trimmed))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for AadMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("Agent-As-Data MCP Server")
    }
}
