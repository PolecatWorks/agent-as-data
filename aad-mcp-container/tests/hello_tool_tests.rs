use aad_mcp_container::tools::hello::HelloTool;
use aad_mcp_container::tools::Tool;
use serde_json::json;

#[tokio::test]
async fn test_hello_tool_metadata() {
    let tool = HelloTool::new();
    assert_eq!(tool.name(), "hello");
    assert!(!tool.description().is_empty());

    let schema = tool.input_schema();
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["name"].is_object());
    assert_eq!(schema["required"], json!(["name"]));
}

#[tokio::test]
async fn test_hello_tool_valid_greeting() {
    let tool = HelloTool::new();
    let result = tool.execute(json!({ "name": "Alice" })).await;

    assert!(!result.is_error);
    assert_eq!(result.content.len(), 1);
    assert_eq!(result.content[0].type_field, "text");
    assert_eq!(result.content[0].text, "Hello, Alice!");
}

#[tokio::test]
async fn test_hello_tool_whitespace_trimming() {
    let tool = HelloTool::new();
    let result = tool.execute(json!({ "name": "   Bob   " })).await;

    assert!(!result.is_error);
    assert_eq!(result.content[0].text, "Hello, Bob!");
}

#[tokio::test]
async fn test_hello_tool_utf8_greeting() {
    let tool = HelloTool::new();
    let result1 = tool.execute(json!({ "name": "José" })).await;
    assert!(!result1.is_error);
    assert_eq!(result1.content[0].text, "Hello, José!");

    let result2 = tool.execute(json!({ "name": "世界" })).await;
    assert!(!result2.is_error);
    assert_eq!(result2.content[0].text, "Hello, 世界!");
}

#[tokio::test]
async fn test_hello_tool_empty_name_error() {
    let tool = HelloTool::new();
    let result = tool.execute(json!({ "name": "" })).await;

    assert!(result.is_error);
    assert!(result.content[0].text.contains("non-empty string"));
}

#[tokio::test]
async fn test_hello_tool_whitespace_only_error() {
    let tool = HelloTool::new();
    let result = tool.execute(json!({ "name": "     " })).await;

    assert!(result.is_error);
    assert!(result.content[0].text.contains("non-empty string"));
}

#[tokio::test]
async fn test_hello_tool_missing_name_field_error() {
    let tool = HelloTool::new();
    let result = tool.execute(json!({})).await;

    assert!(result.is_error);
    assert!(result.content[0].text.contains("non-empty string"));
}

#[tokio::test]
async fn test_hello_tool_invalid_type_error() {
    let tool = HelloTool::new();
    let result = tool.execute(json!({ "name": 12345 })).await;

    assert!(result.is_error);
    assert!(result.content[0].text.contains("non-empty string"));
}
