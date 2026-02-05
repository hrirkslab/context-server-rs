use crate::models::plugin::{PluginInstanceId, PluginStatus};
use crate::services::PluginManager;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Plugin error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginError {
    /// Plugin initialization failed
    InitializationError {
        instance_id: PluginInstanceId,
        error: String,
        timestamp: DateTime<Utc>,
    },
    /// Plugin execution error
    ExecutionError {
        instance_id: PluginInstanceId,
        operation: String,
        error: String,
        timestamp: DateTime<Utc>,
    },
    /// Plugin timeout error
    TimeoutError {
        instance_id: PluginInstanceId,
        operation: String,
        timeout_duration: Duration,
        timestamp: DateTime<Utc>,
    },
    /// Plugin resource limit exceeded
    ResourceLimitError {
        instance_id: PluginInstanceId,
        resource_type: String,
        limit: f64,
        actual: f64,
        timestamp: DateTime<Utc>,
    },
    /// Plugin permission denied
    PermissionError {
        instance_id: PluginInstanceId,
        operation: String,
        required_permission: String,
        timestamp: DateTime<Utc>,
    },
    /// Plugin communication error
    CommunicationError {
        instance_id: PluginInstanceId,
        error: String,
        timestamp: DateTime<Utc>,
    },
}

impl PluginError {
    /// Get the plugin instance ID associated with this error
    pub fn instance_id(&self) -> PluginInstanceId {
        match self {
            PluginError::InitializationError { instance_id, .. } |
            PluginError::ExecutionError { instance_id, .. } |
            PluginError::TimeoutError { instance_id, .. } |
            PluginError::ResourceLimitError { instance_id, .. } |
            PluginError::PermissionError { instance_id, .. } |
            PluginError::CommunicationError { instance_id, .. } => *instance_id,
        }
    }
    
    /// Get the timestamp of this error
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            PluginError::InitializationError { timestamp, .. } |
            PluginError::ExecutionError { timestamp, .. } |
            PluginError::TimeoutError { timestamp, .. } |
            PluginError::ResourceLimitError { timestamp, .. } |
            PluginError::PermissionError { timestamp, .. } |
            PluginError::CommunicationError { timestamp, .. } => *timestamp,
        }
    }
    
    /// Get a human-readable description of the error
    pub fn description(&self) -> String {
        match self {
            PluginError::InitializationError { error, .. } => {
                format!("Plugin initialization failed: {}", error)
            }
            PluginError::ExecutionError { operation, error, .. } => {
                format!("Plugin execution error in '{}': {}", operation, error)
            }
            PluginError::TimeoutError { operation, timeout_duration, .. } => {
                format!("Plugin operation '{}' timed out after {:?}", operation, timeout_duration)
            }
            PluginError::ResourceLimitError { resource_type, limit, actual, .. } => {
                format!("Plugin exceeded {} limit: {} > {}", resource_type, actual, limit)
            }
            PluginError::PermissionError { operation, required_permission, .. } => {
                format!("Plugin lacks permission '{}' for operation '{}'", required_permission, operation)
            }
            PluginError::CommunicationError { error, .. } => {
                format!("Plugin communication error: {}", error)
            }
        }
    }
}

/// Error recovery strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Restart the plugin
    Restart,
    /// Disable the plugin temporarily
    Disable { duration: Option<Duration> },
    /// Reset plugin configuration to defaults
    ResetConfiguration,
    /// Reduce resource limits
    ReduceResourceLimits { factor: f64 },
    /// Retry the operation
    Retry { max_attempts: u32, delay: Duration },
    /// Ignore the error
    Ignore,
    /// Manual intervention required
    Manual,
}

/// Error recovery result
#[derive(Debug, Clone)]
pub enum RecoveryResult {
    Success,
    Failed(String),
    RequiresManualIntervention,
}

/// Plugin error handler trait
#[async_trait]
pub trait PluginErrorHandler: Send + Sync {
    /// Handle a plugin error
    async fn handle_error(&self, error: PluginError) -> Result<RecoveryResult>;
    
