use crate::models::plugin::{
    ContextPlugin, PluginConfiguration, PluginContext, PluginEvent, PluginHealth, 
    PluginMetadata, PluginResponse, PluginPermission, ResourceUsage, HealthStatus,
    ContextContribution, ContributedContext,
};
use crate::services::plugin_api::{PluginApiClient, LogLevel};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::Mutex;
use uuid::Uuid;
use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::sync::mpsc;

/// Kiro integration plugin for monitoring spec files and task synchronization
pub struct KiroIntegrationPlugin {
    metadata: PluginMetadata,
    config: Mutex<KiroPluginConfig>,
    api_client: Option<PluginApiClient>,
    spec_directory: Option<PathBuf>,
    file_watcher: Mutex<Option<notify::RecommendedWatcher>>,
    spec_cache: Mutex<HashMap<String, KiroSpec>>,
}

/// Configuration for the Kiro integration plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiroPluginConfig {
    pub spec_directory: String,
    pub auto_sync_tasks: bool,
    pub monitor_file_changes: bool,
    pub validate_specs: bool,
    pub extract_requirements: bool,
    pub track_task_progress: bool,
    pub sync_interval_seconds: u64,
}

/// Kiro specification structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiroSpec {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub requirements: Option<KiroRequirements>,
    pub design: Option<KiroDesign>,
    pub tasks: Option<KiroTasks>,
    pub last_modified: DateTime<Utc>,
}

/// Kiro requirements structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiroRequirements {
    pub introduction: String,
    pub requirements: Vec<KiroRequirement>,
}

/// Individual requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiroRequirement {
    pub id: String,
    pub title: String,
    pub user_story: String,
    pub acceptance_criteria: Vec<String>,
}

/// Kiro design structure  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiroDesign {
    pub overview: String,
    pub architecture: String,
    pub components: String,
    pub data_models: String,
}

/// Kiro tasks structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiroTasks {
    pub tasks: Vec<KiroTask>,
}

/// Individual task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiroTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub subtasks: Vec<KiroTask>,
    pub requirements: Vec<String>,
}

/// Task status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    NotStarted,
    InProgress,
    Completed,
}i
mpl Default for KiroPluginConfig {
    fn default() -> Self {
        Self {
            spec_directory: ".kiro/specs".to_string(),
            auto_sync_tasks: true,
            monitor_file_changes: true,
            validate_specs: true,
            extract_requirements: true,
            track_task_progress: true,
            sync_interval_seconds: 30,
        }
    }
}

impl KiroIntegrationPlugin {
    /// Create a new Kiro integration plugin
    pub fn new() -> Self {
        let metadata = PluginMetadata {
            id: Uuid::new_v4(),
            name: "Kiro Integration".to_string(),
            version: "1.0.0".to_string(),
            description: "Integrates with Kiro spec system for automatic spec parsing and task synchronization".to_string(),
            author: "Context Server".to_string(),
            homepage: None,
            repository: None,
            license: "MIT".to_string(),
            keywords: vec!["kiro".to_string(), "specs".to_string(), "tasks".to_string()],
            dependencies: vec![],
            permissions: vec![
                PluginPermission::ReadContext,
                PluginPermission::WriteContext,
                PluginPermission::FileSystemRead,
                PluginPermission::FileSystemWrite,
            ],
            configuration_schema: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        Self {
            metadata,
            config: Mutex::new(KiroPluginConfig::default()),
            api_client: None,
            spec_directory: None,
            file_watcher: Mutex::new(None),
            spec_cache: Mutex::new(HashMap::new()),
        }
    }

    /// Initialize file watching for spec directory
    async fn setup_file_watcher(&self) -> Result<()> {
        let config = self.config.lock().await;
        if !config.monitor_file_changes {
            return Ok(());
        }

        let spec_dir = self.spec_directory.as_ref()
            .ok_or_else(|| anyhow!("Spec directory not set"))?;

        if !spec_dir.exists() {
            return Err(anyhow!("Spec directory does not exist: {}", spec_dir.display()));
        }

        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx)?;
        watcher.watch(spec_dir, RecursiveMode::Recursive)?;

        // Store the watcher
        let mut file_watcher = self.file_watcher.lock().await;
        *file_watcher = Some(watcher);

        // Spawn a task to handle file events
        let api_client = self.api_client.clone();
        tokio::spawn(async move {
            while let Ok(event) = rx.recv() {
                if let Ok(event) = event {
                    if let Some(client) = &api_client {
                        let _ = Self::handle_file_event(client, event).await;
                    }
                }
            }
        });

        Ok(())
    }

