use crate::models::plugin::{
    ContextPlugin, PluginContext, PluginEvent, PluginHealth, 
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
use tokio::sync::Mutex;
use uuid::Uuid;

/// IDE integration plugin for real-time context updates and code analysis
pub struct IdeIntegrationPlugin {
    metadata: PluginMetadata,
    config: Mutex<IdePluginConfig>,
    api_client: Option<PluginApiClient>,
    workspace_path: Option<PathBuf>,
    active_files: Mutex<HashMap<String, FileAnalysisState>>,
    context_suggestions: Mutex<Vec<ContextSuggestion>>,
}

/// Configuration for the IDE integration plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdePluginConfig {
    pub workspace_path: String,
    pub supported_languages: Vec<String>,
    pub auto_analyze_on_save: bool,
    pub real_time_suggestions: bool,
    pub context_extraction_enabled: bool,
    pub debugging_integration: bool,
    pub file_watch_patterns: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub analysis_rules: Vec<CodeAnalysisRule>,
    pub suggestion_triggers: Vec<SuggestionTrigger>,
}

/// Rule for analyzing code and extracting context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysisRule {
    pub name: String,
    pub language: String,
    pub pattern: String, // regex pattern to match in code
    pub context_type: String,
    pub extraction_method: CodeExtractionMethod,
    pub confidence: f64,
}

/// Method for extracting context from code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodeExtractionMethod {
    FunctionSignature,
    ClassDefinition,
    InterfaceDefinition,
    Comment,
    DocString,
    ImportStatement,
    ConfigurationValue,
    TestCase,
    ErrorHandling,
    Custom(String),
}

/// Trigger for context suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionTrigger {
    pub name: String,
    pub trigger_type: TriggerType,
    pub condition: String, // condition to check
    pub suggestion_template: String,
}

/// Type of suggestion trigger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    FileOpen,
    CursorPosition,
    TextChange,
    DebugBreakpoint,
    ErrorOccurred,
    TestRun,
    BuildComplete,
}

/// State of file analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAnalysisState {
    pub file_path: String,
    pub language: String,
    pub last_modified: DateTime<Utc>,
    pub last_analyzed: DateTime<Utc>,
    pub extracted_contexts: Vec<ExtractedContext>,
    pub suggestions: Vec<ContextSuggestion>,
    pub analysis_errors: Vec<String>,
}

/// Context extracted from code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedContext {
    pub id: String,
    pub context_type: String,
    pub title: String,
    pub content: String,
    pub line_number: Option<u32>,
    pub column_number: Option<u32>,
    pub confidence: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Context suggestion for IDE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSuggestion {
    pub id: String,
    pub title: String,
    pub description: String,
    pub suggestion_type: SuggestionType,
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
    pub priority: SuggestionPriority,
    pub actions: Vec<SuggestionAction>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Type of context suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    MissingDocumentation,
    ArchitecturalDecision,
    BusinessRule,
    PerformanceRequirement,
    SecurityConcern,
    TestCoverage,
    CodePattern,
    Refactoring,
}

/// Priority of suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Action that can be taken on a suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionAction {
    pub action_type: ActionType,
    pub title: String,
    pub description: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Type of suggestion action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    CreateContext,
    UpdateContext,
    NavigateToCode,
    ShowDocumentation,
    RunAnalysis,
    ApplyFix,
}

