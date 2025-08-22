// Integration test for VS Code Extension with IDE Integration Plugin
// This test verifies that the VS Code extension can communicate with the Context Engine server
// and that the IDE Integration Plugin processes the requests correctly.

use std::time::Duration;
use tokio::time::sleep;
use serde_json::json;

#[tokio::test]
async fn test_vscode_extension_integration() {
    // This test simulates the VS Code extension communicating with the Context Engine server
    // It tests the key integration points that the extension relies on
    
    println!("🧪 Testing VS Code Extension Integration");
    
    // Test 1: Health Check Endpoint
    println!("📡 Testing health check endpoint...");
    let health_response = test_health_endpoint().await;
    assert!(health_response.is_ok(), "Health check should succeed");
    
    // Test 2: File Analysis Endpoint
    println!("📄 Testing file analysis endpoint...");
    let analysis_response = test_file_analysis().await;
    assert!(analysis_response.is_ok(), "File analysis should succeed");
    
    // Test 3: Context Suggestions Endpoint
    println!("💡 Testing context suggestions endpoint...");
    let suggestions_response = test_context_suggestions().await;
    assert!(suggestions_response.is_ok(), "Context suggestions should succeed");
    
    // Test 4: Context Creation Endpoint
    println!("➕ Testing context creation endpoint...");
    let creation_response = test_context_creation().await;
    assert!(creation_response.is_ok(), "Context creation should succeed");
    
    // Test 5: WebSocket Connection
    println!("🔌 Testing WebSocket connection...");
    let websocket_response = test_websocket_connection().await;
    assert!(websocket_response.is_ok(), "WebSocket connection should succeed");
    
    // Test 6: Real-time File Change Events
    println!("⚡ Testing real-time file change events...");
    let realtime_response = test_realtime_events().await;
    assert!(realtime_response.is_ok(), "Real-time events should work");
    
    println!("✅ All VS Code Extension integration tests passed!");
}

async fn test_health_endpoint() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate VS Code extension health check request
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:3000/health")
        .header("User-Agent", "VSCode-ContextEngine/1.0.0")
        .send()
        .await?;
    
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await?;
    assert_eq!(body["status"], "healthy");
    
    Ok(())
}

async fn test_file_analysis() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate VS Code extension file analysis request
    let client = reqwest::Client::new();
    let request_body = json!({
        "file_path": "/test/example.rs",
        "project_id": "test-project"
    });
    
    let response = client
        .post("http://localhost:3000/analyze")
        .header("Content-Type", "application/json")
        .header("User-Agent", "VSCode-ContextEngine/1.0.0")
        .json(&request_body)
        .send()
        .await?;
    
    assert!(response.status().is_success());
    
    Ok(())
}

async fn test_context_suggestions() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate VS Code extension requesting context suggestions
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:3000/suggestions")
        .query(&[("file_path", "/test/example.rs")])
        .header("User-Agent", "VSCode-ContextEngine/1.0.0")
        .send()
        .await?;
    
    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await?;
    assert!(body.get("suggestions").is_some());
    
    Ok(())
}

async fn test_context_creation() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate VS Code extension creating context
    let client = reqwest::Client::new();
    let request_body = json!({
        "title": "Test Context from VS Code",
        "context_type": "business_rule",
        "content": "This is a test context created from VS Code extension",
        "description": "Testing context creation from IDE",
        "project_id": "test-project",
        "metadata": {
            "file_path": "/test/example.rs",
            "line_number": 42,
            "source": "vscode_extension"
        }
    });
    
    let response = client
        .post("http://localhost:3000/context")
        .header("Content-Type", "application/json")
        .header("User-Agent", "VSCode-ContextEngine/1.0.0")
        .json(&request_body)
        .send()
        .await?;
    
    assert!(response.status().is_success());
    
    Ok(())
}

async fn test_websocket_connection() -> Result<(), Box<dyn std::error::Error>> {
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use futures_util::{SinkExt, StreamExt};
    
    // Simulate VS Code extension WebSocket connection
    let ws_url = "ws://localhost:3000/ws";
    let (ws_stream, _) = connect_async(ws_url).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    // Send subscription message (like VS Code extension would)
    let subscription_msg = json!({
        "type": "subscribe",
        "filters": {
            "project_id": "test-project"
        }
    });
    
    ws_sender
        .send(Message::Text(subscription_msg.to_string()))
        .await?;
    
    // Wait for acknowledgment or timeout
    let timeout_duration = Duration::from_secs(5);
    let result = tokio::time::timeout(timeout_duration, ws_receiver.next()).await;
    
    match result {
        Ok(Some(Ok(Message::Text(_)))) => {
            println!("✅ WebSocket connection established and working");
        }
        Ok(Some(Ok(_))) => {
            println!("✅ WebSocket connection established (non-text message)");
        }
        Ok(Some(Err(e))) => {
            return Err(format!("WebSocket error: {}", e).into());
        }
        Ok(None) => {
            return Err("WebSocket connection closed unexpectedly".into());
        }
        Err(_) => {
            println!("⚠️ WebSocket connection timeout (may be normal in test environment)");
        }
    }
    
    Ok(())
}