    /// Handle file system events
    async fn handle_file_event(client: &PluginApiClient, event: Event) -> Result<()> {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                for path in event.paths {
                    if path.extension().and_then(|s| s.to_str()) == Some("md") {
                        client.log(LogLevel::Info, &format!("Spec file changed: {}", path.display())).await?;
                        // TODO: Re-parse and update context
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}    //
/ Discover and parse Kiro specs in the directory
    async fn discover_specs(&self) -> Result<Vec<KiroSpec>> {
        let spec_dir = self.spec_directory.as_ref()
            .ok_or_else(|| anyhow!("Spec directory not set"))?;

        let mut specs = Vec::new();
        let mut entries = fs::read_dir(spec_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                // Check if this directory contains Kiro spec files
                if let Ok(spec) = self.parse_spec_directory(&path).await {
                    specs.push(spec);
                }
            }
        }

        Ok(specs)
    }

    /// Parse a Kiro spec directory
    async fn parse_spec_directory(&self, spec_path: &Path) -> Result<KiroSpec> {
        let spec_name = spec_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut spec = KiroSpec {
            id: Uuid::new_v4().to_string(),
            name: spec_name,
            path: spec_path.to_path_buf(),
            requirements: None,
            design: None,
            tasks: None,
            last_modified: Utc::now(),
        };

        // Parse requirements.md
        let requirements_path = spec_path.join("requirements.md");
        if requirements_path.exists() {
            if let Ok(content) = fs::read_to_string(&requirements_path).await {
                spec.requirements = self.parse_requirements(&content).ok();
            }
        }

        // Parse design.md
        let design_path = spec_path.join("design.md");
        if design_path.exists() {
            if let Ok(content) = fs::read_to_string(&design_path).await {
                spec.design = self.parse_design(&content).ok();
            }
        }

        // Parse tasks.md
        let tasks_path = spec_path.join("tasks.md");
        if tasks_path.exists() {
            if let Ok(content) = fs::read_to_string(&tasks_path).await {
                spec.tasks = self.parse_tasks(&content).ok();
            }
        }

        Ok(spec)
    }

    /// Parse requirements from markdown content
    fn parse_requirements(&self, content: &str) -> Result<KiroRequirements> {
        let lines: Vec<&str> = content.lines().collect();
        let mut introduction = String::new();
        let mut requirements = Vec::new();
        let mut current_requirement: Option<KiroRequirement> = None;
        let mut in_introduction = true;

        for line in lines {
            if line.starts_with("## Requirements") {
                in_introduction = false;
                continue;
            }

            if in_introduction && line.starts_with("## Introduction") {
                continue;
            }

            if in_introduction {
                if !line.trim().is_empty() {
                    introduction.push_str(line);
                    introduction.push('\n');
                }
                continue;
            }

            // Parse requirements
            if line.starts_with("### Requirement") {
                // Save previous requirement
                if let Some(req) = current_requirement.take() {
                    requirements.push(req);
                }

                // Extract requirement title
                let title = line.trim_start_matches("### Requirement").trim();
                current_requirement = Some(KiroRequirement {
                    id: Uuid::new_v4().to_string(),
                    title: title.to_string(),
                    user_story: String::new(),
                    acceptance_criteria: Vec::new(),
                });
            } else if line.starts_with("**User Story:**") {
                if let Some(ref mut req) = current_requirement {
                    req.user_story = line.trim_start_matches("**User Story:**").trim().to_string();
                }
            } else if line.trim().starts_with("1. ") || line.trim().starts_with("2. ") {
                // Acceptance criteria
                if let Some(ref mut req) = current_requirement {
                    req.acceptance_criteria.push(line.trim().to_string());
                }
            }
        }

        // Save last requirement
        if let Some(req) = current_requirement {
            requirements.push(req);
        }

        Ok(KiroRequirements {
            introduction: introduction.trim().to_string(),
            requirements,
        })
    }

    /// Parse design from markdown content
    fn parse_design(&self, content: &str) -> Result<KiroDesign> {
        let mut overview = String::new();
        let mut architecture = String::new();
        let mut components = String::new();
        let mut data_models = String::new();
        
        let mut current_section = "";
        
        for line in content.lines() {
            if line.starts_with("## Overview") {
                current_section = "overview";
                continue;
            } else if line.starts_with("## Architecture") {
                current_section = "architecture";
                continue;
            } else if line.starts_with("## Components") {
                current_section = "components";
                continue;
            } else if line.starts_with("## Data Models") {
                current_section = "data_models";
                continue;
            }
            
            match current_section {
                "overview" => {
                    overview.push_str(line);
                    overview.push('\n');
                }
                "architecture" => {
                    architecture.push_str(line);
                    architecture.push('\n');
                }
                "components" => {
                    components.push_str(line);
                    components.push('\n');
                }
                "data_models" => {
                    data_models.push_str(line);
                    data_models.push('\n');
                }
                _ => {}
            }
        }
        
        Ok(KiroDesign {
            overview: overview.trim().to_string(),
            architecture: architecture.trim().to_string(),
            components: components.trim().to_string(),
            data_models: data_models.trim().to_string(),
        })
    }

    /// Parse tasks from markdown content
    fn parse_tasks(&self, content: &str) -> Result<KiroTasks> {
        let mut tasks = Vec::new();
        let mut current_task: Option<KiroTask> = None;
        let mut task_stack: Vec<KiroTask> = Vec::new();
        
        for line in content.lines() {
            // Check for task items
            if let Some(captures) = regex::Regex::new(r"^- \[([ x-])\] (.+)$")?.captures(line) {
                let status_char = captures.get(1).unwrap().as_str();
                let task_text = captures.get(2).unwrap().as_str();
                
                let status = match status_char {
                    "x" => TaskStatus::Completed,
                    "-" => TaskStatus::InProgress,
                    _ => TaskStatus::NotStarted,
                };
                
                // Save previous task
                if let Some(task) = current_task.take() {
                    if task_stack.is_empty() {
                        tasks.push(task);
                    } else {
                        // This is a subtask
                        if let Some(parent) = task_stack.last_mut() {
                            parent.subtasks.push(task);
                        }
                    }
                }
                
                current_task = Some(KiroTask {
                    id: Uuid::new_v4().to_string(),
                    title: task_text.to_string(),
                    description: String::new(),
                    status,
                    subtasks: Vec::new(),
                    requirements: Vec::new(),
                });
            }
            // Check for task descriptions or requirements
            else if line.trim().starts_with("- ") && current_task.is_some() {
                if let Some(ref mut task) = current_task {
                    if line.contains("_Requirements:") {
                        // Extract requirement references
                        let req_text = line.trim_start_matches("- ").trim();
                        task.requirements.push(req_text.to_string());
                    } else {
                        // Task description
                        task.description.push_str(line.trim_start_matches("- "));
                        task.description.push('\n');
                    }
                }
            }
        }
        
        // Save last task
        if let Some(task) = current_task {
            tasks.push(task);
        }
        
        Ok(KiroTasks { tasks })
    }

    /// Convert Kiro specs to context contributions
    async fn specs_to_context(&self, specs: &[KiroSpec]) -> Result<Vec<ContributedContext>> {
        let mut contexts = Vec::new();
        
        for spec in specs {
            // Create context for requirements
            if let Some(ref requirements) = spec.requirements {
                for req in &requirements.requirements {
                    let mut metadata = HashMap::new();
                    metadata.insert("spec_id".to_string(), serde_json::Value::String(spec.id.clone()));
                    metadata.insert("spec_name".to_string(), serde_json::Value::String(spec.name.clone()));
                    metadata.insert("requirement_id".to_string(), serde_json::Value::String(req.id.clone()));
                    
                    contexts.push(ContributedContext {
                        id: Uuid::new_v4().to_string(),
                        context_type: "business_rule".to_string(),
                        title: format!("Requirement: {}", req.title),
                        content: format!("{}\n\nAcceptance Criteria:\n{}", 
                            req.user_story,
                            req.acceptance_criteria.join("\n")
                        ),
                        tags: vec!["kiro".to_string(), "requirement".to_string(), spec.name.clone()],
                        confidence: 0.9,
                        source: "KiroIntegrationPlugin".to_string(),
                        metadata,
                    });
                }
            }
            
            // Create context for design
            if let Some(ref design) = spec.design {
                let mut metadata = HashMap::new();
                metadata.insert("spec_id".to_string(), serde_json::Value::String(spec.id.clone()));
                metadata.insert("spec_name".to_string(), serde_json::Value::String(spec.name.clone()));
                
                contexts.push(ContributedContext {
                    id: Uuid::new_v4().to_string(),
                    context_type: "architectural_decision".to_string(),
                    title: format!("Design: {}", spec.name),
                    content: format!("Overview:\n{}\n\nArchitecture:\n{}\n\nComponents:\n{}", 
                        design.overview, design.architecture, design.components),
                    tags: vec!["kiro".to_string(), "design".to_string(), spec.name.clone()],
                    confidence: 0.9,
                    source: "KiroIntegrationPlugin".to_string(),
                    metadata,
                });
            }
            
            // Create context for tasks
            if let Some(ref tasks) = spec.tasks {
                for task in &tasks.tasks {
                    let mut metadata = HashMap::new();
                    metadata.insert("spec_id".to_string(), serde_json::Value::String(spec.id.clone()));
                    metadata.insert("spec_name".to_string(), serde_json::Value::String(spec.name.clone()));
                    metadata.insert("task_id".to_string(), serde_json::Value::String(task.id.clone()));
                    metadata.insert("task_status".to_string(), serde_json::Value::String(format!("{:?}", task.status)));
                    
                    contexts.push(ContributedContext {
                        id: Uuid::new_v4().to_string(),
                        context_type: "performance_requirement".to_string(),
                        title: format!("Task: {}", task.title),
                        content: format!("Status: {:?}\n\nDescription:\n{}\n\nRequirements: {}", 
                            task.status, task.description, task.requirements.join(", ")),
                        tags: vec!["kiro".to_string(), "task".to_string(), spec.name.clone()],
                        confidence: 0.8,
                        source: "KiroIntegrationPlugin".to_string(),
                        metadata,
                    });
                }
            }
        }
        
        Ok(contexts)
    }

    /// Sync specs with context server
    async fn sync_specs(&self) -> Result<()> {
        if let Some(client) = &self.api_client {
            client.log(LogLevel::Info, "Starting Kiro spec synchronization").await?;
            
            let specs = self.discover_specs().await?;
            client.log(LogLevel::Info, &format!("Found {} Kiro specs", specs.len())).await?;
            
            // Update cache
            {
                let mut cache = self.spec_cache.lock().await;
                cache.clear();
                for spec in &specs {
                    cache.insert(spec.id.clone(), spec.clone());
                }
            }
            
            // Convert to context and contribute
            let contexts = self.specs_to_context(&specs).await?;
            
            if !contexts.is_empty() {
                let contribution = ContextContribution {
                    context_items: contexts,
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("source".to_string(), serde_json::Value::String("kiro_integration".to_string()));
                        meta.insert("sync_time".to_string(), serde_json::Value::String(Utc::now().to_rfc3339()));
                        meta
                    },
                };
                
                match client.contribute_context(contribution).await {
                    Ok(created_ids) => {
                        client.log(LogLevel::Info, &format!("Successfully synced {} context items from Kiro specs", created_ids.len())).await?;
                    }
                    Err(e) => {
                        client.log(LogLevel::Error, &format!("Failed to sync Kiro specs: {}", e)).await?;
                    }
                }
            }
        }
        
        Ok(())
    }

#[async_trait]
impl ContextPlugin for KiroIntegrationPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&mut self, context: PluginContext) -> Result<()> {
        // Set up API client (simplified for now)
        self.api_client = Some(PluginApiClient::new(context.instance_id, 
            std::sync::Arc::new(crate::services::DefaultPluginApi::new(
                std::sync::Arc::new(crate::services::context_crud_service::ContextCrudServiceImpl::new(
                    crate::infrastructure::SqliteBusinessRuleRepository::new(std::sync::Arc::new(std::sync::Mutex::new(rusqlite::Connection::open_in_memory()?))),
                    crate::infrastructure::SqliteArchitecturalDecisionRepository::new(std::sync::Arc::new(std::sync::Mutex::new(rusqlite::Connection::open_in_memory()?))),
                    crate::infrastructure::SqlitePerformanceRequirementRepository::new(std::sync::Arc::new(std::sync::Mutex::new(rusqlite::Connection::open_in_memory()?))),
                )),
                std::sync::Arc::new(crate::services::context_query_service::ContextQueryServiceImpl::new(
                    crate::infrastructure::SqliteBusinessRuleRepository::new(std::sync::Arc::new(std::sync::Mutex::new(rusqlite::Connection::open_in_memory()?))),
                    crate::infrastructure::SqliteArchitecturalDecisionRepository::new(std::sync::Arc::new(std::sync::Mutex::new(rusqlite::Connection::open_in_memory()?))),
                    crate::infrastructure::SqlitePerformanceRequirementRepository::new(std::sync::Arc::new(std::sync::Mutex::new(rusqlite::Connection::open_in_memory()?))),
                )),
                std::sync::Arc::new(crate::services::DefaultPluginSecurity::new(std::time::Duration::from_secs(60))),
                std::sync::Arc::new(crate::services::project_service::ProjectServiceImpl::new(
                    crate::infrastructure::SqliteProjectRepository::new(std::sync::Arc::new(std::sync::Mutex::new(rusqlite::Connection::open_in_memory()?))),
                )),
            ))
        ));

