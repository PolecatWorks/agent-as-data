use aad_mcp_container::server::{AadMcpServer, HelloRequest};
use rmcp::handler::server::wrapper::Parameters;

#[test]
fn test_hello_tool_valid_greeting() {
    let server = AadMcpServer::new();
    let result = server.hello(Parameters(HelloRequest {
        name: "Alice".to_string(),
    }));

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello, Alice!");
}

#[test]
fn test_hello_tool_whitespace_trimming() {
    let server = AadMcpServer::new();
    let result = server.hello(Parameters(HelloRequest {
        name: "   Bob   ".to_string(),
    }));

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello, Bob!");
}

#[test]
fn test_hello_tool_utf8_greeting() {
    let server = AadMcpServer::new();
    let result1 = server.hello(Parameters(HelloRequest {
        name: "José".to_string(),
    }));
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap(), "Hello, José!");

    let result2 = server.hello(Parameters(HelloRequest {
        name: "世界".to_string(),
    }));
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap(), "Hello, 世界!");
}

#[test]
fn test_hello_tool_empty_name_error() {
    let server = AadMcpServer::new();
    let result = server.hello(Parameters(HelloRequest {
        name: "".to_string(),
    }));

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("non-empty string"));
}

#[test]
fn test_hello_tool_whitespace_only_error() {
    let server = AadMcpServer::new();
    let result = server.hello(Parameters(HelloRequest {
        name: "     ".to_string(),
    }));

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("non-empty string"));
}

#[tokio::test]
async fn test_hello_tool_list_and_call() {
    let server = AadMcpServer::new();
    let tools = server.list_tools();
    assert_eq!(tools.len(), 6);
    assert!(tools.iter().any(|t| t.name == "hello"));

    let call_res = server.call_tool("hello", serde_json::json!({ "name": "Antigravity" })).await;
    assert!(call_res.is_ok());
    let val = call_res.unwrap();
    assert_eq!(val["isError"], false);
    assert_eq!(val["content"][0]["text"], "Hello, Antigravity!");

    let missing_res = server.call_tool("missing", serde_json::json!({})).await;
    assert!(missing_res.is_err());
}

