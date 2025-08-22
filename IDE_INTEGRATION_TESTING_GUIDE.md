# IDE Integration Plugin Testing Guide

This guide provides multiple approaches to test the IDE Integration Plugin functionality.

## 1. Unit Tests (Already Implemented)

The plugin includes comprehensive unit tests that you can run:

```bash
# Run all IDE integration plugin tests
cargo test ide_integration_plugin --lib

# Run with verbose output
cargo test ide_integration_plugin --lib -- --nocapture
```

### Current Unit Tests Cover:
- Plugin initialization
- Language detection for multiple file types
- File analysis and context extraction
- Event handling (system startup, file changes, custom events)
- Context suggestions management
- Health checks and resource monitoring
- Plugin metadata validation

## 2. Integration Testing with Plugin Framework

### Create a Test Plugin Instance

```rust
// Add this to src/services/plugins/ide_integration_plugin.rs for testing
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::services::{DefaultPluginService, DefaultPluginApi};
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_full_plugin_lifecycle() {
        // Create a temporary workspace
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path();
        
        // Create test files
        let rust_file = workspace_path.join("test.rs");
        tokio::fs::write(&rust_file, r#"
/// This is a test function
pub fn hello_world() -> String {
    "Hello, World!".to_string()
}

pub struct TestStruct {
    pub field: String,
}
"#).await.unwrap();

        // Initialize plugin
        let mut plugin = IdeIntegrationPlugin::new();
        let context = create_test_context(workspace_path.to_str().unwrap());
        plugin.initialize(context).await.unwrap();
        
        // Test file analysis
        let analysis = plugin.analyze_file(&rust_file).await.unwrap();
        assert!(!analysis.extracted_contexts.is_empty());
        
        // Test event handling
        let mut event_data = HashMap::new();
        event_data.insert("file_path".to_string(), 
            serde_json::Value::String(rust_file.to_string_lossy().to_string()));
        
        let file_event = PluginEvent::Custom {
            event_type: "file_changed".to_string(),
            data: serde_json::Value::Object(event_data.into_iter().collect()),
        };
        
        let response = plugin.handle_event(file_event).await.unwrap();
        assert!(matches!(response, PluginResponse::Success));
    }
}
```

## 3. Manual Testing with Context Server

### Step 1: Start the Context Server

```bash
# Build and run the context server
cargo build --release
./target/release/context-server-rs
```

### Step 2: Create Test Configuration

Create a plugin configuration file `ide_plugin_config.json`:

```json
{
  "workspace_path": "./test_workspace",
  "supported_languages": ["rust", "typescript", "javascript", "python"],
  "auto_analyze_on_save": true,
  "real_time_suggestions": true,
  "context_extraction_enabled": true,
  "debugging_integration": true,
  "file_watch_patterns": [
    "**/*.rs",
    "**/*.ts",
    "**/*.js",
    "**/*.py"
  ],
  "ignore_patterns": [
    "**/target/**",
    "**/node_modules/**",
    "**/.git/**"
  ]
}
```

### Step 3: Test File Analysis

Create test files in your workspace:

**test_workspace/example.rs:**
```rust
/// This function calculates the fibonacci sequence
pub fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

pub struct Calculator {
    pub memory: f64,
}

impl Calculator {
    pub fn new() -> Self {
        Self { memory: 0.0 }
    }
}
```

**test_workspace/example.ts:**
```typescript
/**
 * User management service
 */
export class UserService {
    private users: User[] = [];
    
    async createUser(userData: CreateUserRequest): Promise<User> {
        // Implementation here
        return {} as User;
    }
}

interface User {
    id: string;
    name: string;
    email: string;
}
```

## 4. Event-Based Testing

### Test File Change Events

```rust
// Create a test that simulates IDE file change events
#[tokio::test]
async fn test_file_change_events() {
    let plugin = setup_test_plugin().await;
    
    // Simulate file opened event
    let open_event = PluginEvent::Custom {
        event_type: "file_opened".to_string(),
        data: json!({
            "file_path": "test.rs",
            "language": "rust"
        }),
    };
    
    let response = plugin.handle_event(open_event).await.unwrap();
    assert!(matches!(response, PluginResponse::EventHandled));
    
    // Simulate file changed event
    let change_event = PluginEvent::Custom {
        event_type: "file_changed".to_string(),
        data: json!({
            "file_path": "test.rs",
            "changes": ["added function", "modified struct"]
        }),
    };
    
    let response = plugin.handle_event(change_event).await.unwrap();
    assert!(matches!(response, PluginResponse::Success));
}
```