        // Load configuration
        if let Some(spec_dir) = context.configuration.settings.get("spec_directory") {
            if let Some(dir_str) = spec_dir.as_str() {
                let mut config = self.config.lock().await;
                config.spec_directory = dir_str.to_string();
                self.spec_directory = Some(PathBuf::from(dir_str));
            }
        }

        // Load other configuration settings
        let mut config = self.config.lock().await;
        if let Some(auto_sync) = context.configuration.settings.get("auto_sync_tasks") {
            if let Some(auto_sync_bool) = auto_sync.as_bool() {
                config.auto_sync_tasks = auto_sync_bool;
            }
        }

        if let Some(monitor_files) = context.configuration.settings.get("monitor_file_changes") {
            if let Some(monitor_bool) = monitor_files.as_bool() {
                config.monitor_file_changes = monitor_bool;
            }
        }

        drop(config);

        // Set up file watcher
        if let Err(e) = self.setup_file_watcher().await {
            if let Some(client) = &self.api_client {
                client.log(LogLevel::Warn, &format!("Failed to setup file watcher: {}", e)).await?;
            }
        }

        // Initial sync
        if let Err(e) = self.sync_specs().await {
            if let Some(client) = &self.api_client {
                client.log(LogLevel::Error, &format!("Failed initial spec sync: {}", e)).await?;
            }
        }