    /// Get error history for a plugin
    async fn get_error_history(&self, instance_id: PluginInstanceId) -> Result<Vec<PluginError>>;
    
    /// Get error statistics
    async fn get_error_stats(&self) -> Result<ErrorStatistics>;
    
    /// Clear error history for a plugin
    async fn clear_error_history(&self, instance_id: PluginInstanceId) -> Result<()>;
    
    /// Set recovery strategy for an error type
    async fn set_recovery_strategy(&self, error_pattern: ErrorPattern, strategy: RecoveryStrategy) -> Result<()>;
    
    /// Get recovery strategy for an error
    async fn get_recovery_strategy(&self, error: &PluginError) -> Result<RecoveryStrategy>;
}

/// Error pattern for matching errors to recovery strategies
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ErrorPattern {
    InitializationError,
    ExecutionError { operation: Option<String> },
    TimeoutError { operation: Option<String> },
    ResourceLimitError { resource_type: Option<String> },
    PermissionError { operation: Option<String> },
    CommunicationError,
    Any,
}

impl ErrorPattern {
    /// Check if this pattern matches the given error
    pub fn matches(&self, error: &PluginError) -> bool {
        match (self, error) {
            (ErrorPattern::InitializationError, PluginError::InitializationError { .. }) => true,
            (ErrorPattern::ExecutionError { operation: pattern_op }, PluginError::ExecutionError { operation, .. }) => {
                pattern_op.as_ref().map_or(true, |p| p == operation)
            }
            (ErrorPattern::TimeoutError { operation: pattern_op }, PluginError::TimeoutError { operation, .. }) => {
                pattern_op.as_ref().map_or(true, |p| p == operation)
            }
            (ErrorPattern::ResourceLimitError { resource_type: pattern_type }, PluginError::ResourceLimitError { resource_type, .. }) => {
                pattern_type.as_ref().map_or(true, |p| p == resource_type)
            }
            (ErrorPattern::PermissionError { operation: pattern_op }, PluginError::PermissionError { operation, .. }) => {
                pattern_op.as_ref().map_or(true, |p| p == operation)
            }
            (ErrorPattern::CommunicationError, PluginError::CommunicationError { .. }) => true,
            (ErrorPattern::Any, _) => true,
            _ => false,
        }
    }
}

/// Error statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStatistics {
    pub total_errors: u64,
    pub errors_by_type: HashMap<String, u64>,
    pub errors_by_plugin: HashMap<PluginInstanceId, u64>,
    pub recovery_success_rate: f64,
    pub most_common_errors: Vec<(String, u64)>,
}

/// Default implementation of plugin error handler
pub struct DefaultPluginErrorHandler {
    plugin_manager: Arc<dyn PluginManager>,
    error_history: Arc<RwLock<HashMap<PluginInstanceId, Vec<PluginError>>>>,
    recovery_strategies: Arc<RwLock<HashMap<ErrorPattern, RecoveryStrategy>>>,
    error_stats: Arc<RwLock<ErrorStatistics>>,
}

impl DefaultPluginErrorHandler {
    /// Create a new plugin error handler
    pub fn new(plugin_manager: Arc<dyn PluginManager>) -> Self {
        let mut default_strategies = HashMap::new();
        
        // Set default recovery strategies
        default_strategies.insert(ErrorPattern::InitializationError, RecoveryStrategy::Restart);
        default_strategies.insert(ErrorPattern::ExecutionError { operation: None }, RecoveryStrategy::Retry { max_attempts: 3, delay: Duration::from_secs(1) });
        default_strategies.insert(ErrorPattern::TimeoutError { operation: None }, RecoveryStrategy::Retry { max_attempts: 2, delay: Duration::from_secs(2) });
        default_strategies.insert(ErrorPattern::ResourceLimitError { resource_type: None }, RecoveryStrategy::ReduceResourceLimits { factor: 0.8 });
        default_strategies.insert(ErrorPattern::PermissionError { operation: None }, RecoveryStrategy::Manual);
        default_strategies.insert(ErrorPattern::CommunicationError, RecoveryStrategy::Restart);
        
        Self {
            plugin_manager,
            error_history: Arc::new(RwLock::new(HashMap::new())),
            recovery_strategies: Arc::new(RwLock::new(default_strategies)),
            error_stats: Arc::new(RwLock::new(ErrorStatistics {
                total_errors: 0,
                errors_by_type: HashMap::new(),
                errors_by_plugin: HashMap::new(),
                recovery_success_rate: 0.0,
                most_common_errors: Vec::new(),
            })),
        }
    }
    
