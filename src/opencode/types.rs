use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub hostname: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionInfo {
    pub id: String,
    pub title: String,
    pub directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub worktree: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelInfo {
    pub provider: String,
    pub model_id: String,
}

impl std::fmt::Display for ModelInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.provider, self.model_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeEvent {
    pub event_type: EventType,
    pub session_id: Option<String>,
    pub properties: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    SessionCreated,
    SessionUpdated,
    SessionIdle,
    SessionError,
    MessagePartUpdated,
    MessageCompleted,
    ToolCall,
    ToolOutput,
    SubagentUpdate,
    PermissionRequest,
    QuestionAsked,
    ThinkingStarted,
    TokensUpdate,
    CostUpdate,
    SessionCompacted,
    SessionDiff,
    FileChange,
    Unknown,
}

impl OpenCodeEvent {
    pub fn from_json(json: &str) -> crate::error::Result<Self> {
        let event: OpenCodeEvent = serde_json::from_str(json)?;
        Ok(event)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallInfo {
    pub session_id: String,
    pub tool: String,
    pub args: serde_json::Value,
    pub has_file_attachment: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub current_task: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub id: String,
    pub session_id: String,
    pub permission: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    pub text: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub input: usize,
    pub output: usize,
    pub cache_read: usize,
    pub cache_write: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_info_display() {
        let model = ModelInfo {
            provider: "opencode".to_string(),
            model_id: "big-pickle".to_string(),
        };
        assert_eq!(model.to_string(), "opencode/big-pickle");
    }

    #[test]
    fn test_event_type_serialization() {
        let event_type = EventType::SessionCreated;
        let json = serde_json::to_string(&event_type).unwrap();
        assert!(json.contains("SessionCreated"));

        let deserialized: EventType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, EventType::SessionCreated);
    }

    #[test]
    fn test_opencode_event_from_json() {
        let json = r#"{
            "event_type": "SessionCreated",
            "session_id": "test-session",
            "properties": {"key": "value"}
        }"#;
        
        let event = OpenCodeEvent::from_json(json).unwrap();
        assert_eq!(event.event_type, EventType::SessionCreated);
        assert_eq!(event.session_id, Some("test-session".to_string()));
    }
}