impl Default for IdePluginConfig {
    fn default() -> Self {
        Self {
            workspace_path: ".".to_string(),
            supported_languages: vec![
                "rust".to_string(),
                "typescript".to_string(),
                "javascript".to_string(),
                "python".to_string(),
                "java".to_string(),
                "cpp".to_string(),
                "csharp".to_string(),
            ],
            auto_analyze_on_save: true,
            real_time_suggestions: true,
            context_extraction_enabled: true,
            debugging_integration: true,
            file_watch_patterns: vec![
                "**/*.rs".to_string(),
                "**/*.ts".to_string(),
                "**/*.js".to_string(),
                "**/*.py".to_string(),
                "**/*.java".to_string(),
                "**/*.cpp".to_string(),
                "**/*.h".to_string(),
                "**/*.cs".to_string(),
            ],
            ignore_patterns: vec![
                "**/target/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/build/**".to_string(),
                "**/dist/**".to_string(),
                "**/.git/**".to_string(),
            ],
            analysis_rules: vec![
                CodeAnalysisRule {
                    name: "Function Documentation".to_string(),
                    language: "rust".to_string(),
                    pattern: r"pub fn\s+(\w+)".to_string(),
                    context_type: "business_rule".to_string(),
                    extraction_method: CodeExtractionMethod::FunctionSignature,
                    confidence: 0.8,
                },
                CodeAnalysisRule {
                    name: "Struct Definition".to_string(),
                    language: "rust".to_string(),
                    pattern: r"pub struct\s+(\w+)".to_string(),
                    context_type: "architectural_decision".to_string(),
                    extraction_method: CodeExtractionMethod::ClassDefinition,
                    confidence: 0.9,
                },
                CodeAnalysisRule {
                    name: "Error Handling".to_string(),
                    language: "rust".to_string(),
                    pattern: r"Result<.*>".to_string(),
                    context_type: "performance_requirement".to_string(),
                    extraction_method: CodeExtractionMethod::ErrorHandling,
                    confidence: 0.7,
                },
            ],
            suggestion_triggers: vec![
                SuggestionTrigger {
                    name: "Missing Documentation".to_string(),
                    trigger_type: TriggerType::FileOpen,
                    condition: "undocumented_function".to_string(),
                    suggestion_template: "Consider adding documentation for this function".to_string(),
                },
                SuggestionTrigger {
                    name: "Complex Function".to_string(),
                    trigger_type: TriggerType::TextChange,
                    condition: "function_complexity > 10".to_string(),
                    suggestion_template: "This function is complex, consider documenting its business logic".to_string(),
                },
            ],
        }
    }
}

impl IdeIntegrationPlugin {
    /// Create a new IDE integration plugin
    pub fn new() -> Self {
        let metadata = PluginMetadata {
            id: Uuid::new_v4(),
            name: "IDE Integration".to_string(),
            version: "1.0.0".to_string(),
            description: "Provides real-time context updates and code analysis integration for IDEs".to_string(),
            author: "Context Server".to_string(),
            homepage: None,
            repository: None,
            license: "MIT".to_string(),
            keywords: vec!["ide".to_string(), "code-analysis".to_string(), "real-time".to_string()],
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
            config: Mutex::new(IdePluginConfig::default()),
            api_client: None,
            workspace_path: None,
            active_files: Mutex::new(HashMap::new()),
            context_suggestions: Mutex::new(Vec::new()),
        }
    }

    /// Analyze a file and extract context
    async fn analyze_file(&self, file_path: &Path) -> Result<FileAnalysisState> {
        let content = tokio::fs::read_to_string(file_path).await?;
        let language = self.detect_language(file_path)?;
        let config = self.config.lock().await;
        
        let mut extracted_contexts = Vec::new();
        let mut suggestions = Vec::new();
        let mut analysis_errors = Vec::new();

        // Apply analysis rules
        for rule in &config.analysis_rules {
            if rule.language != language {
                continue;
            }

            match self.apply_analysis_rule(rule, &content, file_path).await {
                Ok(mut contexts) => extracted_contexts.append(&mut contexts),
                Err(e) => analysis_errors.push(format!("Rule '{}' failed: {}", rule.name, e)),
            }
        }

        // Generate suggestions based on analysis
        for trigger in &config.suggestion_triggers {
            match self.evaluate_suggestion_trigger(trigger, &content, file_path, &extracted_contexts).await {
                Ok(mut trigger_suggestions) => suggestions.append(&mut trigger_suggestions),
                Err(e) => analysis_errors.push(format!("Trigger '{}' failed: {}", trigger.name, e)),
            }
        }

        Ok(FileAnalysisState {
            file_path: file_path.to_string_lossy().to_string(),
            language,
            last_modified: Utc::now(),
            last_analyzed: Utc::now(),
            extracted_contexts,
            suggestions,
            analysis_errors,
        })
    }

    /// Detect programming language from file extension
    fn detect_language(&self, file_path: &Path) -> Result<String> {
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow!("No file extension"))?;

