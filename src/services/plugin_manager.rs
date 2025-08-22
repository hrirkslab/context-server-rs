use crate::models::plugin::{
    ContextPlugin, PluginConfiguration, PluginContext, PluginEvent, PluginHealth, PluginId,
    PluginInstance, PluginInstanceId, PluginMetadata, PluginResponse, PluginStatus,
    ResourceUsage, HealthStatus, PluginApiClient,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

/// Plugin manager trait for managing plugin lifecycle
#[async_trait]
pub trait PluginManager: Send + Sync {
    /// Load a plugin from a file path
    async fn load_plugin(&self, plugin_path: &str, config: PluginConfiguration) -> Result<PluginInstanceId>;
    
    /// Unload a plugin instance
    async fn unload_plugin(&self, instance_id: PluginInstanceId) -> Result<()>;
    
    /// Get all loaded plugin instances
    async fn get_loaded_plugins(&self) -> Result<Vec<PluginInstance>>;
    
    /// Get a specific plugin instance
    async fn get_plugin_instance(&self, instance_id: PluginInstanceId) -> Result<Option<PluginInstance>>;
    
    /// Start a plugin instance
    async fn start_plugin(&self, instance_id: PluginInstanceId) -> Result<()>;
    
    /// Stop a plugin instance
    async fn stop_plugin(&self, instance_id: PluginInstanceId) -> Result<()>;
    
    /// Broadcast an event to all active plugins
    async fn broadcast_event(&self, event: PluginEvent) -> Result<Vec<(PluginInstanceId, PluginResponse)>>;
    
    /// Send an event to a specific plugin
    async fn send_event(&self, instance_id: PluginInstanceId, event: PluginEvent) -> Result<PluginResponse>;
    
    /// Get plugin health status
    async fn get_plugin_health(&self, instance_id: PluginInstanceId) -> Result<PluginHealth>;
    
    /// Update plugin configuration
    async fn update_plugin_config(&self, instance_id: PluginInstanceId, config: PluginConfiguration) -> Result<()>;
    
    /// Get plugin resource usage
    async fn get_resource_usage(&self, instance_id: PluginInstanceId) -> Result<ResourceUsage>;
}

/// Default implementation of the plugin manager
pub struct DefaultPluginManager {
    plugins: Arc<RwLock<HashMap<PluginInstanceId, Arc<Mutex<Box<dyn ContextPlugin>>>>>>,
    instances: Arc<RwLock<HashMap<PluginInstanceId, PluginInstance>>>,
    plugin_data_dir: PathBuf,
    temp_dir: PathBuf,
}

impl DefaultPluginManager {
    /// Create a new plugin manager
    pub fn new(plugin_data_dir: PathBuf, temp_dir: PathBuf) -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            instances: Arc::new(RwLock::new(HashMap::new())),
            plugin_data_dir,
            temp_dir,
        }
    }
    
    /// Create plugin context for initialization
    fn create_plugin_context(&self, instance_id: PluginInstanceId, config: PluginConfiguration) -> PluginContext {
        let data_directory = self.plugin_data_dir.join(instance_id.to_string());
        let temp_directory = self.temp_dir.join(instance_id.to_string());
        
        PluginContext {
            instance_id,
            configuration: config,
            data_directory,
            temp_directory,
            api_client: PluginApiClient { _private: () },
        }
    }
    
    /// Update plugin status
    async fn update_plugin_status(&self, instance_id: PluginInstanceId, status: PluginStatus) -> Result<()> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(&instance_id) {
            instance.status = status;
            instance.last_activity = Utc::now();
        }
        Ok(())
    }
    
    /// Update plugin resource usage
    async fn update_resource_usage(&self, instance_id: PluginInstanceId, usage: ResourceUsage) -> Result<()> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(&instance_id) {
            instance.resource_usage = usage;
        }
        Ok(())
    }
}

