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
use std::process::Stdio;
use tokio::process::Command;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Git integration plugin for monitoring repository changes
pub struct GitIntegrationPlugin {
    metadata: PluginMetadata,
    config: Mutex<GitPluginConfig>,
    api_client: Option<PluginApiClient>,
    repository_path: Option<PathBuf>,
    last_commit_hash: Mutex<Option<String>>,
}

/// Configuration for the Git integration plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitPluginConfig {
    pub repository_path: String,
    pub watch_branches: Vec<String>,
    pub auto_update_context: bool,
    pub commit_message_analysis: bool,
    pub file_change_tracking: bool,
    pub ignore_patterns: Vec<String>,
    pub context_extraction_rules: Vec<ContextExtractionRule>,
}

/// Rule for extracting context from Git changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextExtractionRule {
    pub name: String,
    pub file_pattern: String, // regex pattern
    pub change_type: GitChangeType,
    pub context_type: String,
    pub extraction_method: ExtractionMethod,
}

/// Type of Git change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GitChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Any,
}

/// Method for extracting context from changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtractionMethod {
    CommitMessage,
    FileContent,
    FileDiff,
    FileName,
    Custom(String),
}

/// Git commit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub author: String,
    pub email: String,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub files_changed: Vec<GitFileChange>,
}

/// Information about a changed file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFileChange {
    pub path: String,
    pub change_type: GitChangeType,
    pub additions: u32,
    pub deletions: u32,
    pub diff: Option<String>,
}

impl Default for GitPluginConfig {
    fn default() -> Self {
        Self {
            repository_path: ".".to_string(),
            watch_branches: vec!["main".to_string(), "master".to_string(), "develop".to_string()],
            auto_update_context: true,
            commit_message_analysis: true,
            file_change_tracking: true,
            ignore_patterns: vec![
                r"\.git/.*".to_string(),
                r"node_modules/.*".to_string(),
                r"target/.*".to_string(),
                r"\.DS_Store".to_string(),
            ],
            context_extraction_rules: vec![
                ContextExtractionRule {
                    name: "README Updates".to_string(),
                    file_pattern: r"README\.md".to_string(),
                    change_type: GitChangeType::Modified,
                    context_type: "documentation".to_string(),
                    extraction_method: ExtractionMethod::FileContent,
                },
                ContextExtractionRule {
                    name: "Configuration Changes".to_string(),
                    file_pattern: r".*\.(json|yaml|yml|toml|ini)$".to_string(),
                    change_type: GitChangeType::Any,
                    context_type: "configuration".to_string(),
                    extraction_method: ExtractionMethod::FileDiff,
                },
                ContextExtractionRule {
                    name: "Code Architecture".to_string(),
                    file_pattern: r".*\.(rs|ts|js|py|java|cpp|h)$".to_string(),
                    change_type: GitChangeType::Added,
                    context_type: "architectural_decision".to_string(),
                    extraction_method: ExtractionMethod::CommitMessage,
                },
            ],
        }
    }
}

impl GitIntegrationPlugin {
    /// Create a new Git integration plugin
    pub fn new() -> Self {
        let metadata = PluginMetadata {
            id: Uuid::new_v4(),
            name: "Git Integration".to_string(),
            version: "1.0.0".to_string(),
            description: "Monitors Git repository changes and automatically updates context".to_string(),
            author: "Context Server".to_string(),
            homepage: None,
            repository: None,
            license: "MIT".to_string(),
            keywords: vec!["git".to_string(), "vcs".to_string(), "integration".to_string()],
            dependencies: vec![],
            permissions: vec![
                PluginPermission::ReadContext,
                PluginPermission::WriteContext,
                PluginPermission::FileSystemRead,
                PluginPermission::ExecuteCommands,
            ],
            configuration_schema: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        Self {
            metadata,
            config: Mutex::new(GitPluginConfig::default()),
            api_client: None,
            repository_path: None,
            last_commit_hash: Mutex::new(None),
        }
    }

    /// Execute a Git command
    async fn execute_git_command(&self, args: &[&str]) -> Result<String> {
        let repo_path = self.repository_path.as_ref()
            .ok_or_else(|| anyhow!("Repository path not set"))?;

        let output = Command::new("git")
            .args(args)
            .current_dir(repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("Git command failed: {}", error))
        }
    }

    /// Get the current commit hash
    async fn get_current_commit_hash(&self) -> Result<String> {
        let output = self.execute_git_command(&["rev-parse", "HEAD"]).await?;
        Ok(output.trim().to_string())
    }

