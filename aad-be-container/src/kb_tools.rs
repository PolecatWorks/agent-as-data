use std::fmt;
use rig_core::tool::PortableTool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use reqwest::Client;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug)]
pub struct ToolError(pub String);

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ToolError {}

#[derive(Clone)]
pub struct KbNodeBrowseTool {
    pub backend_url: String, // e.g. "http://127.0.0.1:8080"
}

#[derive(Deserialize)]
pub struct KbNodeBrowseArgs {
    pub query: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Serialize)]
pub struct KbNodeBrowseOutput {
    pub results: serde_json::Value,
}

impl PortableTool for KbNodeBrowseTool {
    const NAME: &'static str = "kb_node_browse";
    type Error = ToolError;
    type Args = KbNodeBrowseArgs;
    type Output = KbNodeBrowseOutput;

    fn description(&self) -> String {
        "Browse or search knowledge nodes. If a query is provided, it performs a search. Otherwise, it lists the nodes.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Optional search query to find relevant knowledge nodes."
                },
                "limit": {
                    "type": "number",
                    "description": "Maximum number of results to return."
                }
            }
        })
    }

    #[allow(refining_impl_trait)]
    fn call(
        &self,
        args: Self::Args,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Output, Self::Error>> + Send>> {
        let backend_url = self.backend_url.clone();
        Box::pin(async move {
            let client = Client::new();
            if let Some(query) = args.query {
                // Search
                let payload = json!({
                    "query": query,
                    "limit": args.limit.unwrap_or(10)
                });
                let res = client.post(format!("{}/api/v1/knowledge/search", backend_url))
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| ToolError(e.to_string()))?;

                let results: serde_json::Value = res.json().await.map_err(|e| ToolError(e.to_string()))?;
                Ok(KbNodeBrowseOutput { results })
            } else {
                // List
                let mut url = format!("{}/api/v1/knowledge", backend_url);
                if let Some(limit) = args.limit {
                    url.push_str(&format!("?limit={}", limit));
                }
                let res = client.get(url)
                    .send()
                    .await
                    .map_err(|e| ToolError(e.to_string()))?;

                let results: serde_json::Value = res.json().await.map_err(|e| ToolError(e.to_string()))?;
                Ok(KbNodeBrowseOutput { results })
            }
        })
    }
}

// Read Tool
#[derive(Clone)]
pub struct KbNodeReadTool {
    pub backend_url: String,
}

#[derive(Deserialize)]
pub struct KbNodeReadArgs {
    pub id: String,
}

#[derive(Serialize)]
pub struct KbNodeReadOutput {
    pub node: serde_json::Value,
}

impl PortableTool for KbNodeReadTool {
    const NAME: &'static str = "kb_node_read";
    type Error = ToolError;
    type Args = KbNodeReadArgs;
    type Output = KbNodeReadOutput;

    fn description(&self) -> String {
        "Read the full details and content of a specific knowledge node by its ID.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "The UUID of the knowledge node to read."
                }
            },
            "required": ["id"]
        })
    }

    #[allow(refining_impl_trait)]
    fn call(
        &self,
        args: Self::Args,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Output, Self::Error>> + Send>> {
        let backend_url = self.backend_url.clone();
        Box::pin(async move {
            let client = Client::new();
            let res = client.get(format!("{}/api/v1/knowledge/{}", backend_url, args.id))
                .send()
                .await
                .map_err(|e| ToolError(e.to_string()))?;

            if !res.status().is_success() {
                return Err(ToolError(format!("Failed to read node: {}", res.status())));
            }

            let node: serde_json::Value = res.json().await.map_err(|e| ToolError(e.to_string()))?;
            Ok(KbNodeReadOutput { node })
        })
    }
}

// Add Tool
#[derive(Clone)]
pub struct KbNodeAddTool {
    pub backend_url: String,
}

#[derive(Deserialize)]
pub struct KbNodeAddArgs {
    pub topic: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub content: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct KbNodeAddOutput {
    pub result: serde_json::Value,
}

impl PortableTool for KbNodeAddTool {
    const NAME: &'static str = "kb_node_add";
    type Error = ToolError;
    type Args = KbNodeAddArgs;
    type Output = KbNodeAddOutput;

    fn description(&self) -> String {
        "Create a new knowledge node. Used to add new information to the knowledge base.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "topic": { "type": "string" },
                "title": { "type": "string" },
                "description": { "type": "string" },
                "content": { "type": "string" },
                "tags": { "type": "array", "items": { "type": "string" } }
            },
            "required": ["topic", "content"]
        })
    }

    #[allow(refining_impl_trait)]
    fn call(
        &self,
        args: Self::Args,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Output, Self::Error>> + Send>> {
        let backend_url = self.backend_url.clone();
        Box::pin(async move {
            let client = Client::new();
            let payload = json!({
                "topic": args.topic,
                "title": args.title,
                "description": args.description,
                "content": args.content,
                "tags": args.tags
            });
            let res = client.post(format!("{}/api/v1/knowledge", backend_url))
                .json(&payload)
                .send()
                .await
                .map_err(|e| ToolError(e.to_string()))?;

            if !res.status().is_success() {
                return Err(ToolError(format!("Failed to add node: {}", res.status())));
            }

            let result: serde_json::Value = res.json().await.map_err(|e| ToolError(e.to_string()))?;
            Ok(KbNodeAddOutput { result })
        })
    }
}