#[async_trait]
impl PluginManager for DefaultPluginManager {
    async fn load_plugin(&self, plugin_path: &str, config: PluginConfiguration) -> Result<PluginInstanceId> {
        let instance_id = Uuid::new_v4();
        
        // For now, we'll create a placeholder plugin since we don't have dynamic loading yet
        // In a real implementation, this would load the plugin from the file system
        let plugin = create_placeholder_plugin(plugin_path)?;
        let metadata = plugin.metadata().clone();
        
        // Create plugin instance
        let instance = PluginInstance {
            instance_id,
            plugin_id: metadata.id,
            metadata: metadata.clone(),
            configuration: config.clone(),
            status: PluginStatus::Loaded,
            last_error: None,
            resource_usage: ResourceUsage::default(),
            created_at: Utc::now(),
            last_activity: Utc::now(),
        };
        
        // Store the plugin and instance
        {
            let mut plugins = self.plugins.write().await;
            plugins.insert(instance_id, Arc::new(Mutex::new(plugin)));
        }
        
        {
            let mut instances = self.instances.write().await;
            instances.insert(instance_id, instance);
        }
        
        // Initialize the plugin if auto_start is enabled
        if config.auto_start {
            self.start_plugin(instance_id).await?;
        }
        
        Ok(instance_id)
    }
    
    async fn unload_plugin(&self, instance_id: PluginInstanceId) -> Result<()> {
        // Stop the plugin first
        if let Ok(instance) = self.get_plugin_instance(instance_id).await {
            if let Some(instance) = instance {
                if matches!(instance.status, PluginStatus::Active) {
                    self.stop_plugin(instance_id).await?;
                }
            }
        }
        
        // Remove from storage
        {
            let mut plugins = self.plugins.write().await;
            if let Some(plugin_arc) = plugins.remove(&instance_id) {
                let mut plugin = plugin_arc.lock().await;
                plugin.shutdown().await.unwrap_or_else(|e| {
                    eprintln!("Error shutting down plugin {}: {}", instance_id, e);
                });
            }
        }
        
        {
            let mut instances = self.instances.write().await;
            instances.remove(&instance_id);
        }
        
        Ok(())
    }
    
    async fn get_loaded_plugins(&self) -> Result<Vec<PluginInstance>> {
        let instances = self.instances.read().await;
        Ok(instances.values().cloned().collect())
    }
    
    async fn get_plugin_instance(&self, instance_id: PluginInstanceId) -> Result<Option<PluginInstance>> {
        let instances = self.instances.read().await;
        Ok(instances.get(&instance_id).cloned())
    }
    
    async fn start_plugin(&self, instance_id: PluginInstanceId) -> Result<()> {
        self.update_plugin_status(instance_id, PluginStatus::Initializing).await?;
        
        let plugins = self.plugins.read().await;
        let instances = self.instances.read().await;
        
        if let (Some(plugin_arc), Some(instance)) = (plugins.get(&instance_id), instances.get(&instance_id)) {
            let mut plugin = plugin_arc.lock().await;
            let context = self.create_plugin_context(instance_id, instance.configuration.clone());
            
            match plugin.initialize(context).await {
                Ok(()) => {
                    drop(plugin);
                    drop(plugins);
                    drop(instances);
                    self.update_plugin_status(instance_id, PluginStatus::Active).await?;
                    Ok(())
                }
                Err(e) => {
                    drop(plugin);
                    drop(plugins);
                    drop(instances);
                    self.update_plugin_status(instance_id, PluginStatus::Error(e.to_string())).await?;
                    Err(e)
                }
            }
        } else {
            Err(anyhow!("Plugin instance not found: {}", instance_id))
        }
    }
    
    async fn stop_plugin(&self, instance_id: PluginInstanceId) -> Result<()> {
        self.update_plugin_status(instance_id, PluginStatus::Paused).await?;
        Ok(())
    }
    
