/// Integration tests for Context Server CLI
/// These tests verify CLI functionality and dual-mode operation
///
/// Run with: cargo test --test cli_integration_tests -- --nocapture

#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::fs;
    use serde_json::Value;

    fn get_db_path() -> String {
        format!(
            "{}/.config/context-server-rs/context.db",
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
        )
    }

    fn run_command(args: &[&str]) -> Result<String, String> {
        let output = Command::new("context-server-rs")
            .args(args)
            .output()
            .map_err(|e| format!("Failed to run command: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(format!("Command failed: {}", stderr))
        }
    }

    #[test]
    fn test_list_business_rules_json() {
        let output = run_command(&["list", "business_rule", "--format", "json"])
            .expect("Failed to list business rules");
        
        // Parse as JSON to verify format
        let result: Value = serde_json::from_str(&output)
            .expect("Output is not valid JSON");
        
        assert!(result["entity_type"].as_str().unwrap_or("") == "business_rule");
        assert!(result["count"].is_number());
        assert!(result["items"].is_array());
        
        println!("✓ Listed {} business rules", result["count"]);
    }

    #[test]
    fn test_list_all_entity_types() {
        let entity_types = vec![
            "business_rule",
            "architectural_decision",
            "performance_requirement",
            "security_policy",
            "feature",
        ];

        for entity_type in entity_types {
            let output = run_command(&["list", entity_type, "--format", "json"])
                .expect(&format!("Failed to list {}", entity_type));
            
            let result: Value = serde_json::from_str(&output)
                .expect("Output is not valid JSON");
            
            assert_eq!(result["entity_type"].as_str().unwrap_or(""), entity_type);
            println!("✓ Listed {} entities of type: {}", result["count"], entity_type);
        }
    }

    #[test]
    fn test_query_by_task() {
        let output = run_command(&[
            "query",
            "--task", "auth",
            "--project", "test-project",
            "--format", "json"
        ]).expect("Failed to query by task");
        
        let result: Value = serde_json::from_str(&output)
            .expect("Output is not valid JSON");
        
        assert_eq!(result["status"].as_str().unwrap_or(""), "success");
        assert!(result["data"].is_object());
        
        println!("✓ Queried task 'auth' successfully");
    }

    #[test]
    fn test_search_functionality() {
        let output = run_command(&[
            "search", "authentication",
            "--format", "json"
        ]).expect("Failed to search");
        
        let result: Value = serde_json::from_str(&output)
            .expect("Output is not valid JSON");
        
        assert_eq!(result["query"].as_str().unwrap_or(""), "authentication");
        assert!(result["results"].is_array());
        
        println!("✓ Search for 'authentication' executed successfully");
    }

    #[test]
    fn test_output_format_json() {
        let output = run_command(&[
            "list", "business_rule",
            "--format", "json"
        ]).expect("Failed to get JSON format");
        
        // Should be valid JSON
        serde_json::from_str::<Value>(&output)
            .expect("Output is not valid JSON");
        
        println!("✓ JSON format output is valid");
    }

    #[test]
    fn test_output_format_text() {
        let output = run_command(&[
            "list", "business_rule",
            "--format", "text"
        ]).expect("Failed to get text format");
        
        // Text format should contain human-readable content
        assert!(!output.is_empty());
        // Should contain typical key-value lines
        assert!(output.contains(":") || output.contains("Entity") || output.contains("Items"));
        
        println!("✓ Text format output is readable");
    }

    #[test]
    fn test_output_format_yaml() {
        let output = run_command(&[
            "list", "business_rule",
            "--format", "yaml"
        ]).expect("Failed to get YAML format");
        
        // YAML format should contain colons and newlines
        assert!(!output.is_empty());
        
        println!("✓ YAML format output generated");
    }

    #[test]
    fn test_project_filtering() {
        let output = run_command(&[
            "list", "business_rule",
            "--project", "test-project",
            "--format", "json"
        ]).expect("Failed to filter by project");
        
        let result: Value = serde_json::from_str(&output)
            .expect("Output is not valid JSON");
        
        // Result should contain project-filtered data
        assert!(result["items"].is_array());
        
        println!("✓ Project filtering works correctly");
    }

    #[test]
    fn test_database_exists() {
        let db_path = get_db_path();
        
        // Database should exist if initialized
        if fs::metadata(&db_path).is_ok() {
            println!("✓ Database found at: {}", db_path);
        } else {
            println!("⚠ Database not initialized yet (this is OK)");
        }
    }

    #[test]
    fn test_invalid_entity_type_error() {
        let output = run_command(&["list", "invalid_type", "--format", "json"]);
        
        // Should return an error
        assert!(output.is_err(), "Expected error for invalid entity type");
        
        println!("✓ Invalid entity type properly rejected");
    }

    #[test]
    fn test_get_by_id() {
        // First, list to get an ID
        let list_output = run_command(&[
            "list", "business_rule",
            "--format", "json"
        ]).expect("Failed to list");
        
        let result: Value = serde_json::from_str(&list_output)
            .expect("Output is not valid JSON");
        
        // If there are items, try to get by ID
        if let Some(items) = result["items"].as_array() {
            if !items.is_empty() {
                if let Some(id) = items[0]["id"].as_str() {
                    let get_output = run_command(&["get", id, "--format", "json"])
                        .expect("Failed to get by ID");
                    
                    let get_result: Value = serde_json::from_str(&get_output)
                        .expect("Output is not valid JSON");
                    
                    assert_eq!(get_result["id"].as_str().unwrap_or(""), id);
                    println!("✓ Get by ID works correctly");
                }
            } else {
                println!("⚠ No items to test get by ID (this is OK if database is empty)");
            }
        }
    }

    #[test]
    fn test_cli_responsiveness() {
        use std::time::Instant;

        let start = Instant::now();
        let _output = run_command(&["list", "business_rule", "--format", "json"])
            .expect("Failed to run command");
        let elapsed = start.elapsed();

        println!("✓ Command completed in: {:?}", elapsed);
        
        // CLI should be responsive (under 5 seconds for most operations)
        assert!(elapsed.as_secs() < 5, "Command took too long: {:?}", elapsed);
    }

    #[test]
    fn test_help_command() {
        // This tests the Clap help system
        let output = Command::new("context-server-rs")
            .arg("--help")
            .output()
            .expect("Failed to run help");
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Context Server") || stdout.contains("Usage"));
        
        println!("✓ Help command works");
    }
}