// Edit Tool
#[derive(Clone)]
pub struct KbNodeEditTool {
    pub backend_url: String,
}

#[derive(Deserialize)]
pub struct KbNodeEditArgs {
    pub id: String,
    pub topic: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct KbNodeEditOutput {
    pub result: String,
}

impl PortableTool for KbNodeEditTool {
    const NAME: &'static str = "kb_node_edit";
    type Error = ToolError;
    type Args = KbNodeEditArgs;
    type Output = KbNodeEditOutput;

    fn description(&self) -> String {
        "Update an existing knowledge node.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "topic": { "type": "string" },
                "title": { "type": "string" },
                "description": { "type": "string" },
                "content": { "type": "string" },
                "tags": { "type": "array", "items": { "type": "string" } }
            },
            "required": ["id"]
        })
    }

    #[allow(refining_impl_trait)]
    fn call(
        &self,
        args: Self::Args,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Output, Self::Error>> + Send>> {
        let backend_url = self.backend_url.clone();
        Box::pin(async move {
            let client = Client::new();
            let payload = json!({
                "topic": args.topic,
                "title": args.title,
                "description": args.description,
                "content": args.content,
                "tags": args.tags
            });
            let res = client.put(format!("{}/api/v1/knowledge/{}", backend_url, args.id))
                .json(&payload)
                .send()
                .await
                .map_err(|e| ToolError(e.to_string()))?;

            if !res.status().is_success() {
                return Err(ToolError(format!("Failed to edit node: {}", res.status())));
            }

            Ok(KbNodeEditOutput { result: "Success".to_string() })
        })
    }
}

// Delete Tool
#[derive(Clone)]
pub struct KbNodeDeleteTool {
    pub backend_url: String,
}

#[derive(Deserialize)]
pub struct KbNodeDeleteArgs {
    pub id: String,
}

#[derive(Serialize)]
pub struct KbNodeDeleteOutput {
    pub result: String,
}

impl PortableTool for KbNodeDeleteTool {
    const NAME: &'static str = "kb_node_delete";
    type Error = ToolError;
    type Args = KbNodeDeleteArgs;
    type Output = KbNodeDeleteOutput;

    fn description(&self) -> String {
        "Delete a knowledge node by its ID.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string" }
            },
            "required": ["id"]
        })
    }

    #[allow(refining_impl_trait)]
    fn call(
        &self,
        args: Self::Args,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Output, Self::Error>> + Send>> {
        let backend_url = self.backend_url.clone();
        Box::pin(async move {
            let client = Client::new();
            let res = client.delete(format!("{}/api/v1/knowledge/{}", backend_url, args.id))
                .send()
                .await
                .map_err(|e| ToolError(e.to_string()))?;

            if !res.status().is_success() {
                return Err(ToolError(format!("Failed to delete node: {}", res.status())));
            }

            Ok(KbNodeDeleteOutput { result: "Success".to_string() })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kb_tools_metadata_and_parameters() {
        let browse_tool = KbNodeBrowseTool { backend_url: "http://localhost:8080".to_string() };
        assert_eq!(KbNodeBrowseTool::NAME, "kb_node_browse");
        assert!(!browse_tool.description().is_empty());
        assert!(browse_tool.parameters().is_object());

        let read_tool = KbNodeReadTool { backend_url: "http://localhost:8080".to_string() };
        assert_eq!(KbNodeReadTool::NAME, "kb_node_read");
        assert!(!read_tool.description().is_empty());
        assert_eq!(read_tool.parameters()["required"][0], "id");

        let add_tool = KbNodeAddTool { backend_url: "http://localhost:8080".to_string() };
        assert_eq!(KbNodeAddTool::NAME, "kb_node_add");
        assert_eq!(add_tool.parameters()["required"][0], "topic");
        assert_eq!(add_tool.parameters()["required"][1], "content");

        let edit_tool = KbNodeEditTool { backend_url: "http://localhost:8080".to_string() };
        assert_eq!(KbNodeEditTool::NAME, "kb_node_edit");
        assert_eq!(edit_tool.parameters()["required"][0], "id");

        let delete_tool = KbNodeDeleteTool { backend_url: "http://localhost:8080".to_string() };
        assert_eq!(KbNodeDeleteTool::NAME, "kb_node_delete");
        assert_eq!(delete_tool.parameters()["required"][0], "id");
    }

    #[test]
    fn test_kb_tools_args_deserialization() {
        let add_json = json!({
            "topic": "test-topic",
            "content": "test-content",
            "title": "Title",
            "tags": ["tag1", "tag2"]
        });
        let add_args: KbNodeAddArgs = serde_json::from_value(add_json).expect("deserialize add args");
        assert_eq!(add_args.topic, "test-topic");
        assert_eq!(add_args.content, "test-content");
        assert_eq!(add_args.title.as_deref(), Some("Title"));
        assert_eq!(add_args.tags.as_ref().unwrap().len(), 2);

        let browse_json = json!({
            "query": "search term",
            "limit": 5
        });
        let browse_args: KbNodeBrowseArgs = serde_json::from_value(browse_json).expect("deserialize browse args");
        assert_eq!(browse_args.query.as_deref(), Some("search term"));
        assert_eq!(browse_args.limit, Some(5));
    }
}