        let language = match extension {
            "rs" => "rust",
            "ts" => "typescript",
            "js" => "javascript",
            "py" => "python",
            "java" => "java",
            "cpp" | "cc" | "cxx" => "cpp",
            "h" | "hpp" => "cpp",
            "cs" => "csharp",
            _ => return Err(anyhow!("Unsupported file extension: {}", extension)),
        };

        Ok(language.to_string())
    }

    /// Apply a single analysis rule to extract context
    async fn apply_analysis_rule(&self, rule: &CodeAnalysisRule, content: &str, file_path: &Path) -> Result<Vec<ExtractedContext>> {
        let regex = regex::Regex::new(&rule.pattern)?;
        let mut contexts = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if let Some(captures) = regex.captures(line) {
                let extracted_content = match &rule.extraction_method {
                    CodeExtractionMethod::FunctionSignature => {
                        if let Some(func_name) = captures.get(1) {
                            format!("Function: {}", func_name.as_str())
                        } else {
                            line.to_string()
                        }
                    }
                    CodeExtractionMethod::ClassDefinition => {
                        if let Some(class_name) = captures.get(1) {
                            format!("Struct/Class: {}", class_name.as_str())
                        } else {
                            line.to_string()
                        }
                    }
                    CodeExtractionMethod::Comment => {
                        line.trim_start_matches("//").trim().to_string()
                    }
                    CodeExtractionMethod::DocString => {
                        line.trim_start_matches("///").trim().to_string()
                    }
                    _ => line.to_string(),
                };

                let mut metadata = HashMap::new();
                metadata.insert("file_path".to_string(), serde_json::Value::String(file_path.to_string_lossy().to_string()));
                metadata.insert("language".to_string(), serde_json::Value::String(rule.language.clone()));
                metadata.insert("rule_name".to_string(), serde_json::Value::String(rule.name.clone()));

                contexts.push(ExtractedContext {
                    id: Uuid::new_v4().to_string(),
                    context_type: rule.context_type.clone(),
                    title: format!("{} in {}", rule.name, file_path.file_name().unwrap_or_default().to_string_lossy()),
                    content: extracted_content,
                    line_number: Some(line_num as u32 + 1),
                    column_number: None,
                    confidence: rule.confidence,
                    metadata,
                });
            }
        }

        Ok(contexts)
    }

    /// Evaluate a suggestion trigger
    async fn evaluate_suggestion_trigger(
        &self, 
        trigger: &SuggestionTrigger, 
        content: &str, 
        file_path: &Path,
        extracted_contexts: &[ExtractedContext]
    ) -> Result<Vec<ContextSuggestion>> {
        let mut suggestions = Vec::new();

        // Simple condition evaluation (in a real implementation, this would be more sophisticated)
        let should_trigger = match trigger.condition.as_str() {
            "undocumented_function" => {
                // Check if there are functions without documentation
                let has_functions = content.contains("fn ") || content.contains("function ");
                let has_docs = content.contains("///") || content.contains("/**");
                has_functions && !has_docs
            }
            condition if condition.starts_with("function_complexity") => {
                // Simple complexity check based on line count and nesting
                let lines = content.lines().count();
                let nesting_level = content.matches('{').count();
                lines > 50 || nesting_level > 10
            }
            _ => false,
        };

        if should_trigger {
            let mut metadata = HashMap::new();
            metadata.insert("file_path".to_string(), serde_json::Value::String(file_path.to_string_lossy().to_string()));
            metadata.insert("trigger_condition".to_string(), serde_json::Value::String(trigger.condition.clone()));

            let actions = vec![
                SuggestionAction {
                    action_type: ActionType::CreateContext,
                    title: "Create Context".to_string(),
                    description: "Create context entry for this code".to_string(),
                    parameters: HashMap::new(),
                },
                SuggestionAction {
                    action_type: ActionType::ShowDocumentation,
                    title: "Show Examples".to_string(),
                    description: "Show documentation examples".to_string(),
                    parameters: HashMap::new(),
                },
            ];

            suggestions.push(ContextSuggestion {
                id: Uuid::new_v4().to_string(),
                title: trigger.name.clone(),
                description: trigger.suggestion_template.clone(),
                suggestion_type: SuggestionType::MissingDocumentation,
                file_path: Some(file_path.to_string_lossy().to_string()),
                line_number: None,
                priority: SuggestionPriority::Medium,
                actions,
                metadata,
            });
        }

        Ok(suggestions)
    }

    /// Process file change event
    async fn process_file_change(&self, file_path: &str) -> Result<()> {
        let path = Path::new(file_path);
        
        // Check if file should be analyzed
        if !self.should_analyze_file(path).await? {
            return Ok(());
        }

        if let Some(client) = &self.api_client {
            client.log(LogLevel::Info, &format!("Analyzing file: {}", file_path)).await?;
        }

        // Analyze the file
        let analysis_state = self.analyze_file(path).await?;
        
        // Update active files
        {
            let mut active_files = self.active_files.lock().await;
            active_files.insert(file_path.to_string(), analysis_state.clone());
        }

        // Create context contributions from extracted contexts
        if !analysis_state.extracted_contexts.is_empty() {
            let context_items: Vec<ContributedContext> = analysis_state.extracted_contexts
                .into_iter()
                .map(|ctx| ContributedContext {
                    id: ctx.id,
                    context_type: ctx.context_type,
                    title: ctx.title,
                    content: ctx.content,
                    tags: vec!["ide".to_string(), "auto-generated".to_string(), analysis_state.language.clone()],
                    confidence: ctx.confidence,
                    source: "IdeIntegrationPlugin".to_string(),
                    metadata: ctx.metadata,
                })
                .collect();

            if let Some(client) = &self.api_client {
                let contribution = ContextContribution {
                    context_items,
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("source".to_string(), serde_json::Value::String("ide_integration".to_string()));
                        meta.insert("file_path".to_string(), serde_json::Value::String(file_path.to_string()));
                        meta
                    },
                };

                match client.contribute_context(contribution).await {
                    Ok(created_ids) => {
                        client.log(LogLevel::Info, &format!("Created {} context items from {}", created_ids.len(), file_path)).await?;
                    }
                    Err(e) => {
                        client.log(LogLevel::Error, &format!("Failed to contribute context from {}: {}", file_path, e)).await?;
                    }
                }
            }
        }

        // Update suggestions
        {
            let mut suggestions = self.context_suggestions.lock().await;
            suggestions.extend(analysis_state.suggestions);
        }

        Ok(())
    }

    /// Check if a file should be analyzed
    async fn should_analyze_file(&self, file_path: &Path) -> Result<bool> {
        let config = self.config.lock().await;
        
        // Check ignore patterns
        let path_str = file_path.to_string_lossy();
        for ignore_pattern in &config.ignore_patterns {
            if glob::Pattern::new(ignore_pattern)?.matches(&path_str) {
                return Ok(false);
            }
        }

        // Check watch patterns
        for watch_pattern in &config.file_watch_patterns {
            if glob::Pattern::new(watch_pattern)?.matches(&path_str) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Get context suggestions for a specific file
    pub async fn get_suggestions_for_file(&self, file_path: &str) -> Vec<ContextSuggestion> {
        let suggestions = self.context_suggestions.lock().await;
        suggestions.iter()
            .filter(|s| s.file_path.as_ref().map_or(false, |p| p == file_path))
            .cloned()
            .collect()
    }

    /// Get all active context suggestions
    pub async fn get_all_suggestions(&self) -> Vec<ContextSuggestion> {
        let suggestions = self.context_suggestions.lock().await;
        suggestions.clone()
    }

    /// Clear suggestions for a file
    pub async fn clear_suggestions_for_file(&self, file_path: &str) {
        let mut suggestions = self.context_suggestions.lock().await;
        suggestions.retain(|s| s.file_path.as_ref().map_or(true, |p| p != file_path));
    }
}

#[async_trait]
impl ContextPlugin for IdeIntegrationPlugin {
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

        // Load configuration from context
        if let Some(workspace_path) = context.configuration.settings.get("workspace_path") {
            if let Some(path_str) = workspace_path.as_str() {
                let mut config = self.config.lock().await;
                config.workspace_path = path_str.to_string();
                self.workspace_path = Some(PathBuf::from(path_str));
            }
        }

        // Load other configuration settings
        let mut config = self.config.lock().await;
        if let Some(auto_analyze) = context.configuration.settings.get("auto_analyze_on_save") {
            if let Some(auto_analyze_bool) = auto_analyze.as_bool() {
                config.auto_analyze_on_save = auto_analyze_bool;
            }
        }

        if let Some(real_time) = context.configuration.settings.get("real_time_suggestions") {
            if let Some(real_time_bool) = real_time.as_bool() {
                config.real_time_suggestions = real_time_bool;
            }
        }

        Ok(())
    }

    async fn handle_event(&self, event: PluginEvent) -> Result<PluginResponse> {
        match event {
            PluginEvent::SystemStartup => {
                if let Some(client) = &self.api_client {
                    client.log(LogLevel::Info, "IDE Integration Plugin started").await?;
                }
                Ok(PluginResponse::EventHandled)
            }
            PluginEvent::Custom { event_type, data } => {
                match event_type.as_str() {
                    "file_changed" => {
                        if let Some(file_path) = data.get("file_path").and_then(|v| v.as_str()) {
                            if let Err(e) = self.process_file_change(file_path).await {
                                return Ok(PluginResponse::Error(format!("Failed to process file change: {}", e)));
                            }
                        }
                        Ok(PluginResponse::Success)
                    }
                    "file_opened" => {
                        if let Some(file_path) = data.get("file_path").and_then(|v| v.as_str()) {
                            // Analyze file when opened
                            if let Err(e) = self.process_file_change(file_path).await {
                                if let Some(client) = &self.api_client {
                                    client.log(LogLevel::Warn, &format!("Failed to analyze opened file {}: {}", file_path, e)).await?;
                                }
                            }
                        }
                        Ok(PluginResponse::EventHandled)
                    }
                    "get_suggestions" => {
                        let suggestions = if let Some(file_path) = data.get("file_path").and_then(|v| v.as_str()) {
                            self.get_suggestions_for_file(file_path).await
                        } else {
                            self.get_all_suggestions().await
                        };
                        
                        let response_data = serde_json::to_value(suggestions)?;
                        Ok(PluginResponse::ContextContribution(ContextContribution {
                            context_items: vec![],
                            metadata: {
                                let mut meta = HashMap::new();
                                meta.insert("suggestions".to_string(), response_data);
                                meta
                            },
                        }))
                    }
                    "clear_suggestions" => {
                        if let Some(file_path) = data.get("file_path").and_then(|v| v.as_str()) {
                            self.clear_suggestions_for_file(file_path).await;
                        }
                        Ok(PluginResponse::Success)
                    }
                    _ => Ok(PluginResponse::EventIgnored),
                }
            }
            _ => Ok(PluginResponse::EventIgnored),
        }
    }

    async fn provide_context(&self, query: &str, project_id: &str) -> Result<Option<ContextContribution>> {
        // Check if query is related to code analysis or IDE features
        let ide_keywords = ["code", "function", "class", "method", "analysis", "suggestion", "ide"];
        let query_lower = query.to_lowercase();
        
        if !ide_keywords.iter().any(|&keyword| query_lower.contains(keyword)) {
            return Ok(None);
        }

        // Get relevant file analysis states
        let active_files = self.active_files.lock().await;
        let mut contexts = Vec::new();

        for (file_path, analysis_state) in active_files.iter() {
            // Check if file content or path matches the query
            if file_path.to_lowercase().contains(&query_lower) ||
               analysis_state.extracted_contexts.iter().any(|ctx| 
                   ctx.content.to_lowercase().contains(&query_lower) ||
                   ctx.title.to_lowercase().contains(&query_lower)
               ) {
                
                for extracted_ctx in &analysis_state.extracted_contexts {
                    let mut metadata = extracted_ctx.metadata.clone();
                    metadata.insert("project_id".to_string(), serde_json::Value::String(project_id.to_string()));
                    metadata.insert("analysis_timestamp".to_string(), serde_json::Value::String(analysis_state.last_analyzed.to_rfc3339()));

                    contexts.push(ContributedContext {
                        id: extracted_ctx.id.clone(),
                        context_type: extracted_ctx.context_type.clone(),
                        title: extracted_ctx.title.clone(),
                        content: extracted_ctx.content.clone(),
                        tags: vec!["ide".to_string(), "code-analysis".to_string(), analysis_state.language.clone()],
                        confidence: extracted_ctx.confidence,
                        source: "IdeIntegrationPlugin".to_string(),
                        metadata,
                    });
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
                    meta.insert("source".to_string(), serde_json::Value::String("ide_integration".to_string()));
                    meta.insert("query".to_string(), serde_json::Value::String(query.to_string()));
                    meta
                },
            }))
        }
    }

    async fn shutdown(&mut self) -> Result<()> {
        if let Some(client) = &self.api_client {
            client.log(LogLevel::Info, "IDE Integration Plugin shutting down").await?;
        }
        Ok(())
    }

    async fn health_check(&self) -> Result<PluginHealth> {
        let mut status = HealthStatus::Healthy;
        let mut message = None;

        // Check if workspace is accessible
        if let Some(workspace_path) = &self.workspace_path {
            if !workspace_path.exists() {
                status = HealthStatus::Critical;
                message = Some("Workspace path does not exist".to_string());
            } else if !workspace_path.is_dir() {
                status = HealthStatus::Critical;
                message = Some("Workspace path is not a directory".to_string());
            } else {
                status = HealthStatus::Healthy;
                message = Some("Workspace is accessible".to_string());
            }
        } else {
            status = HealthStatus::Warning;
            message = Some("Workspace path not configured".to_string());
        }

        // Check active files count
        let active_files_count = self.active_files.lock().await.len();
        let suggestions_count = self.context_suggestions.lock().await.len();

        Ok(PluginHealth {
            status,
            message,
            last_check: Utc::now(),
            resource_usage: ResourceUsage {
                memory_mb: 20.0 + (active_files_count as f64 * 0.5), // Estimated memory usage
                cpu_percent: 2.0, // Low CPU usage when idle
                network_requests_count: 0,
                file_operations_count: active_files_count as u32,
                last_updated: Utc::now(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::plugin::{PluginConfiguration, PluginContext, PluginEvent};
    use std::collections::HashMap;
    use tempfile::TempDir;
    use tokio_test;

    fn create_test_plugin() -> IdeIntegrationPlugin {
        IdeIntegrationPlugin::new()
    }

    fn create_test_context(workspace_path: &str) -> PluginContext {
        let mut settings = HashMap::new();
        settings.insert("workspace_path".to_string(), serde_json::Value::String(workspace_path.to_string()));
        settings.insert("auto_analyze_on_save".to_string(), serde_json::Value::Bool(true));
        settings.insert("real_time_suggestions".to_string(), serde_json::Value::Bool(true));

        PluginContext {
            instance_id: Uuid::new_v4(),
            configuration: PluginConfiguration {
                enabled: true,
                auto_start: true,
                settings,
                resource_limits: crate::models::plugin::ResourceLimits::default(),
            },
            data_directory: std::path::PathBuf::from("/tmp"),
            temp_directory: std::path::PathBuf::from("/tmp"),
            api_client: crate::models::plugin::PluginApiClient { _private: () },
        }
    }

    #[tokio::test]
    async fn test_plugin_initialization() {
        let mut plugin = create_test_plugin();
        let temp_dir = TempDir::new().unwrap();
        let context = create_test_context(temp_dir.path().to_str().unwrap());

        let result = plugin.initialize(context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_language_detection() {
        let plugin = create_test_plugin();
        
        assert_eq!(plugin.detect_language(Path::new("test.rs")).unwrap(), "rust");
        assert_eq!(plugin.detect_language(Path::new("test.ts")).unwrap(), "typescript");
        assert_eq!(plugin.detect_language(Path::new("test.js")).unwrap(), "javascript");
        assert_eq!(plugin.detect_language(Path::new("test.py")).unwrap(), "python");
        assert_eq!(plugin.detect_language(Path::new("test.java")).unwrap(), "java");
        assert_eq!(plugin.detect_language(Path::new("test.cpp")).unwrap(), "cpp");
        assert_eq!(plugin.detect_language(Path::new("test.cs")).unwrap(), "csharp");
        
        assert!(plugin.detect_language(Path::new("test.unknown")).is_err());
    }

    #[tokio::test]
    async fn test_should_analyze_file() {
        let plugin = create_test_plugin();
        
        // Should analyze Rust files
        assert!(plugin.should_analyze_file(Path::new("src/main.rs")).await.unwrap());
        
        // Should not analyze ignored files
        assert!(!plugin.should_analyze_file(Path::new("target/debug/main")).await.unwrap());
        assert!(!plugin.should_analyze_file(Path::new("node_modules/package/index.js")).await.unwrap());
        assert!(!plugin.should_analyze_file(Path::new(".git/config")).await.unwrap());
    }

    #[tokio::test]
    async fn test_file_analysis() {
        let plugin = create_test_plugin();
        let temp_dir = TempDir::new().unwrap();
        
        // Create a test Rust file
        let test_file = temp_dir.path().join("test.rs");
        tokio::fs::write(&test_file, r#"
/// This is a documented function
pub fn hello_world() -> String {
    "Hello, World!".to_string()
}

pub struct TestStruct {
    pub field: String,
}

fn complex_function() -> Result<String, Box<dyn std::error::Error>> {
    // This is a complex function with error handling
    Ok("test".to_string())
}
"#).await.unwrap();

        let analysis = plugin.analyze_file(&test_file).await.unwrap();
        
        assert_eq!(analysis.language, "rust");
        assert!(!analysis.extracted_contexts.is_empty());
        
        // Should have extracted some contexts
        let function_contexts: Vec<_> = analysis.extracted_contexts.iter()
            .filter(|ctx| ctx.context_type == "business_rule")
            .collect();
        assert!(!function_contexts.is_empty());
    }

    #[tokio::test]
    async fn test_event_handling() {
        let mut plugin = create_test_plugin();
        let temp_dir = TempDir::new().unwrap();
        let context = create_test_context(temp_dir.path().to_str().unwrap());
        
        plugin.initialize(context).await.unwrap();

        // Test system startup event
        let startup_event = PluginEvent::SystemStartup;
        let response = plugin.handle_event(startup_event).await.unwrap();
        assert!(matches!(response, PluginResponse::EventHandled));

        // Test custom file_changed event
        let mut event_data = HashMap::new();
        event_data.insert("file_path".to_string(), serde_json::Value::String("test.rs".to_string()));
        
        let file_event = PluginEvent::Custom {
            event_type: "file_changed".to_string(),
            data: serde_json::Value::Object(event_data.into_iter().collect()),
        };
        
        let response = plugin.handle_event(file_event).await.unwrap();
        // Should succeed even if file doesn't exist (will just log an error)
        assert!(matches!(response, PluginResponse::Success | PluginResponse::Error(_)));
    }

    #[tokio::test]
    async fn test_context_suggestions() {
        let plugin = create_test_plugin();
        
        // Initially no suggestions
        let suggestions = plugin.get_all_suggestions().await;
        assert!(suggestions.is_empty());
        
        // Test getting suggestions for a specific file
        let file_suggestions = plugin.get_suggestions_for_file("test.rs").await;
        assert!(file_suggestions.is_empty());
    }

    #[tokio::test]
    async fn test_health_check() {
        let mut plugin = create_test_plugin();
        let temp_dir = TempDir::new().unwrap();
        let context = create_test_context(temp_dir.path().to_str().unwrap());
        
        plugin.initialize(context).await.unwrap();
        
        let health = plugin.health_check().await.unwrap();
        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(health.message.is_some());
    }

    #[tokio::test]
    async fn test_provide_context() {
        let plugin = create_test_plugin();
        
        // Test with IDE-related query
        let result = plugin.provide_context("code analysis", "test_project").await.unwrap();
        // Should return None since no files are analyzed yet
        assert!(result.is_none());
        
        // Test with non-IDE query
        let result = plugin.provide_context("database schema", "test_project").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_plugin_metadata() {
        let plugin = create_test_plugin();
        let metadata = plugin.metadata();
        
        assert_eq!(metadata.name, "IDE Integration");
        assert_eq!(metadata.version, "1.0.0");
        assert!(metadata.permissions.contains(&PluginPermission::ReadContext));
        assert!(metadata.permissions.contains(&PluginPermission::WriteContext));
        assert!(metadata.permissions.contains(&PluginPermission::FileSystemRead));
        assert!(metadata.permissions.contains(&PluginPermission::FileSystemWrite));
    }
}