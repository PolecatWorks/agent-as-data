use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone, sqlx::FromRow)]
pub struct ThreadRun {
    pub id: Uuid,
    pub thread_id: Uuid,
    pub bench_id: Uuid,
    pub status: String,
    pub current_phase: String,
    pub active_tool_name: Option<String>,
    pub error: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateMessageResponse {
    #[serde(flatten)]
    pub message: super::thread::Message,
    pub run_id: Option<Uuid>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CancelRunResponse {
    pub message: String,
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::thread::Message;

    #[test]
    fn test_thread_run_serialization() {
        let run = ThreadRun {
            id: Uuid::new_v4(),
            thread_id: Uuid::new_v4(),
            bench_id: Uuid::new_v4(),
            status: "running".to_string(),
            current_phase: "thinking".to_string(),
            active_tool_name: Some("read_file".to_string()),
            error: None,
            created_at: Some(chrono::Utc::now()),
            updated_at: Some(chrono::Utc::now()),
        };

        let json = serde_json::to_string(&run).expect("Failed to serialize ThreadRun");
        let deserialized: ThreadRun = serde_json::from_str(&json).expect("Failed to deserialize ThreadRun");
        assert_eq!(run.id, deserialized.id);
        assert_eq!(run.status, deserialized.status);
        assert_eq!(run.active_tool_name, deserialized.active_tool_name);
    }

    #[test]
    fn test_create_message_response_flattening() {
        let msg = Message {
            id: Uuid::new_v4(),
            thread_id: Uuid::new_v4(),
            role: "user".to_string(),
            content: "Test prompt".to_string(),
            created_at: Some(chrono::Utc::now()),
        };
        let run_id = Uuid::new_v4();

        let resp = CreateMessageResponse {
            message: msg.clone(),
            run_id: Some(run_id),
        };

        let json = serde_json::to_string(&resp).expect("Failed to serialize CreateMessageResponse");
        assert!(json.contains("Test prompt"));
        assert!(json.contains(&run_id.to_string()));

        let deserialized_as_msg: Message = serde_json::from_str(&json).expect("Should deserialize as Message directly");
        assert_eq!(deserialized_as_msg.id, msg.id);
        assert_eq!(deserialized_as_msg.content, "Test prompt");
    }
}