    /// Get commit information
    async fn get_commit_info(&self, commit_hash: &str) -> Result<GitCommit> {
        // Get commit details
        let commit_info = self.execute_git_command(&[
            "show", "--format=%H|%an|%ae|%at|%s", "--name-status", commit_hash
        ]).await?;

        let lines: Vec<&str> = commit_info.lines().collect();
        if lines.is_empty() {
            return Err(anyhow!("No commit information found"));
        }

        // Parse commit header
        let header_parts: Vec<&str> = lines[0].split('|').collect();
        if header_parts.len() < 5 {
            return Err(anyhow!("Invalid commit format"));
        }

        let timestamp = header_parts[3].parse::<i64>()
            .map_err(|_| anyhow!("Invalid timestamp"))?;

        // Parse file changes
        let mut files_changed = Vec::new();
        for line in lines.iter().skip(1) {
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                let change_type = match parts[0].chars().next() {
                    Some('A') => GitChangeType::Added,
                    Some('M') => GitChangeType::Modified,
                    Some('D') => GitChangeType::Deleted,
                    Some('R') => GitChangeType::Renamed,
                    _ => GitChangeType::Modified,
                };

                files_changed.push(GitFileChange {
                    path: parts[1].to_string(),
                    change_type,
                    additions: 0, // Would need additional git command to get this
                    deletions: 0, // Would need additional git command to get this
                    diff: None,   // Would need additional git command to get this
                });
            }
        }

