use crate::models::plugin::{
    PluginInstanceId, PluginPermission, ContextContribution, ContributedContext,
};
use crate::models::context::{BusinessRule, ArchitecturalDecision, PerformanceRequirement};
use crate::services::{
    context_crud_service::ContextCrudService,
    context_query_service::{ContextQueryService, ContextQueryResult},
    PluginSecurity,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Plugin API for safe context operations
#[async_trait]
pub trait PluginApi: Send + Sync {
    /// Query context with permission checking
    async fn query_context(&self, instance_id: PluginInstanceId, query: &str, project_id: &str) -> Result<Vec<Value>>;
    
    /// Create context item with permission checking
    async fn create_context(&self, instance_id: PluginInstanceId, context: ContributedContext) -> Result<String>;
    
    /// Update context item with permission checking
    async fn update_context(&self, instance_id: PluginInstanceId, context_id: &str, updates: HashMap<String, Value>) -> Result<()>;
    
    /// Delete context item with permission checking
    async fn delete_context(&self, instance_id: PluginInstanceId, context_id: &str) -> Result<()>;
    
    /// Contribute context from plugin
    async fn contribute_context(&self, instance_id: PluginInstanceId, contribution: ContextContribution) -> Result<Vec<String>>;
    
    /// Get project information
    async fn get_project_info(&self, instance_id: PluginInstanceId, project_id: &str) -> Result<Value>;
    
    /// List available projects
    async fn list_projects(&self, instance_id: PluginInstanceId) -> Result<Vec<Value>>;
    
    /// Execute network request (if permitted)
    async fn make_network_request(&self, instance_id: PluginInstanceId, url: &str, method: &str, body: Option<Value>) -> Result<Value>;
    
    /// Read file (if permitted)
    async fn read_file(&self, instance_id: PluginInstanceId, file_path: &str) -> Result<String>;
    
    /// Write file (if permitted)
    async fn write_file(&self, instance_id: PluginInstanceId, file_path: &str, content: &str) -> Result<()>;
    
    /// Execute command (if permitted)
    async fn execute_command(&self, instance_id: PluginInstanceId, command: &str, args: &[&str]) -> Result<String>;
    
    /// Log message from plugin
    async fn log_message(&self, instance_id: PluginInstanceId, level: LogLevel, message: &str) -> Result<()>;
}

/// Log levels for plugin logging
#[derive(Debug, Clone)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Default implementation of the plugin API
pub struct DefaultPluginApi {
    context_crud_service: Arc<dyn ContextCrudService>,
    context_query_service: Arc<dyn ContextQueryService>,
    plugin_security: Arc<dyn PluginSecurity>,
    project_service: Arc<dyn crate::services::ProjectService>,
}

impl DefaultPluginApi {
    /// Create a new plugin API
    pub fn new(
        context_crud_service: Arc<dyn ContextCrudService>,
        context_query_service: Arc<dyn ContextQueryService>,
        plugin_security: Arc<dyn PluginSecurity>,
        project_service: Arc<dyn crate::services::ProjectService>,
    ) -> Self {
        Self {
            context_crud_service,
            context_query_service,
            plugin_security,
            project_service,
        }
    }
    
    /// Check if plugin has required permissions
    async fn check_permissions(&self, instance_id: PluginInstanceId, permissions: &[PluginPermission]) -> Result<()> {
        for permission in permissions {
            if !self.plugin_security.check_permission(instance_id, permission).await? {
                return Err(anyhow!("Plugin {} lacks required permission: {:?}", instance_id, permission));
            }
        }
        Ok(())
    }
    
    /// Convert contributed context to internal context types
    fn convert_contributed_context(&self, context: ContributedContext) -> Result<Value> {
        let mut context_data = serde_json::Map::new();
        context_data.insert("id".to_string(), Value::String(context.id));
        context_data.insert("title".to_string(), Value::String(context.title));
        context_data.insert("content".to_string(), Value::String(context.content));
        context_data.insert("tags".to_string(), Value::Array(context.tags.into_iter().map(Value::String).collect()));
        context_data.insert("confidence".to_string(), Value::Number(serde_json::Number::from_f64(context.confidence).unwrap_or_else(|| serde_json::Number::from(0))));
        context_data.insert("source".to_string(), Value::String(context.source));
        
        // Add metadata
        for (key, value) in context.metadata {
            context_data.insert(format!("meta_{}", key), value);
        }
        
        Ok(Value::Object(context_data))
    }
}

#[async_trait]
impl PluginApi for DefaultPluginApi {
    async fn query_context(&self, instance_id: PluginInstanceId, query: &str, project_id: &str) -> Result<Vec<Value>> {
        // Check read permission
        self.check_permissions(instance_id, &[PluginPermission::ReadContext]).await?;
        
        // Execute query - using simplified parameters for now
        let results = self.context_query_service.query_context(project_id, query, "general", &[]).await?;
        
        // Convert results to JSON values
        let mut json_results = Vec::new();
        
        // Add business rules
        for rule in &results.business_rules {
            json_results.push(serde_json::to_value(rule)?);
        }
        
        // Add architectural decisions
        for decision in &results.architectural_decisions {
            json_results.push(serde_json::to_value(decision)?);
        }
        
        // Add performance requirements
        for requirement in &results.performance_requirements {
            json_results.push(serde_json::to_value(requirement)?);
        }
        
        Ok(json_results)
    }
    
    async fn create_context(&self, instance_id: PluginInstanceId, context: ContributedContext) -> Result<String> {
        // Check write permission
        self.check_permissions(instance_id, &[PluginPermission::WriteContext]).await?;
        
        // Convert context based on type
        let project_id = context.metadata.get("project_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");
            
        match context.context_type.as_str() {
            "business_rule" => {
                self.context_crud_service.create_business_rule(
                    project_id,
                    &context.title,
                    Some(&context.content),
                    None, // domain_area
                ).await?;
            }
            "architectural_decision" => {
                self.context_crud_service.create_architectural_decision(
                    project_id,
                    &context.title,
                    Some(&context.content),
                    None, // decision
                ).await?;
            }
            "performance_requirement" => {
                self.context_crud_service.create_performance_requirement(
                    project_id,
                    Some(&context.title), // component_area
                    Some(&context.context_type), // requirement_type
                    Some(&context.content), // target_value
                ).await?;
            }
            _ => {
                return Err(anyhow!("Unsupported context type: {}", context.context_type));
            }
        }
        
        Ok(context.id)
    }
    
    async fn update_context(&self, instance_id: PluginInstanceId, context_id: &str, updates: HashMap<String, Value>) -> Result<()> {
        // Check write permission
        self.check_permissions(instance_id, &[PluginPermission::WriteContext]).await?;
        
        // For now, return an error as we need to implement update methods in the CRUD service
        Err(anyhow!("Context updates not yet implemented in CRUD service"))
    }
    
    async fn delete_context(&self, instance_id: PluginInstanceId, context_id: &str) -> Result<()> {
        // Check delete permission
        self.check_permissions(instance_id, &[PluginPermission::DeleteContext]).await?;
        
        // Try to delete from all context types
        // This is a simplified approach - in practice, we'd need to know the context type
        let _ = self.context_crud_service.delete_business_rule(context_id).await;
        let _ = self.context_crud_service.delete_architectural_decision(context_id).await;
        let _ = self.context_crud_service.delete_performance_requirement(context_id).await;
        
        Ok(())
    }
    
    async fn contribute_context(&self, instance_id: PluginInstanceId, contribution: ContextContribution) -> Result<Vec<String>> {
        // Check write permission
        self.check_permissions(instance_id, &[PluginPermission::WriteContext]).await?;
        
        let mut created_ids = Vec::new();
        
        for context in contribution.context_items {
            match self.create_context(instance_id, context).await {
                Ok(id) => created_ids.push(id),
                Err(e) => {
                    eprintln!("Failed to create context item: {}", e);
                    // Continue with other items
                }
            }
        }
        
        Ok(created_ids)
    }
    
    async fn get_project_info(&self, instance_id: PluginInstanceId, project_id: &str) -> Result<Value> {
        // Check read permission
        self.check_permissions(instance_id, &[PluginPermission::ReadProject]).await?;
        
        match self.project_service.get_project(project_id).await {
            Ok(Some(project)) => Ok(serde_json::to_value(project)?),
            Ok(None) => Err(anyhow!("Project not found: {}", project_id)),
            Err(e) => Err(e.into()),
        }
    }
    
    async fn list_projects(&self, instance_id: PluginInstanceId) -> Result<Vec<Value>> {
        // Check read permission
        self.check_permissions(instance_id, &[PluginPermission::ReadProject]).await?;
        
        let projects = self.project_service.list_projects().await?;
        let mut json_projects = Vec::new();
        
        for project in projects {
            json_projects.push(serde_json::to_value(project)?);
        }
        
        Ok(json_projects)
    }
    
    async fn make_network_request(&self, instance_id: PluginInstanceId, url: &str, method: &str, body: Option<Value>) -> Result<Value> {
        // Check network permission
        self.check_permissions(instance_id, &[PluginPermission::NetworkAccess]).await?;
        
        // Create HTTP client
        let client = reqwest::Client::new();
        
        let request = match method.to_uppercase().as_str() {
            "GET" => client.get(url),
            "POST" => {
                let mut req = client.post(url);
                if let Some(body) = body {
                    req = req.json(&body);
                }
                req
            }
            "PUT" => {
                let mut req = client.put(url);
                if let Some(body) = body {
                    req = req.json(&body);
                }
                req
            }
            "DELETE" => client.delete(url),
            _ => return Err(anyhow!("Unsupported HTTP method: {}", method)),
        };
        
        let response = request.send().await?;
        let status = response.status();
        let text = response.text().await?;
        
        let mut result = serde_json::Map::new();
        result.insert("status".to_string(), Value::Number(serde_json::Number::from(status.as_u16())));
        result.insert("body".to_string(), Value::String(text));
        
        Ok(Value::Object(result))
    }
    
    async fn read_file(&self, instance_id: PluginInstanceId, file_path: &str) -> Result<String> {
        // Check file system read permission
        self.check_permissions(instance_id, &[PluginPermission::FileSystemRead]).await?;
        
        // Security check: prevent path traversal
        if file_path.contains("..") || file_path.starts_with('/') {
            return Err(anyhow!("Invalid file path: {}", file_path));
        }
        
        let content = tokio::fs::read_to_string(file_path).await?;
        Ok(content)
    }
    
    async fn write_file(&self, instance_id: PluginInstanceId, file_path: &str, content: &str) -> Result<()> {
        // Check file system write permission
        self.check_permissions(instance_id, &[PluginPermission::FileSystemWrite]).await?;
        
        // Security check: prevent path traversal
        if file_path.contains("..") || file_path.starts_with('/') {
            return Err(anyhow!("Invalid file path: {}", file_path));
        }
        
        tokio::fs::write(file_path, content).await?;
        Ok(())
    }
    
    async fn execute_command(&self, instance_id: PluginInstanceId, command: &str, args: &[&str]) -> Result<String> {
        // Check command execution permission
        self.check_permissions(instance_id, &[PluginPermission::ExecuteCommands]).await?;
        
        // Security check: only allow safe commands
        let safe_commands = ["git", "ls", "cat", "grep", "find", "wc"];
        if !safe_commands.contains(&command) {
            return Err(anyhow!("Command not allowed: {}", command));
        }
        
        let output = tokio::process::Command::new(command)
            .args(args)
            .output()
            .await?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow!("Command failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }
    
    async fn log_message(&self, instance_id: PluginInstanceId, level: LogLevel, message: &str) -> Result<()> {
        let log_message = format!("[Plugin {}] {}", instance_id, message);
        
        match level {
            LogLevel::Debug => tracing::debug!("{}", log_message),
            LogLevel::Info => tracing::info!("{}", log_message),
            LogLevel::Warn => tracing::warn!("{}", log_message),
            LogLevel::Error => tracing::error!("{}", log_message),
        }
        
        Ok(())
    }
}

/// Plugin API client that plugins use to interact with the context server
#[derive(Clone)]
pub struct PluginApiClient {
    instance_id: PluginInstanceId,
    api: Arc<dyn PluginApi>,
}

impl PluginApiClient {
    /// Create a new plugin API client
    pub fn new(instance_id: PluginInstanceId, api: Arc<dyn PluginApi>) -> Self {
        Self { instance_id, api }
    }
    
    /// Query context
    pub async fn query_context(&self, query: &str, project_id: &str) -> Result<Vec<Value>> {
        self.api.query_context(self.instance_id, query, project_id).await
    }
    
    /// Create context
    pub async fn create_context(&self, context: ContributedContext) -> Result<String> {
        self.api.create_context(self.instance_id, context).await
    }
    
    /// Update context
    pub async fn update_context(&self, context_id: &str, updates: HashMap<String, Value>) -> Result<()> {
        self.api.update_context(self.instance_id, context_id, updates).await
    }
    
    /// Delete context
    pub async fn delete_context(&self, context_id: &str) -> Result<()> {
        self.api.delete_context(self.instance_id, context_id).await
    }
    
    /// Contribute context
    pub async fn contribute_context(&self, contribution: ContextContribution) -> Result<Vec<String>> {
        self.api.contribute_context(self.instance_id, contribution).await
    }
    
    /// Get project info
    pub async fn get_project_info(&self, project_id: &str) -> Result<Value> {
        self.api.get_project_info(self.instance_id, project_id).await
    }
    
    /// List projects
    pub async fn list_projects(&self) -> Result<Vec<Value>> {
        self.api.list_projects(self.instance_id).await
    }
    
    /// Make network request
    pub async fn make_network_request(&self, url: &str, method: &str, body: Option<Value>) -> Result<Value> {
        self.api.make_network_request(self.instance_id, url, method, body).await
    }
    
    /// Read file
    pub async fn read_file(&self, file_path: &str) -> Result<String> {
        self.api.read_file(self.instance_id, file_path).await
    }
    
    /// Write file
    pub async fn write_file(&self, file_path: &str, content: &str) -> Result<()> {
        self.api.write_file(self.instance_id, file_path, content).await
    }
    
    /// Execute command
    pub async fn execute_command(&self, command: &str, args: &[&str]) -> Result<String> {
        self.api.execute_command(self.instance_id, command, args).await
    }
    
    /// Log message
    pub async fn log(&self, level: LogLevel, message: &str) -> Result<()> {
        self.api.log_message(self.instance_id, level, message).await
    }
}