    async fn broadcast_event(&self, event: PluginEvent) -> Result<Vec<(PluginInstanceId, PluginResponse)>> {
        let plugins = self.plugins.read().await;
        let instances = self.instances.read().await;
        let mut responses = Vec::new();
        
        for (instance_id, plugin_arc) in plugins.iter() {
            if let Some(instance) = instances.get(instance_id) {
                if matches!(instance.status, PluginStatus::Active) {
                    let plugin = plugin_arc.lock().await;
                    match plugin.handle_event(event.clone()).await {
                        Ok(response) => {
                            responses.push((*instance_id, response));
                        }
                        Err(e) => {
                            eprintln!("Plugin {} error handling event: {}", instance_id, e);
                            responses.push((*instance_id, PluginResponse::Error(e.to_string())));
                        }
                    }
                }
            }
        }
        
        Ok(responses)
    }
    
    async fn send_event(&self, instance_id: PluginInstanceId, event: PluginEvent) -> Result<PluginResponse> {
        let plugins = self.plugins.read().await;
        let instances = self.instances.read().await;
        
        if let (Some(plugin_arc), Some(instance)) = (plugins.get(&instance_id), instances.get(&instance_id)) {
            if matches!(instance.status, PluginStatus::Active) {
                let plugin = plugin_arc.lock().await;
                plugin.handle_event(event).await
            } else {
                Err(anyhow!("Plugin is not active: {}", instance_id))
            }
        } else {
            Err(anyhow!("Plugin instance not found: {}", instance_id))
        }
    }
    
    async fn get_plugin_health(&self, instance_id: PluginInstanceId) -> Result<PluginHealth> {
        let plugins = self.plugins.read().await;
        
        if let Some(plugin_arc) = plugins.get(&instance_id) {
            let plugin = plugin_arc.lock().await;
            plugin.health_check().await
        } else {
            Err(anyhow!("Plugin instance not found: {}", instance_id))
        }
    }
    
    async fn update_plugin_config(&self, instance_id: PluginInstanceId, config: PluginConfiguration) -> Result<()> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(&instance_id) {
            instance.configuration = config;
            instance.last_activity = Utc::now();
            Ok(())
        } else {
            Err(anyhow!("Plugin instance not found: {}", instance_id))
        }
    }
    
    async fn get_resource_usage(&self, instance_id: PluginInstanceId) -> Result<ResourceUsage> {
        let instances = self.instances.read().await;
        if let Some(instance) = instances.get(&instance_id) {
            Ok(instance.resource_usage.clone())
        } else {
            Err(anyhow!("Plugin instance not found: {}", instance_id))
        }
    }
}

/// Create a placeholder plugin for testing (will be replaced with dynamic loading)
fn create_placeholder_plugin(plugin_path: &str) -> Result<Box<dyn ContextPlugin>> {
    // This is a placeholder implementation
    // In a real system, this would load the plugin from the file system
    let metadata = PluginMetadata {
        id: Uuid::new_v4(),
        name: format!("Plugin from {}", plugin_path),
        version: "1.0.0".to_string(),
        description: "Placeholder plugin".to_string(),
        author: "System".to_string(),
        homepage: None,
        repository: None,
        license: "MIT".to_string(),
        keywords: vec!["placeholder".to_string()],
        dependencies: vec![],
        permissions: vec![],
        configuration_schema: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    Ok(Box::new(PlaceholderPlugin { metadata }))
}

/// Placeholder plugin implementation for testing
struct PlaceholderPlugin {
    metadata: PluginMetadata,
}

#[async_trait]
impl ContextPlugin for PlaceholderPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&mut self, _context: PluginContext) -> Result<()> {
        Ok(())
    }
    
    async fn handle_event(&self, _event: PluginEvent) -> Result<PluginResponse> {
        Ok(PluginResponse::EventIgnored)
    }
    
    async fn provide_context(&self, _query: &str, _project_id: &str) -> Result<Option<crate::models::plugin::ContextContribution>> {
        Ok(None)
    }
    
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
    
    async fn health_check(&self) -> Result<PluginHealth> {
        Ok(PluginHealth {
            status: HealthStatus::Healthy,
            message: None,
            last_check: Utc::now(),
            resource_usage: ResourceUsage::default(),
        })
    }
}