        Ok(())
    }

    async fn handle_event(&self, event: PluginEvent) -> Result<PluginResponse> {
        match event {
            PluginEvent::SystemStartup => {
                if let Some(client) = &self.api_client {
                    client.log(LogLevel::Info, "Kiro Integration Plugin started").await?;
                }
                
                // Sync specs on startup
                if let Err(e) = self.sync_specs().await {
                    if let Some(client) = &self.api_client {
                        client.log(LogLevel::Error, &format!("Failed to sync specs on startup: {}", e)).await?;
                    }
                }
                
                Ok(PluginResponse::EventHandled)
            }
            PluginEvent::Custom { event_type, .. } => {
                if event_type == "kiro_sync" {
                    if let Err(e) = self.sync_specs().await {
                        return Ok(PluginResponse::Error(format!("Failed to sync Kiro specs: {}", e)));
                    }
                    Ok(PluginResponse::Success)
                } else {
                    Ok(PluginResponse::EventIgnored)
                }
            }
            _ => Ok(PluginResponse::EventIgnored),
        }
    }

    async fn provide_context(&self, query: &str, project_id: &str) -> Result<Option<ContextContribution>> {
        // Check if query is related to Kiro specs
        let kiro_keywords = ["spec", "requirement", "task", "design", "kiro"];
        let query_lower = query.to_lowercase();
        
        if !kiro_keywords.iter().any(|&keyword| query_lower.contains(keyword)) {
            return Ok(None);
        }

        let cache = self.spec_cache.lock().await;
        let mut contexts = Vec::new();

        for spec in cache.values() {
            // Check if spec matches query
            if spec.name.to_lowercase().contains(&query_lower) {
                // Add relevant context from this spec
                if let Some(ref requirements) = spec.requirements {
                    for req in &requirements.requirements {
                        if req.title.to_lowercase().contains(&query_lower) ||
                           req.user_story.to_lowercase().contains(&query_lower) {
                            
                            let mut metadata = HashMap::new();
                            metadata.insert("spec_id".to_string(), serde_json::Value::String(spec.id.clone()));
                            metadata.insert("spec_name".to_string(), serde_json::Value::String(spec.name.clone()));
                            metadata.insert("project_id".to_string(), serde_json::Value::String(project_id.to_string()));

                            contexts.push(ContributedContext {
                                id: Uuid::new_v4().to_string(),
                                context_type: "kiro_requirement".to_string(),
                                title: format!("Kiro Requirement: {}", req.title),
                                content: format!("User Story: {}\n\nAcceptance Criteria:\n{}", 
                                    req.user_story,
                                    req.acceptance_criteria.join("\n")
                                ),
                                tags: vec!["kiro".to_string(), "requirement".to_string(), spec.name.clone()],
                                confidence: 0.9,
                                source: "KiroIntegrationPlugin".to_string(),
                                metadata,
                            });
                        }
                    }
                }
            }
        }

        if contexts.is_empty() {
            Ok(None)
        } else {
            Ok(Some(ContextContribution {
                context_items: contexts,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("source".to_string(), serde_json::Value::String("kiro_integration".to_string()));
                    meta.insert("query".to_string(), serde_json::Value::String(query.to_string()));
                    meta
                },
            }))
        }
    }

    async fn shutdown(&mut self) -> Result<()> {
        if let Some(client) = &self.api_client {
            client.log(LogLevel::Info, "Kiro Integration Plugin shutting down").await?;
        }
        
        // Stop file watcher
        let mut watcher = self.file_watcher.lock().await;
        *watcher = None;
        
        Ok(())
    }

    async fn health_check(&self) -> Result<PluginHealth> {
        let mut status = HealthStatus::Healthy;
        let mut message = None;

        // Check if spec directory exists
        if let Some(spec_dir) = &self.spec_directory {
            if !spec_dir.exists() {
                status = HealthStatus::Critical;
                message = Some("Spec directory does not exist".to_string());
            } else {
                // Check if we can read the directory
                match fs::read_dir(spec_dir).await {
                    Ok(_) => {
                        status = HealthStatus::Healthy;
                        message = Some("Spec directory is accessible".to_string());
                    }
                    Err(e) => {
                        status = HealthStatus::Warning;
                        message = Some(format!("Cannot read spec directory: {}", e));
                    }
                }
            }
        } else {
            status = HealthStatus::Warning;
            message = Some("Spec directory not configured".to_string());
        }

        Ok(PluginHealth {
            status,
            message,
            last_check: Utc::now(),
            resource_usage: ResourceUsage {
                memory_mb: 15.0, // Estimated memory usage
                cpu_percent: 2.0, // Low CPU usage
                network_requests_count: 0,
                file_operations_count: 2, // File system operations
                last_updated: Utc::now(),
            },
        })
    }
}