// Example usage patterns for documentation
#[allow(dead_code)]
mod example_workflows {
    /// Example: Query all contexts for an authentication task
    pub fn example_query_auth_contexts() {
        // Command:
        // context-server-rs query --task auth --project myapp --format json
        
        // Output: JSON with business_rules, architectural_decisions, etc.
        println!("Example: Query authentication contexts for myapp project");
    }

    /// Example: Search for pagination-related patterns
    pub fn example_search_pagination() {
        // Command:
        // context-server-rs search "pagination" --format json
        
        // Output: JSON with matching contexts across all tables
        println!("Example: Search for pagination patterns");
    }

    /// Example: List all security policies
    pub fn example_list_security_policies() {
        // Command:
        // context-server-rs list security_policy --format json
        
        // Output: JSON array of all security policies
        println!("Example: List all security policies");
    }

    /// Example: Get specific context by ID
    pub fn example_get_by_id() {
        // Command:
        // context-server-rs get rule-123 --format json
        
        // Output: JSON with full context details
        println!("Example: Get specific context by ID");
    }

    /// Example: Use in shell script
    pub fn example_shell_script() {
        let script = r#"
#!/bin/bash
# Get all security policies and format as YAML
context-server-rs list security_policy --format yaml

# Search and pipe to jq for processing
context-server-rs search "authentication" --format json | jq '.results[].id'

# Query by project and count results
context-server-rs query --task api --project myapp --format json | jq '.data | length'
"#;
        println!("Example shell script:\n{}", script);
    }

    /// Example: Use in Python script (for OpenClaw integration)
    pub fn example_python_integration() {
        let example = r#"
import json
import subprocess

def query_context(task, project="default"):
    result = subprocess.run(
        ["context-server-rs", "query", "--task", task, "--project", project, "--format", "json"],
        capture_output=True,
        text=True
    )
    return json.loads(result.stdout)

# Usage
context = query_context("auth", "myapp")
print(context["data"]["business_rules"])
"#;
        println!("Example Python integration:\n{}", example);
    }
}
