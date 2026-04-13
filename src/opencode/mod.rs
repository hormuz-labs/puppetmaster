use crate::config::OpencodeConfig;
use crate::error::{BotError, Result};
use std::sync::Arc;
use tracing::{debug, error, info};

pub mod events;
pub mod types;

pub use types::*;

/// Wrapper around the OpenCode API client
#[derive(Clone)]
pub struct OpenCodeClient {
    base_url: String,
    auth: Option<String>,
    config: OpencodeConfig,
}

impl OpenCodeClient {
    /// Create a new OpenCode client from configuration
    pub fn new(config: &OpencodeConfig) -> Result<Self> {
        let auth = if let Some(password) = &config.password {
            let credentials = format!("{}:{}", config.username, password);
            Some(format!("Basic {}", base64::encode(credentials)))
        } else {
            None
        };

        Ok(Self {
            base_url: config.api_url.clone(),
            auth,
            config: config.clone(),
        })
    }

    /// Check if the OpenCode server is available
    pub async fn health_check(&self) -> Result<bool> {
        // Placeholder health check
        // In production, this would make an actual HTTP request
        debug!("Checking OpenCode server health at {}", self.base_url);
        Ok(true)
    }

    /// Get server information
    pub async fn get_server_info(&self) -> Result<ServerInfo> {
        Ok(ServerInfo {
            hostname: self.base_url.clone(),
            version: "unknown".to_string(),
        })
    }

    /// List available sessions
    pub async fn list_sessions(&self) -> Result<Vec<SessionInfo>> {
        // Placeholder implementation
        Ok(vec![])
    }

    /// Create a new session
    pub async fn create_session(&self, title: Option<&str>) -> Result<SessionInfo> {
        let title = title.unwrap_or("New Session").to_string();
        let id = uuid::Uuid::new_v4().to_string();
        
        info!("Created new session: {} ({})", title, id);
        
        Ok(SessionInfo {
            id,
            title,
            directory: "/tmp".to_string(),
        })
    }

    /// Get session details
    pub async fn get_session(&self, session_id: &str) -> Result<SessionInfo> {
        // Placeholder
        Ok(SessionInfo {
            id: session_id.to_string(),
            title: "Session".to_string(),
            directory: "/tmp".to_string(),
        })
    }

    /// List available projects
    pub async fn list_projects(&self) -> Result<Vec<ProjectInfo>> {
        // Placeholder
        Ok(vec![])
    }

    /// Send a message/prompt to a session
    pub async fn send_prompt(&self, session_id: &str, prompt: &str) -> Result<()> {
        info!("Sending prompt to session {}: {}", session_id, 
            prompt.chars().take(50).collect::<String>());
        Ok(())
    }

    /// Abort the current operation in a session
    pub async fn abort_session(&self, session_id: &str) -> Result<()> {
        info!("Aborting session {}", session_id);
        Ok(())
    }

    /// Rename a session
    pub async fn rename_session(&self, session_id: &str, new_title: &str) -> Result<()> {
        info!("Renamed session {} to {}", session_id, new_title);
        Ok(())
    }

    /// Get the current model for the session
    pub async fn get_session_model(&self, _session_id: &str) -> Result<ModelInfo> {
        Ok(ModelInfo {
            provider: self.config.model_provider.clone(),
            model_id: self.config.model_id.clone(),
        })
    }

    /// Set the model for a session
    pub async fn set_session_model(
        &self,
        session_id: &str,
        provider: &str,
        model_id: &str,
    ) -> Result<()> {
        info!("Set session {} model to {}/{}", session_id, provider, model_id);
        Ok(())
    }

    /// Subscribe to events for a directory
    pub async fn subscribe_events(
        &self,
        _directory: &str,
    ) -> Result<events::EventStream> {
        Ok(events::EventStream::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OpencodeConfig;

    fn create_test_config() -> OpencodeConfig {
        OpencodeConfig {
            api_url: "http://localhost:4096".to_string(),
            username: "opencode".to_string(),
            password: None,
            model_provider: "opencode".to_string(),
            model_id: "big-pickle".to_string(),
        }
    }

    #[test]
    fn test_client_creation() {
        let config = create_test_config();
        let client = OpenCodeClient::new(&config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_creation_with_auth() {
        let mut config = create_test_config();
        config.password = Some("secret".to_string());
        
        let client = OpenCodeClient::new(&config);
        assert!(client.is_ok());
    }
}