        Ok(GitCommit {
            hash: header_parts[0].to_string(),
            author: header_parts[1].to_string(),
            email: header_parts[2].to_string(),
            timestamp: DateTime::from_timestamp(timestamp, 0)
                .unwrap_or_else(|| Utc::now()),
            message: header_parts[4].to_string(),
            files_changed,
        })
    }

    /// Check for new commits
    async fn check_for_new_commits(&self) -> Result<Vec<GitCommit>> {
        let current_hash = self.get_current_commit_hash().await?;
        let mut last_hash = self.last_commit_hash.lock().await;

        if let Some(ref last) = *last_hash {
            if last == &current_hash {
                return Ok(vec![]); // No new commits
            }

            // Get commits between last known and current
            let commit_range = format!("{}..{}", last, current_hash);
            let commit_hashes = self.execute_git_command(&["rev-list", &commit_range]).await?;
            
            let mut commits = Vec::new();
            for hash in commit_hashes.lines() {
                if !hash.trim().is_empty() {
                    match self.get_commit_info(hash.trim()).await {
                        Ok(commit) => commits.push(commit),
                        Err(e) => {
                            if let Some(client) = &self.api_client {
                                let _ = client.log(LogLevel::Warn, &format!("Failed to get commit info for {}: {}", hash, e)).await;
                            }
                        }
                    }
                }
            }

            *last_hash = Some(current_hash);
            Ok(commits)
        } else {
            // First time, just store current hash
            *last_hash = Some(current_hash.clone());
            
            // Return the current commit
            match self.get_commit_info(&current_hash).await {
                Ok(commit) => Ok(vec![commit]),
                Err(_) => Ok(vec![]),
            }
        }
    }

    /// Extract context from a commit
    async fn extract_context_from_commit(&self, commit: &GitCommit) -> Result<Vec<ContributedContext>> {
        let config = self.config.lock().await;
        let mut contexts = Vec::new();

        for rule in &config.context_extraction_rules {
            for file_change in &commit.files_changed {
                // Check if file matches pattern
                let regex = regex::Regex::new(&rule.file_pattern)
                    .map_err(|e| anyhow!("Invalid regex pattern: {}", e))?;
                
                if !regex.is_match(&file_change.path) {
                    continue;
                }

                // Check if change type matches
                if !matches!(rule.change_type, GitChangeType::Any) && 
                   std::mem::discriminant(&rule.change_type) != std::mem::discriminant(&file_change.change_type) {
                    continue;
                }

                // Extract context based on method
                let content = match &rule.extraction_method {
                    ExtractionMethod::CommitMessage => commit.message.clone(),
                    ExtractionMethod::FileName => file_change.path.clone(),
                    ExtractionMethod::FileContent => {
                        // Read current file content
                        if let Some(repo_path) = &self.repository_path {
                            let file_path = repo_path.join(&file_change.path);
                            match tokio::fs::read_to_string(&file_path).await {
                                Ok(content) => content,
                                Err(_) => continue, // File might be deleted or binary
                            }
                        } else {
                            continue;
                        }
                    }
                    ExtractionMethod::FileDiff => {
                        // Get file diff
                        match self.execute_git_command(&["show", &format!("{}:{}", commit.hash, file_change.path)]).await {
                            Ok(diff) => diff,
                            Err(_) => continue,
                        }
                    }
                    ExtractionMethod::Custom(_) => {
                        // Custom extraction would be implemented here
                        commit.message.clone()
                    }
                };

                let mut metadata = HashMap::new();
                metadata.insert("commit_hash".to_string(), serde_json::Value::String(commit.hash.clone()));
                metadata.insert("author".to_string(), serde_json::Value::String(commit.author.clone()));
                metadata.insert("file_path".to_string(), serde_json::Value::String(file_change.path.clone()));
                metadata.insert("change_type".to_string(), serde_json::Value::String(format!("{:?}", file_change.change_type)));

                contexts.push(ContributedContext {
                    id: Uuid::new_v4().to_string(),
                    context_type: rule.context_type.clone(),
                    title: format!("{}: {}", rule.name, file_change.path),
                    content,
                    tags: vec![
                        "git".to_string(),
                        "auto-generated".to_string(),
                        commit.author.clone(),
                    ],
                    confidence: 0.7, // Medium confidence for auto-generated content
                    source: "GitIntegrationPlugin".to_string(),
                    metadata,
                });
            }
        }

        Ok(contexts)
    }

    /// Process new commits and update context
    async fn process_new_commits(&self) -> Result<()> {
        if let Some(client) = &self.api_client {
            let commits = self.check_for_new_commits().await?;
            
            for commit in commits {
                client.log(LogLevel::Info, &format!("Processing commit: {} by {}", commit.hash, commit.author)).await?;
                
                let contexts = self.extract_context_from_commit(&commit).await?;
                
                if !contexts.is_empty() {
                    let contribution = ContextContribution {
                        context_items: contexts,
                        metadata: {
                            let mut meta = HashMap::new();
                            meta.insert("source".to_string(), serde_json::Value::String("git_integration".to_string()));
                            meta.insert("commit_hash".to_string(), serde_json::Value::String(commit.hash.clone()));
                            meta
                        },
                    };
                    
                    match client.contribute_context(contribution).await {
                        Ok(created_ids) => {
                            client.log(LogLevel::Info, &format!("Created {} context items from commit {}", created_ids.len(), commit.hash)).await?;
                        }
                        Err(e) => {
                            client.log(LogLevel::Error, &format!("Failed to contribute context from commit {}: {}", commit.hash, e)).await?;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Initialize the Git repository
    async fn initialize_repository(&self, repo_path: &str) -> Result<()> {
        let path = PathBuf::from(repo_path);
        
        // Check if it's a Git repository
        if !path.join(".git").exists() {
            return Err(anyhow!("Not a Git repository: {}", repo_path));
        }

        // Verify Git is available
        let output = Command::new("git")
            .args(&["--version"])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Git is not available"));
        }

        Ok(())
    }
}

#[async_trait]
impl ContextPlugin for GitIntegrationPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&mut self, context: PluginContext) -> Result<()> {
        // Set up API client
        self.api_client = Some(PluginApiClient::new(context.instance_id, 
            // This would need to be passed in from the plugin manager
            // For now, we'll leave it as a placeholder
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

        // Load configuration from context
        if let Some(repo_path) = context.configuration.settings.get("repository_path") {
            if let Some(path_str) = repo_path.as_str() {
                let mut config = self.config.lock().await;
                config.repository_path = path_str.to_string();
                
                // Initialize repository
                self.initialize_repository(path_str).await?;
                self.repository_path = Some(PathBuf::from(path_str));
            }
        }

        // Load other configuration settings
        let mut config = self.config.lock().await;
        if let Some(watch_branches) = context.configuration.settings.get("watch_branches") {
            if let Some(branches_array) = watch_branches.as_array() {
                config.watch_branches = branches_array.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }
        }

        if let Some(auto_update) = context.configuration.settings.get("auto_update_context") {
            if let Some(auto_update_bool) = auto_update.as_bool() {
                config.auto_update_context = auto_update_bool;
            }
        }

        Ok(())
    }

    async fn handle_event(&self, event: PluginEvent) -> Result<PluginResponse> {
        match event {
            PluginEvent::SystemStartup => {
                if let Some(client) = &self.api_client {
                    client.log(LogLevel::Info, "Git Integration Plugin started").await?;
                }
                
                // Initial check for commits
                if let Err(e) = self.process_new_commits().await {
                    if let Some(client) = &self.api_client {
                        client.log(LogLevel::Error, &format!("Failed to process initial commits: {}", e)).await?;
                    }
                }
                
                Ok(PluginResponse::EventHandled)
            }
            PluginEvent::QueryExecuted { .. } => {
                // Check for new commits when queries are executed
                let config = self.config.lock().await;
                if config.auto_update_context {
                    drop(config);
                    if let Err(e) = self.process_new_commits().await {
                        if let Some(client) = &self.api_client {
                            client.log(LogLevel::Warn, &format!("Failed to check for new commits: {}", e)).await?;
                        }
                    }
                }
                Ok(PluginResponse::EventHandled)
            }
            PluginEvent::Custom { event_type, data } => {
                if event_type == "git_check" {
                    if let Err(e) = self.process_new_commits().await {
                        return Ok(PluginResponse::Error(format!("Failed to check Git commits: {}", e)));
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
        // Check if query is related to Git or version control
        let git_keywords = ["git", "commit", "version", "history", "change", "diff"];
        let query_lower = query.to_lowercase();
        
        if !git_keywords.iter().any(|&keyword| query_lower.contains(keyword)) {
            return Ok(None);
        }

        // Get recent commits that might be relevant
        let commits = self.check_for_new_commits().await.unwrap_or_default();
        let mut contexts = Vec::new();

        for commit in commits.iter().take(5) { // Limit to 5 most recent commits
            if commit.message.to_lowercase().contains(&query_lower) ||
               commit.files_changed.iter().any(|f| f.path.to_lowercase().contains(&query_lower)) {
                
                let mut metadata = HashMap::new();
                metadata.insert("commit_hash".to_string(), serde_json::Value::String(commit.hash.clone()));
                metadata.insert("author".to_string(), serde_json::Value::String(commit.author.clone()));
                metadata.insert("timestamp".to_string(), serde_json::Value::String(commit.timestamp.to_rfc3339()));
                metadata.insert("project_id".to_string(), serde_json::Value::String(project_id.to_string()));

                contexts.push(ContributedContext {
                    id: Uuid::new_v4().to_string(),
                    context_type: "git_commit".to_string(),
                    title: format!("Commit: {}", commit.message),
                    content: format!("Commit {} by {}\n\nFiles changed:\n{}", 
                        commit.hash, 
                        commit.author,
                        commit.files_changed.iter()
                            .map(|f| format!("- {} ({})", f.path, format!("{:?}", f.change_type)))
                            .collect::<Vec<_>>()
                            .join("\n")
                    ),
                    tags: vec!["git".to_string(), "commit".to_string(), commit.author.clone()],
                    confidence: 0.8,
                    source: "GitIntegrationPlugin".to_string(),
                    metadata,
                });
            }
        }

        if contexts.is_empty() {
            Ok(None)
        } else {
            Ok(Some(ContextContribution {
                context_items: contexts,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("source".to_string(), serde_json::Value::String("git_integration".to_string()));
                    meta.insert("query".to_string(), serde_json::Value::String(query.to_string()));
                    meta
                },
            }))
        }
    }

    async fn shutdown(&mut self) -> Result<()> {
        if let Some(client) = &self.api_client {
            client.log(LogLevel::Info, "Git Integration Plugin shutting down").await?;
        }
        Ok(())
    }

    async fn health_check(&self) -> Result<PluginHealth> {
        let mut status = HealthStatus::Healthy;
        let mut message = None;

        // Check if repository is accessible
        if let Some(repo_path) = &self.repository_path {
            if !repo_path.exists() {
                status = HealthStatus::Critical;
                message = Some("Repository path does not exist".to_string());
            } else if !repo_path.join(".git").exists() {
                status = HealthStatus::Critical;
                message = Some("Not a Git repository".to_string());
            } else {
                // Try to execute a simple Git command
                match self.execute_git_command(&["status", "--porcelain"]).await {
                    Ok(_) => {
                        status = HealthStatus::Healthy;
                        message = Some("Git repository is accessible".to_string());
                    }
                    Err(e) => {
                        status = HealthStatus::Warning;
                        message = Some(format!("Git command failed: {}", e));
                    }
                }
            }
        } else {
            status = HealthStatus::Warning;
            message = Some("Repository path not configured".to_string());
        }

        Ok(PluginHealth {
            status,
            message,
            last_check: Utc::now(),
            resource_usage: ResourceUsage {
                memory_mb: 10.0, // Estimated memory usage
                cpu_percent: 1.0, // Low CPU usage
                network_requests_count: 0,
                file_operations_count: 1, // Git operations
                last_updated: Utc::now(),
            },
        })
    }
}