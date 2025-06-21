use std::sync::Arc;

use fig_desktop_api::handler::Wrapped;
use fig_desktop_api::requests::{RequestResult, RequestResultImpl};
use fig_os_shim::{ContextArcProvider, ContextProvider};
use fig_proto::fig::{SamplingApprovalRequest, SamplingApprovalResponse};

/// Handle sampling approval requests from MCP clients
pub async fn handle_sampling_approval_request<T: ContextProvider>(
    ctx: ContextArcProvider<T>,
    request: SamplingApprovalRequest,
) -> RequestResult<SamplingApprovalResponse> {
    tracing::info!(
        target: "sampling",
        "Received sampling approval request from server: {} (request_id: {})",
        request.server_name,
        request.request_id
    );

    // In a full implementation, this would:
    // 1. Show a native dialog with the sampling request details
    // 2. Display the server name, prompt content, and model preferences
    // 3. Provide approve/deny/modify options
    // 4. Handle user interaction and return the response

    let approval_result = show_sampling_approval_dialog(&request).await;

    Ok(SamplingApprovalResponse {
        request_id: request.request_id,
        approved: approval_result.approved,
        modified_prompt: approval_result.modified_prompt,
        error_message: approval_result.error_message,
    })
}

/// Show native approval dialog for sampling request
async fn show_sampling_approval_dialog(
    request: &SamplingApprovalRequest,
) -> SamplingApprovalResult {
    // For now, implement a simple approval logic
    // In a full implementation, this would show a native dialog using:
    // - macOS: NSAlert or custom window
    // - Linux: GTK dialog
    // - Windows: MessageBox or custom dialog

    tracing::info!(
        target: "sampling",
        "Showing approval dialog for sampling request from: {}",
        request.server_name
    );

    // Simulate user interaction delay
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Simple approval logic for testing
    let should_approve = request.prompt_content.len() < 2000 && 
                        !request.prompt_content.to_lowercase().contains("harmful");

    SamplingApprovalResult {
        approved: should_approve,
        modified_prompt: None,
        error_message: if should_approve {
            None
        } else {
            Some("Request rejected by user".to_string())
        },
    }
}

/// Result of sampling approval dialog
struct SamplingApprovalResult {
    approved: bool,
    modified_prompt: Option<String>,
    error_message: Option<String>,
}

/// Register sampling request handler
pub fn register_sampling_handler<T: ContextProvider>(
    ctx: ContextArcProvider<T>,
) -> Wrapped<SamplingApprovalRequest, SamplingApprovalResponse> {
    Wrapped::new(move |request| {
        let ctx = ctx.clone();
        async move {
            handle_sampling_approval_request(ctx, request).await
        }
    })
}