    /// Record an error in the history
    async fn record_error(&self, error: PluginError) {
        let instance_id = error.instance_id();
        
        // Add to error history
        {
            let mut history = self.error_history.write().await;
            let plugin_errors = history.entry(instance_id).or_insert_with(Vec::new);
            plugin_errors.push(error.clone());
            
            // Keep only the last 100 errors per plugin
            if plugin_errors.len() > 100 {
                plugin_errors.remove(0);
            }
        }
        
        // Update statistics
        {
            let mut stats = self.error_stats.write().await;
            stats.total_errors += 1;
            
            let error_type = match &error {
                PluginError::InitializationError { .. } => "InitializationError",
                PluginError::ExecutionError { .. } => "ExecutionError",
                PluginError::TimeoutError { .. } => "TimeoutError",
                PluginError::ResourceLimitError { .. } => "ResourceLimitError",
                PluginError::PermissionError { .. } => "PermissionError",
                PluginError::CommunicationError { .. } => "CommunicationError",
            };
            
            *stats.errors_by_type.entry(error_type.to_string()).or_insert(0) += 1;
            *stats.errors_by_plugin.entry(instance_id).or_insert(0) += 1;
        }
    }
    
    /// Execute a recovery strategy
    async fn execute_recovery(&self, instance_id: PluginInstanceId, strategy: RecoveryStrategy) -> Result<RecoveryResult> {
        match strategy {
            RecoveryStrategy::Restart => {
                match self.plugin_manager.stop_plugin(instance_id).await {
                    Ok(()) => {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        match self.plugin_manager.start_plugin(instance_id).await {
                            Ok(()) => Ok(RecoveryResult::Success),
                            Err(e) => Ok(RecoveryResult::Failed(format!("Failed to restart plugin: {}", e))),
                        }
                    }
                    Err(e) => Ok(RecoveryResult::Failed(format!("Failed to stop plugin: {}", e))),
                }
            }
            RecoveryStrategy::Disable { duration } => {
                match self.plugin_manager.stop_plugin(instance_id).await {
                    Ok(()) => {
                        if let Some(duration) = duration {
                            // Schedule re-enabling after the duration
                            let plugin_manager = self.plugin_manager.clone();
                            tokio::spawn(async move {
                                tokio::time::sleep(duration).await;
                                let _ = plugin_manager.start_plugin(instance_id).await;
                            });
                        }
                        Ok(RecoveryResult::Success)
                    }
                    Err(e) => Ok(RecoveryResult::Failed(format!("Failed to disable plugin: {}", e))),
                }
            }
            RecoveryStrategy::ResetConfiguration => {
                // This would require access to the plugin configuration service
                Ok(RecoveryResult::RequiresManualIntervention)
            }
            RecoveryStrategy::ReduceResourceLimits { factor: _ } => {
                // This would require access to the plugin security service
                Ok(RecoveryResult::RequiresManualIntervention)
            }
            RecoveryStrategy::Retry { max_attempts: _, delay: _ } => {
                // Retry logic would be handled by the caller
                Ok(RecoveryResult::Success)
            }
            RecoveryStrategy::Ignore => {
                Ok(RecoveryResult::Success)
            }
            RecoveryStrategy::Manual => {
                Ok(RecoveryResult::RequiresManualIntervention)
            }
        }
    }
}