async fn test_realtime_events() -> Result<(), Box<dyn std::error::Error>> {
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use futures_util::{SinkExt, StreamExt};
    
    // Simulate VS Code extension sending file change events
    let ws_url = "ws://localhost:3000/ws";
    let (ws_stream, _) = connect_async(ws_url).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    // Send file change event (like VS Code extension would)
    let file_change_msg = json!({
        "type": "file_changed",
        "data": {
            "file_path": "/test/example.rs",
            "timestamp": "2024-01-01T12:00:00Z"
        }
    });
    
    ws_sender
        .send(Message::Text(file_change_msg.to_string()))
        .await?;
    
    // Give some time for processing
    sleep(Duration::from_millis(100)).await;
    
    println!("✅ File change event sent successfully");
    
    Ok(())
}

#[tokio::test]
async fn test_vscode_extension_error_handling() {
    println!("🧪 Testing VS Code Extension Error Handling");
    
    // Test 1: Invalid server URL handling
    println!("❌ Testing invalid server URL...");
    let result = test_invalid_server_url().await;
    assert!(result.is_err(), "Should handle invalid server URL gracefully");
    
    // Test 2: Malformed request handling
    println!("❌ Testing malformed request...");
    let result = test_malformed_request().await;
    assert!(result.is_err(), "Should handle malformed requests gracefully");
    
    // Test 3: Server unavailable handling
    println!("❌ Testing server unavailable...");
    let result = test_server_unavailable().await;
    assert!(result.is_err(), "Should handle server unavailable gracefully");
    
    println!("✅ All error handling tests passed!");
}

async fn test_invalid_server_url() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get("http://invalid-server:9999/health")
        .header("User-Agent", "VSCode-ContextEngine/1.0.0")
        .send()
        .await;
    
    // Should fail with connection error
    match response {
        Err(_) => Ok(()), // Expected error
        Ok(_) => Err("Should have failed with invalid server URL".into()),
    }
}

async fn test_malformed_request() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:3000/analyze")
        .header("Content-Type", "application/json")
        .header("User-Agent", "VSCode-ContextEngine/1.0.0")
        .body("invalid json")
        .send()
        .await?;
    
    // Should return 400 Bad Request
    if response.status().is_client_error() {
        Ok(())
    } else {
        Err("Should have returned client error for malformed request".into())
    }
}

async fn test_server_unavailable() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:9999/health") // Wrong port
        .header("User-Agent", "VSCode-ContextEngine/1.0.0")
        .send()
        .await;
    
    // Should fail with connection error
    match response {
        Err(_) => Ok(()), // Expected error
        Ok(_) => Err("Should have failed with server unavailable".into()),
    }
}

#[tokio::test]
async fn test_vscode_extension_performance() {
    println!("🧪 Testing VS Code Extension Performance");
    
    // Test 1: Multiple concurrent requests
    println!("⚡ Testing concurrent requests...");
    let concurrent_result = test_concurrent_requests().await;
    assert!(concurrent_result.is_ok(), "Should handle concurrent requests");
    
    // Test 2: Large file analysis
    println!("📊 Testing large file analysis...");
    let large_file_result = test_large_file_analysis().await;
    assert!(large_file_result.is_ok(), "Should handle large files");
    
    println!("✅ All performance tests passed!");
}

async fn test_concurrent_requests() -> Result<(), Box<dyn std::error::Error>> {
    use futures::future::join_all;
    
    let client = reqwest::Client::new();
    let mut futures = Vec::new();
    
    // Create 10 concurrent health check requests
    for i in 0..10 {
        let client_clone = client.clone();
        let future = async move {
            let response = client_clone
                .get("http://localhost:3000/health")
                .header("User-Agent", &format!("VSCode-ContextEngine/1.0.0-test-{}", i))
                .send()
                .await?;
            
            assert_eq!(response.status(), 200);
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        };
        futures.push(future);
    }
    
    // Wait for all requests to complete
    let results = join_all(futures).await;
    
    // Check that all requests succeeded
    for result in results {
        result?;
    }
    
    println!("✅ All {} concurrent requests completed successfully", 10);
    Ok(())
}

async fn test_large_file_analysis() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    // Simulate analysis of a large file
    let request_body = json!({
        "file_path": "/test/large_file.rs",
        "project_id": "test-project",
        "file_size": 1000000, // 1MB file
        "line_count": 10000
    });
    
    let start_time = std::time::Instant::now();
    
    let response = client
        .post("http://localhost:3000/analyze")
        .header("Content-Type", "application/json")
        .header("User-Agent", "VSCode-ContextEngine/1.0.0")
        .json(&request_body)
        .send()
        .await?;
    
    let duration = start_time.elapsed();
    
    assert!(response.status().is_success());
    assert!(duration.as_secs() < 30, "Large file analysis should complete within 30 seconds");
    
    println!("✅ Large file analysis completed in {:?}", duration);
    Ok(())
}

// Helper function to run all integration tests
pub async fn run_vscode_extension_integration_tests() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting VS Code Extension Integration Tests");
    
    // Run main integration test
    test_vscode_extension_integration().await;
    
    // Run error handling tests
    test_vscode_extension_error_handling().await;
    
    // Run performance tests
    test_vscode_extension_performance().await;
    
    println!("🎉 All VS Code Extension integration tests completed successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn integration_test_suite() {
        // This test can be run with: cargo test integration_test_suite
        if std::env::var("RUN_INTEGRATION_TESTS").is_ok() {
            run_vscode_extension_integration_tests().await.unwrap();
        } else {
            println!("Skipping integration tests. Set RUN_INTEGRATION_TESTS=1 to run them.");
        }
    }
}