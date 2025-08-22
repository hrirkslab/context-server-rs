use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use async_trait::async_trait;
use anyhow::Result;

/// Unique identifier for a plugin
pub type PluginId = Uuid;

/// Unique identifier for a plugin instance
pub type PluginInstanceId = Uuid;

/// Plugin metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: PluginId,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: String,
    pub keywords: Vec<String>,
    pub dependencies: Vec<PluginDependency>,
    pub permissions: Vec<PluginPermission>,
    pub configuration_schema: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Plugin dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub name: String,
    pub version_requirement: String,
    pub optional: bool,
}

/// Plugin permission types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginPermission {
    ReadContext,
    WriteContext,
    DeleteContext,
    ReadProject,
    WriteProject,
    NetworkAccess,
    FileSystemRead,
    FileSystemWrite,
    ExecuteCommands,
    AccessDatabase,
    Custom(String),
}

/// Plugin lifecycle status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginStatus {
    Unloaded,
    Loading,
    Loaded,
    Initializing,
    Active,
    Paused,
    Error(String),
    Unloading,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfiguration {
    pub enabled: bool,
    pub auto_start: bool,
    pub settings: HashMap<String, serde_json::Value>,
    pub resource_limits: ResourceLimits,
}

/// Resource limits for plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: Option<u64>,
    pub max_cpu_percent: Option<f64>,
    pub max_execution_time: Option<Duration>,
    pub max_network_requests_per_minute: Option<u32>,
    pub max_file_operations_per_minute: Option<u32>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: Some(256),
            max_cpu_percent: Some(10.0),
            max_execution_time: Some(Duration::from_secs(30)),
            max_network_requests_per_minute: Some(100),
            max_file_operations_per_minute: Some(50),
        }
    }
}

/// Plugin instance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstance {
    pub instance_id: PluginInstanceId,
    pub plugin_id: PluginId,
    pub metadata: PluginMetadata,
    pub configuration: PluginConfiguration,
    pub status: PluginStatus,
    pub last_error: Option<String>,
    pub resource_usage: ResourceUsage,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

/// Current resource usage of a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub memory_mb: f64,
    pub cpu_percent: f64,
    pub network_requests_count: u32,
    pub file_operations_count: u32,
    pub last_updated: DateTime<Utc>,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            memory_mb: 0.0,
            cpu_percent: 0.0,
            network_requests_count: 0,
            file_operations_count: 0,
            last_updated: Utc::now(),
        }
    }
}

/// Context for plugin initialization and execution
#[derive(Debug, Clone)]
pub struct PluginContext {
    pub instance_id: PluginInstanceId,
    pub configuration: PluginConfiguration,
    pub data_directory: std::path::PathBuf,
    pub temp_directory: std::path::PathBuf,
    pub api_client: PluginApiClient,
}

/// API client for plugins to interact with the context server
#[derive(Debug, Clone)]
pub struct PluginApiClient {
    // This will be implemented to provide safe access to context server APIs
    pub(crate) _private: (),
}

/// Events that plugins can listen to and respond to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginEvent {
    ContextCreated {
        context_id: String,
        context_type: String,
        project_id: String,
    },
    ContextUpdated {
        context_id: String,
        context_type: String,
        project_id: String,
        changes: Vec<String>,
    },
    ContextDeleted {
        context_id: String,
        context_type: String,
        project_id: String,
    },
    ProjectCreated {
        project_id: String,
        project_name: String,
    },
    ProjectUpdated {
        project_id: String,
        changes: Vec<String>,
    },
    QueryExecuted {
        query: String,
        results_count: usize,
        execution_time_ms: u64,
    },
    SystemStartup,
    SystemShutdown,
    Custom {
        event_type: String,
        data: serde_json::Value,
    },
}

/// Response from a plugin to an event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginResponse {
    Success,
    Error(String),
    ContextContribution(ContextContribution),
    EventHandled,
    EventIgnored,
}

/// Context contribution from a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextContribution {
    pub context_items: Vec<ContributedContext>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Context item contributed by a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributedContext {
    pub id: String,
    pub context_type: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub confidence: f64,
    pub source: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Main plugin trait that all plugins must implement
#[async_trait]
pub trait ContextPlugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;
    
    /// Initialize the plugin with the given context
    async fn initialize(&mut self, context: PluginContext) -> Result<()>;
    
    /// Handle an event from the context server
    async fn handle_event(&self, event: PluginEvent) -> Result<PluginResponse>;
    
    /// Provide context for a given query (optional)
    async fn provide_context(&self, query: &str, project_id: &str) -> Result<Option<ContextContribution>>;
    
    /// Shutdown the plugin gracefully
    async fn shutdown(&mut self) -> Result<()>;
    
    /// Health check for the plugin
    async fn health_check(&self) -> Result<PluginHealth>;
}

/// Plugin health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHealth {
    pub status: HealthStatus,
    pub message: Option<String>,
    pub last_check: DateTime<Utc>,
    pub resource_usage: ResourceUsage,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Plugin registry entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRegistryEntry {
    pub metadata: PluginMetadata,
    pub download_url: String,
    pub checksum: String,
    pub verified: bool,
    pub downloads: u64,
    pub rating: f64,
    pub reviews_count: u32,
    pub published_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Plugin marketplace search filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSearchFilters {
    pub query: Option<String>,
    pub category: Option<String>,
    pub author: Option<String>,
    pub verified_only: bool,
    pub min_rating: Option<f64>,
    pub sort_by: PluginSortBy,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Plugin sorting options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginSortBy {
    Name,
    Downloads,
    Rating,
    UpdatedAt,
    PublishedAt,
}

impl Default for PluginSearchFilters {
    fn default() -> Self {
        Self {
            query: None,
            category: None,
            author: None,
            verified_only: false,
            min_rating: None,
            sort_by: PluginSortBy::Downloads,
            limit: Some(20),
            offset: Some(0),
        }
    }
}