## 5. Context Suggestion Testing

### Test Suggestion Generation

```rust
#[tokio::test]
async fn test_context_suggestions() {
    let plugin = setup_test_plugin().await;
    
    // Create a file with missing documentation
    let test_file = create_test_file_with_undocumented_function().await;
    
    // Analyze the file
    plugin.process_file_change(&test_file.to_string_lossy()).await.unwrap();
    
    // Check for suggestions
    let suggestions = plugin.get_suggestions_for_file(&test_file.to_string_lossy()).await;
    
    // Should suggest adding documentation
    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().any(|s| 
        matches!(s.suggestion_type, SuggestionType::MissingDocumentation)
    ));
}
```

## 6. Performance Testing

### Test Large File Analysis

```rust
#[tokio::test]
async fn test_large_file_performance() {
    let plugin = setup_test_plugin().await;
    
    // Create a large test file
    let large_file = create_large_rust_file(1000).await; // 1000 functions
    
    let start = std::time::Instant::now();
    let analysis = plugin.analyze_file(&large_file).await.unwrap();
    let duration = start.elapsed();
    
    // Should complete within reasonable time
    assert!(duration.as_secs() < 5);
    assert!(!analysis.extracted_contexts.is_empty());
}
```

## 7. Real-World IDE Integration Testing

### Using with VS Code Extension (Conceptual)

If you were to integrate with VS Code, you'd create an extension that:

1. **Monitors File Changes:**
```javascript
// VS Code extension code
vscode.workspace.onDidSaveTextDocument(async (document) => {
    // Send file_changed event to context server
    await sendEventToContextServer({
        event_type: "file_changed",
        data: {
            file_path: document.fileName,
            content: document.getText(),
            language: document.languageId
        }
    });
});
```

2. **Displays Context Suggestions:**
```javascript
// Get suggestions from plugin
const suggestions = await getContextSuggestions(document.fileName);

// Display as VS Code quick fixes or code actions
suggestions.forEach(suggestion => {
    const codeAction = new vscode.CodeAction(
        suggestion.title, 
        vscode.CodeActionKind.QuickFix
    );
    // Add to code actions provider
});
```

## 8. Health and Monitoring Tests

### Test Plugin Health

```bash
# Check plugin health via API
curl -X GET http://localhost:8080/api/plugins/health/ide-integration

# Expected response:
{
  "status": "Healthy",
  "message": "Workspace is accessible",
  "last_check": "2024-01-15T10:30:00Z",
  "resource_usage": {
    "memory_mb": 25.5,
    "cpu_percent": 2.0,
    "file_operations_count": 15
  }
}
```

## 9. Error Handling Tests

### Test Invalid File Scenarios

```rust
#[tokio::test]
async fn test_error_handling() {
    let plugin = setup_test_plugin().await;
    
    // Test non-existent file
    let result = plugin.analyze_file(Path::new("nonexistent.rs")).await;
    assert!(result.is_err());
    
    // Test binary file
    let binary_file = create_binary_file().await;
    let result = plugin.analyze_file(&binary_file).await;
    // Should handle gracefully without crashing
}
```

## 10. Running All Tests

```bash
# Run all plugin-related tests
cargo test plugin --lib

# Run with coverage (if you have cargo-tarpaulin installed)
cargo tarpaulin --out Html --output-dir coverage

# Run integration tests
cargo test --test integration_tests

# Run performance benchmarks (if implemented)
cargo bench
```

## Expected Test Results

When running the tests, you should see:

✅ **Language Detection**: Correctly identifies file types  
✅ **Code Analysis**: Extracts functions, classes, and patterns  
✅ **Context Generation**: Creates meaningful context entries  
✅ **Suggestion Engine**: Provides relevant suggestions  
✅ **Event Handling**: Responds to IDE events properly  
✅ **Health Monitoring**: Reports plugin status accurately  
✅ **Error Recovery**: Handles edge cases gracefully  

## Troubleshooting

If tests fail, check:

1. **File Permissions**: Ensure test files can be read/written
2. **Workspace Path**: Verify the workspace directory exists
3. **Dependencies**: Make sure all required crates are installed
4. **Plugin Configuration**: Check that config values are valid
5. **Resource Limits**: Ensure sufficient memory/CPU for analysis

This comprehensive testing approach ensures the IDE Integration Plugin works correctly in various scenarios and can handle real-world usage patterns.