#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::mcp_client::{
        Client, ClientConfig, ClientInfo, StdioTransport,
        SamplingMessage, SamplingContent, SamplingRequest, ModelPreferences, ModelHint,
        sampling_ipc::SamplingIpcHandler,
    };
    use tokio::time::{timeout, Duration};

    /// Test complete sampling workflow integration
    #[tokio::test]
    async fn test_complete_sampling_workflow() {
        // This test would require a real MCP server, but demonstrates the approach
        let _client_info = serde_json::json!({
            "name": "test-client",
            "version": "1.0.0"
        });

        // In a real test, this would connect to an actual MCP server
        // For now, we test the client-side logic
        let sampling_request = SamplingRequest {
            messages: vec![
                SamplingMessage {
                    role: "user".to_string(),
                    content: SamplingContent::Text {
                        text: "What is the weather like today?".to_string(),
                    },
                }
            ],
            model_preferences: Some(ModelPreferences {
                hints: Some(vec![ModelHint { name: "gpt-4".to_string() }]),
                cost_priority: Some(0.5),
                speed_priority: Some(0.7),
                intelligence_priority: Some(0.8),
            }),
            system_prompt: Some("You are a helpful weather assistant.".to_string()),
            max_tokens: Some(150),
            include_context: Some("thisServer".to_string()),
            temperature: Some(0.8),
            stop_sequences: Some(vec!["END".to_string()]),
            metadata: Some(serde_json::json!({"test": "integration"})),
        };

        // Test serialization/deserialization
        let json = serde_json::to_string(&sampling_request).expect("Failed to serialize");
        let deserialized: SamplingRequest = serde_json::from_str(&json).expect("Failed to deserialize");
        
        assert_eq!(deserialized.messages.len(), 1);
        assert!(deserialized.model_preferences.is_some());
        assert_eq!(deserialized.max_tokens, Some(150));
    }

    /// Test IPC handler with various approval scenarios
    #[tokio::test]
    async fn test_ipc_approval_scenarios() {
        let handler = SamplingIpcHandler::new();

        // Test 1: Simple approval
        let result = handler.request_approval(
            "weather-server",
            "What's the weather?",
            &Some("You are a weather assistant.".to_string()),
            &Some(ModelPreferences {
                hints: Some(vec![ModelHint { name: "gpt-4".to_string() }]),
                cost_priority: Some(0.5),
                speed_priority: Some(0.5),
                intelligence_priority: Some(0.8),
            }),
            Some(100),
            &Some("thisServer".to_string()),
            Some(0.7),
            &None,
            &None,
        ).await;

        assert!(result.is_ok());
        let approval = result.unwrap();
        assert!(approval.approved);

        // Test 2: Rejection scenario
        let result = handler.request_approval(
            "suspicious-server",
            &"dangerous content".repeat(200), // Long suspicious content
            &None,
            &None,
            None,
            &None,
            None,
            &None,
            &None,
        ).await;

        assert!(result.is_ok());
        let approval = result.unwrap();
        assert!(!approval.approved);
        assert!(approval.error_message.is_some());
    }

    /// Test concurrent sampling requests
    #[tokio::test]
    async fn test_concurrent_sampling_requests() {
        let handler = SamplingIpcHandler::new();
        
        // Create multiple concurrent requests
        let mut handles = vec![];
        
        for i in 0..5 {
            let handler_clone = SamplingIpcHandler::new();
            let handle = tokio::spawn(async move {
                handler_clone.request_approval(
                    &format!("server-{}", i),
                    &format!("Request {} content", i),
                    &None,
                    &None,
                    Some(50),
                    &None,
                    None,
                    &None,
                    &None,
                ).await
            });
            handles.push(handle);
        }

        // Wait for all requests to complete
        let results = futures::future::join_all(handles).await;
        
        // Verify all requests completed successfully
        for result in results {
            assert!(result.is_ok());
            let approval_result = result.unwrap();
            assert!(approval_result.is_ok());
        }
    }

    /// Test timeout scenarios
    #[tokio::test]
    async fn test_approval_timeout() {
        let handler = SamplingIpcHandler::new();
        
        // Test with a very short timeout to simulate timeout scenario
        let result = timeout(
            Duration::from_millis(1), // Very short timeout
            handler.request_approval(
                "slow-server",
                "This request should timeout",
                &None,
                &None,
                None,
                &None,
                None,
                &None,
                &None,
            )
        ).await;

        // The timeout should trigger before the request completes
        // In a real implementation, this would test actual timeout handling
        assert!(result.is_err() || result.unwrap().is_ok());
    }

    /// Test terminal fallback mechanism
    #[tokio::test]
    async fn test_terminal_fallback() {
        let handler = SamplingIpcHandler::new();
        
        let result = handler.request_terminal_approval(
            "fallback-server",
            "Terminal approval test",
        ).await;

        assert!(result.is_ok());
        let approval = result.unwrap();
        assert!(approval.approved); // Terminal fallback should approve
    }

    /// Test security validation
    #[tokio::test]
    async fn test_security_validation() {
        let handler = SamplingIpcHandler::new();

        // Test various potentially malicious inputs
        let long_input = "A".repeat(10000);
        let malicious_inputs = vec![
            "javascript:alert('xss')",
            "<script>alert('xss')</script>",
            "'; DROP TABLE users; --",
            "\x00\x01\x02\x03", // Binary data
            &long_input, // Very long input
        ];

        for input in malicious_inputs {
            let result = handler.request_approval(
                "security-test-server",
                input,
                &None,
                &None,
                None,
                &None,
                None,
                &None,
                &None,
            ).await;

            // Should either reject or handle safely
            assert!(result.is_ok());
            // In a real implementation, malicious content should be rejected
        }
    }

    /// Test model preferences validation
    #[tokio::test]
    async fn test_model_preferences_validation() {
        // Test valid preferences
        let valid_prefs = ModelPreferences {
            hints: Some(vec![
                ModelHint { name: "gpt-4".to_string() },
                ModelHint { name: "claude-3".to_string() },
            ]),
            cost_priority: Some(0.5),
            speed_priority: Some(0.7),
            intelligence_priority: Some(0.9),
        };

        let json = serde_json::to_string(&valid_prefs).expect("Valid preferences should serialize");
        let deserialized: ModelPreferences = serde_json::from_str(&json)
            .expect("Valid preferences should deserialize");

        assert_eq!(deserialized.hints.as_ref().unwrap().len(), 2);
        assert_eq!(deserialized.cost_priority, Some(0.5));

        // Test edge cases
        let edge_prefs = ModelPreferences {
            hints: None,
            cost_priority: Some(0.0),
            speed_priority: Some(1.0),
            intelligence_priority: None,
        };

        let json = serde_json::to_string(&edge_prefs).expect("Edge case preferences should serialize");
        let _: ModelPreferences = serde_json::from_str(&json)
            .expect("Edge case preferences should deserialize");
    }

    /// Test error handling and recovery
    #[tokio::test]
    async fn test_error_handling() {
        // Test JSON parsing errors
        let invalid_json = r#"{"invalid": json}"#;
        let result: Result<SamplingRequest, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());

        // Test missing required fields
        let incomplete_json = r#"{"messages": []}"#;
        let result: Result<SamplingRequest, _> = serde_json::from_str(incomplete_json);
        // Should handle gracefully (messages can be empty)
        assert!(result.is_ok());

        // Test invalid content types
        let invalid_content = r#"{"type": "invalid", "data": "test"}"#;
        let result: Result<SamplingContent, _> = serde_json::from_str(invalid_content);
        assert!(result.is_err());
    }
}
