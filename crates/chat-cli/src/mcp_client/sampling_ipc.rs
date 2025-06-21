use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use serde::{Deserialize, Serialize};

use crate::mcp_client::{ModelPreferences, SamplingApprovalResult};

static REQUEST_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// IPC message for requesting sampling approval from desktop app
#[derive(Debug, Serialize, Deserialize)]
pub struct SamplingApprovalRequest {
    pub server_name: String,
    pub prompt_content: String,
    pub system_prompt: Option<String>,
    pub model_preferences: Option<ModelPreferences>,
    pub request_id: u64,
    pub max_tokens: Option<u32>,
    pub include_context: Option<String>,
    pub temperature: Option<f64>,
    pub stop_sequences: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

/// IPC response from desktop app for sampling approval
#[derive(Debug, Serialize, Deserialize)]
pub struct SamplingApprovalResponse {
    pub request_id: u64,
    pub approved: bool,
    pub modified_prompt: Option<String>,
    pub error_message: Option<String>,
}

/// Handle IPC communication for sampling approval
pub struct SamplingIpcHandler {
    // In a full implementation, this would contain IPC client/connection
}

impl SamplingIpcHandler {
    pub fn new() -> Self {
        Self {}
    }

    /// Request user approval for sampling via IPC to desktop app
    pub async fn request_approval(
        &self,
        server_name: &str,
        prompt_content: &str,
        system_prompt: &Option<String>,
        model_preferences: &Option<ModelPreferences>,
        max_tokens: Option<u32>,
        include_context: &Option<String>,
        temperature: Option<f64>,
        stop_sequences: &Option<Vec<String>>,
        metadata: &Option<serde_json::Value>,
    ) -> Result<SamplingApprovalResult, SamplingIpcError> {
        let request_id = REQUEST_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        
        let request = SamplingApprovalRequest {
            server_name: server_name.to_string(),
            prompt_content: prompt_content.to_string(),
            system_prompt: system_prompt.clone(),
            model_preferences: model_preferences.clone(),
            request_id,
            max_tokens,
            include_context: include_context.clone(),
            temperature,
            stop_sequences: stop_sequences.clone(),
            metadata: metadata.clone(),
        };

        // In a full implementation, this would:
        // 1. Serialize the request to protocol buffer format
        // 2. Send via IPC to fig_desktop
        // 3. Wait for response with timeout
        // 4. Handle fallback to terminal approval if desktop unavailable
        
        tracing::info!(
            target: "mcp_sampling", 
            "Requesting approval for sampling from server: {} (request_id: {})", 
            server_name, 
            request_id
        );

        // For now, simulate the approval process
        let response = self.simulate_approval_process(&request).await?;

        Ok(SamplingApprovalResult {
            approved: response.approved,
            modified_prompt: response.modified_prompt,
            error_message: response.error_message,
        })
    }

    /// Simulate the approval process (placeholder for real IPC)
    async fn simulate_approval_process(
        &self,
        request: &SamplingApprovalRequest,
    ) -> Result<SamplingApprovalResponse, SamplingIpcError> {
        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(100)).await;

        // For testing purposes, auto-approve simple requests
        let should_approve = request.prompt_content.len() < 1000 && 
                           !request.prompt_content.to_lowercase().contains("dangerous");

        Ok(SamplingApprovalResponse {
            request_id: request.request_id,
            approved: should_approve,
            modified_prompt: None,
            error_message: if should_approve {
                None
            } else {
                Some("Request rejected due to content policy".to_string())
            },
        })
    }

    /// Fallback to terminal-based approval when desktop app unavailable
    pub async fn request_terminal_approval(
        &self,
        server_name: &str,
        prompt_content: &str,
    ) -> Result<SamplingApprovalResult, SamplingIpcError> {
        // In a full implementation, this would:
        // 1. Display approval prompt in terminal
        // 2. Wait for user input (y/n)
        // 3. Return approval result
        
        tracing::info!(
            target: "mcp_sampling",
            "Fallback to terminal approval for server: {}",
            server_name
        );

        // For now, auto-approve in terminal mode
        Ok(SamplingApprovalResult {
            approved: true,
            modified_prompt: None,
            error_message: None,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SamplingIpcError {
    #[error("IPC communication failed: {0}")]
    IpcError(String),
    
    #[error("Request timeout")]
    Timeout,
    
    #[error("Desktop app unavailable")]
    DesktopUnavailable,
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl Default for SamplingIpcHandler {
    fn default() -> Self {
        Self::new()
    }
}
