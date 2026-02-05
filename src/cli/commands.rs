/// Command trait abstraction - Liskov Substitution Principle
/// All CLI commands implement this interface consistently
use anyhow::Result;
use serde_json::Value;

/// Abstraction for all CLI commands
/// Enables easy extension and testing without modification to existing code (Open/Closed)
pub trait CliCommand: Send + Sync {
    /// Execute the command and return JSON result
    fn execute(&self) -> Result<Value>;
}

/// Dependency-injected command execution context
/// Follows Dependency Inversion principle - commands depend on abstractions, not concrete types
pub struct CommandContext {
    pub db_path: String,
}

impl CommandContext {
    pub fn new(db_path: String) -> Self {
        Self { db_path }
    }
}
