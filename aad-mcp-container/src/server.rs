use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct HelloRequest {
    #[schemars(description = "The name of the user, persona, or entity to greet.")]
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct RunOcrRequest {
    #[schemars(description = "Path or URL to the receipt image.")]
    pub image_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ConvertCurrencyRequest {
    #[schemars(description = "Amount to convert.")]
    pub amount: f64,
    #[schemars(description = "Original currency code (e.g. EUR).")]
    pub from_currency: String,
    #[schemars(description = "Target currency code (e.g. USD).")]
    pub to_currency: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct LookupHrOrgChartRequest {
    #[schemars(description = "The submitter ID to look up in the HR system.")]
    pub submitter_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct LookupCostCenterRequest {
    #[schemars(description = "The cost center ID to look up.")]
    pub cost_center_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct DispatchSlackNotificationRequest {
    #[schemars(description = "The email of the user to notify.")]
    pub user_email: String,
    #[schemars(description = "The notification message.")]
    pub message: String,
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
            "run_ocr" => {
                let req: RunOcrRequest = serde_json::from_value(arguments)
                    .map_err(|e| format!("Invalid arguments for 'run_ocr': {}", e))?;
                let res = self.run_ocr(Parameters(req))?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": res
                        }
                    ],
                    "isError": false
                }))
            }
            "convert_currency" => {
                let req: ConvertCurrencyRequest = serde_json::from_value(arguments)
                    .map_err(|e| format!("Invalid arguments for 'convert_currency': {}", e))?;
                let res = self.convert_currency(Parameters(req))?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": res
                        }
                    ],
                    "isError": false
                }))
            }
            "lookup_hr_org_chart" => {
                let req: LookupHrOrgChartRequest = serde_json::from_value(arguments)
                    .map_err(|e| format!("Invalid arguments for 'lookup_hr_org_chart': {}", e))?;
                let res = self.lookup_hr_org_chart(Parameters(req))?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": res
                        }
                    ],
                    "isError": false
                }))
            }
            "lookup_cost_center" => {
                let req: LookupCostCenterRequest = serde_json::from_value(arguments)
                    .map_err(|e| format!("Invalid arguments for 'lookup_cost_center': {}", e))?;
                let res = self.lookup_cost_center(Parameters(req))?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": res
                        }
                    ],
                    "isError": false
                }))
            }
            "dispatch_slack_notification" => {
                let req: DispatchSlackNotificationRequest = serde_json::from_value(arguments)
                    .map_err(|e| format!("Invalid arguments for 'dispatch_slack_notification': {}", e))?;
                let res = self.dispatch_slack_notification(Parameters(req))?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": res
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

    #[tool(description = "Simulates running OCR on a receipt image.")]
    pub fn run_ocr(&self, Parameters(req): Parameters<RunOcrRequest>) -> Result<String, String> {
        println!("Running OCR on {}...", req.image_path);
        Ok(serde_json::json!({
            "raw_text": "MERCHANT: ACME Corp\nDATE: 2023-10-25\nTOTAL: 150.00 EUR",
            "confidence": 0.95
        }).to_string())
    }

    #[tool(description = "Simulates converting currency.")]
    pub fn convert_currency(&self, Parameters(req): Parameters<ConvertCurrencyRequest>) -> Result<String, String> {
        println!("Converting {} {} to {}...", req.amount, req.from_currency, req.to_currency);
        let rate = if req.from_currency == "EUR" && req.to_currency == "USD" {
            1.1
        } else {
            1.0
        };
        Ok(serde_json::json!({
            "converted_amount": req.amount * rate
        }).to_string())
    }

    #[tool(description = "Simulates looking up the HR org chart to find a manager.")]
    pub fn lookup_hr_org_chart(&self, Parameters(req): Parameters<LookupHrOrgChartRequest>) -> Result<String, String> {
        println!("Looking up manager for employee {}...", req.submitter_id);
        Ok(serde_json::json!({
            "manager_id": "mgr_8899",
            "manager_name": "Jane Doe",
            "manager_email": "jane.doe@example.com"
        }).to_string())
    }

    #[tool(description = "Simulates looking up cost center details.")]
    pub fn lookup_cost_center(&self, Parameters(req): Parameters<LookupCostCenterRequest>) -> Result<String, String> {
        println!("Looking up cost center {}...", req.cost_center_id);
        Ok(serde_json::json!({
            "department": "Engineering",
            "budget_remaining": 50000.00,
            "requires_vp_approval": false
        }).to_string())
    }

    #[tool(description = "Simulates sending a Slack message.")]
    pub fn dispatch_slack_notification(&self, Parameters(req): Parameters<DispatchSlackNotificationRequest>) -> Result<String, String> {
        println!("Sending Slack message to {}:\n{}", req.user_email, req.message);
        Ok(serde_json::json!({
            "success": true
        }).to_string())
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for AadMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("Agent-As-Data MCP Server")
    }
}
