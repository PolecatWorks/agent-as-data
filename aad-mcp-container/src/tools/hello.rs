use std::future::Future;
use std::pin::Pin;
use serde_json::{json, Value};
use super::{Tool, ToolResult};

#[derive(Default, Clone, Debug)]
pub struct HelloTool;

impl HelloTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for HelloTool {
    fn name(&self) -> &str {
        "hello"
    }

    fn description(&self) -> &str {
        "Generates a friendly greeting response for a specified user or entity name."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "The name of the user, persona, or entity to greet.",
                    "minLength": 1
                }
            },
            "required": ["name"],
            "additionalProperties": false
        })
    }

    fn execute<'a>(
        &'a self,
        arguments: Value,
    ) -> Pin<Box<dyn Future<Output = ToolResult> + Send + 'a>> {
        Box::pin(async move {
            let name_val = match arguments.get("name") {
                Some(val) => val,
                None => {
                    return ToolResult::error_text("Error: 'name' argument must be a non-empty string");
                }
            };

            let name_str = match name_val.as_str() {
                Some(s) => s.trim(),
                None => {
                    return ToolResult::error_text("Error: 'name' argument must be a non-empty string");
                }
            };

            if name_str.is_empty() {
                return ToolResult::error_text("Error: 'name' argument must be a non-empty string");
            }

            ToolResult::success_text(format!("Hello, {}!", name_str))
        })
    }
}
