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

    pub fn list_tools(&self) -> Vec<rmcp::model::Tool> {
        self.tool_router.list_all()
    }

    pub fn info(&self) -> ServerInfo {
        self.get_info()
    }

    pub async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match name {
            "hello" => {
                let req: HelloRequest = serde_json::from_value(arguments)
                    .map_err(|e| format!("Invalid arguments for 'hello': {}", e))?;
                let greeting = self.hello(Parameters(req))?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": greeting
                        }
                    ],
                    "isError": false
                }))
            }
            _ => Err(format!("Error: Tool '{}' not found", name)),
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