#[async_trait]
impl PluginErrorHandler for DefaultPluginErrorHandler {
    async fn handle_error(&self, error: PluginError) -> Result<RecoveryResult> {
        // Record the error
        self.record_error(error.clone()).await;
        
        // Get recovery strategy
        let strategy = self.get_recovery_strategy(&error).await?;
        
        // Execute recovery
        let result = self.execute_recovery(error.instance_id(), strategy).await?;
        
        // Update recovery success rate
        {
            let mut stats = self.error_stats.write().await;
            let success = matches!(result, RecoveryResult::Success);
            let total_recoveries = stats.total_errors;
            let current_success_rate = stats.recovery_success_rate;
            
            // Update running average
            stats.recovery_success_rate = if total_recoveries == 1 {
                if success { 1.0 } else { 0.0 }
            } else {
                let weight = 1.0 / total_recoveries as f64;
                current_success_rate * (1.0 - weight) + if success { weight } else { 0.0 }
            };
        }
        
        Ok(result)
    }
    
    async fn get_error_history(&self, instance_id: PluginInstanceId) -> Result<Vec<PluginError>> {
        let history = self.error_history.read().await;
        Ok(history.get(&instance_id).cloned().unwrap_or_default())
    }
    
    async fn get_error_stats(&self) -> Result<ErrorStatistics> {
        let stats = self.error_stats.read().await;
        Ok(stats.clone())
    }
    
    async fn clear_error_history(&self, instance_id: PluginInstanceId) -> Result<()> {
        let mut history = self.error_history.write().await;
        history.remove(&instance_id);
        Ok(())
    }
    
    async fn set_recovery_strategy(&self, error_pattern: ErrorPattern, strategy: RecoveryStrategy) -> Result<()> {
        let mut strategies = self.recovery_strategies.write().await;
        strategies.insert(error_pattern, strategy);
        Ok(())
    }
    
    async fn get_recovery_strategy(&self, error: &PluginError) -> Result<RecoveryStrategy> {
        let strategies = self.recovery_strategies.read().await;
        
        // Find the most specific matching pattern
        for (pattern, strategy) in strategies.iter() {
            if pattern.matches(error) {
                return Ok(strategy.clone());
            }
        }
        
        // Default to manual intervention if no pattern matches
        Ok(RecoveryStrategy::Manual)
    }
}

/// Error recovery coordinator that manages error handling across all plugins
pub struct ErrorRecoveryCoordinator {
    error_handler: Arc<dyn PluginErrorHandler>,
    active_recoveries: Arc<RwLock<HashMap<PluginInstanceId, Uuid>>>,
}

impl ErrorRecoveryCoordinator {
    /// Create a new error recovery coordinator
    pub fn new(error_handler: Arc<dyn PluginErrorHandler>) -> Self {
        Self {
            error_handler,
            active_recoveries: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Handle an error with coordination to prevent duplicate recovery attempts
    pub async fn handle_error(&self, error: PluginError) -> Result<RecoveryResult> {
        let instance_id = error.instance_id();
        let recovery_id = Uuid::new_v4();
        
        // Check if recovery is already in progress
        {
            let mut active = self.active_recoveries.write().await;
            if active.contains_key(&instance_id) {
                return Ok(RecoveryResult::Failed("Recovery already in progress".to_string()));
            }
            active.insert(instance_id, recovery_id);
        }
        
        // Handle the error
        let result = self.error_handler.handle_error(error).await;
        
        // Remove from active recoveries
        {
            let mut active = self.active_recoveries.write().await;
            active.remove(&instance_id);
        }
        
        result
    }
    
    /// Get active recovery operations
    pub async fn get_active_recoveries(&self) -> Vec<PluginInstanceId> {
        let active = self.active_recoveries.read().await;
        active.keys().cloned().collect()
    }
}