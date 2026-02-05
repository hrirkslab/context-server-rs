use crate::models::plugin::{
    ContextPlugin, PluginConfiguration, PluginEvent, PluginHealth, PluginId, PluginInstance,
    PluginInstanceId, PluginMetadata, PluginRegistryEntry, PluginResponse, PluginSearchFilters,
    ResourceUsage, PluginPermission, ResourceLimits,
};
use crate::services::{
    PluginManager, PluginDiscovery, PluginSecurity, PluginConfigurationManager,
    DefaultPluginManager, DefaultPluginDiscovery, DefaultPluginSecurity, DefaultPluginConfigurationManager,
    ConfigurationTemplateManager, PluginLoader, ResourceMonitor, PermissionValidator,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

/// High-level plugin service that orchestrates all plugin functionality
#[async_trait]
pub trait PluginService: Send + Sync {
    /// Initialize the plugin system
    async fn initialize(&self) -> Result<()>;
    
    /// Shutdown the plugin system
    async fn shutdown(&self) -> Result<()>;
    
    /// Install a plugin from the marketplace
    async fn install_plugin(&self, plugin_id: &str, version: &str) -> Result<PluginInstanceId>;
    
    /// Install a plugin from a local path
    async fn install_plugin_from_path(&self, plugin_path: &Path, config: Option<PluginConfiguration>) -> Result<PluginInstanceId>;
    
    /// Uninstall a plugin
    async fn uninstall_plugin(&self, instance_id: PluginInstanceId) -> Result<()>;
    
    /// Enable a plugin
    async fn enable_plugin(&self, instance_id: PluginInstanceId) -> Result<()>;
    
    /// Disable a plugin
    async fn disable_plugin(&self, instance_id: PluginInstanceId) -> Result<()>;
    
    /// Get all installed plugins
    async fn get_installed_plugins(&self) -> Result<Vec<PluginInstance>>;
    
    /// Get available plugins from marketplace
    async fn search_marketplace(&self, filters: PluginSearchFilters) -> Result<Vec<PluginRegistryEntry>>;
    
    /// Get plugin configuration
    async fn get_plugin_config(&self, instance_id: PluginInstanceId) -> Result<PluginConfiguration>;
    
    /// Update plugin configuration
    async fn update_plugin_config(&self, instance_id: PluginInstanceId, config: PluginConfiguration) -> Result<()>;
    
    /// Get plugin health status
    async fn get_plugin_health(&self, instance_id: PluginInstanceId) -> Result<PluginHealth>;
    
    /// Get plugin resource usage
    async fn get_plugin_resource_usage(&self, instance_id: PluginInstanceId) -> Result<ResourceUsage>;
    
    /// Broadcast event to all active plugins
    async fn broadcast_event(&self, event: PluginEvent) -> Result<Vec<(PluginInstanceId, PluginResponse)>>;
    
    /// Send event to specific plugin
    async fn send_event_to_plugin(&self, instance_id: PluginInstanceId, event: PluginEvent) -> Result<PluginResponse>;
    
    /// Grant permission to plugin
    async fn grant_permission(&self, instance_id: PluginInstanceId, permission: PluginPermission) -> Result<()>;
    
    /// Revoke permission from plugin
    async fn revoke_permission(&self, instance_id: PluginInstanceId, permission: PluginPermission) -> Result<()>;
    
    /// Update resource limits for plugin
    async fn update_resource_limits(&self, instance_id: PluginInstanceId, limits: ResourceLimits) -> Result<()>;
}

/// Default implementation of the plugin service
pub struct DefaultPluginService {
    plugin_manager: Arc<dyn PluginManager>,
    plugin_discovery: Arc<dyn PluginDiscovery>,
    plugin_security: Arc<dyn PluginSecurity>,
    plugin_config: Arc<dyn PluginConfigurationManager>,
    plugin_loader: PluginLoader,
    resource_monitor: ResourceMonitor,
    permission_validator: PermissionValidator,
    config_template_manager: ConfigurationTemplateManager,
    plugin_install_dir: PathBuf,
    plugin_data_dir: PathBuf,
    temp_dir: PathBuf,
}

impl DefaultPluginService {
    /// Create a new plugin service
    pub fn new(
        plugin_install_dir: PathBuf,
        plugin_data_dir: PathBuf,
        temp_dir: PathBuf,
        marketplace_url: Option<String>,
    ) -> Self {
        // Create plugin directories
        let plugin_directories = vec![plugin_install_dir.clone()];
        
        // Create services
        let plugin_manager = Arc::new(DefaultPluginManager::new(
            plugin_data_dir.clone(),
            temp_dir.clone(),
        ));
        
        let plugin_discovery = Arc::new(DefaultPluginDiscovery::new(
            plugin_directories,
            marketplace_url,
        ));
        
        let plugin_security = Arc::new(DefaultPluginSecurity::new(
            Duration::from_secs(60), // monitoring interval
        ));
        
        let plugin_config = Arc::new(DefaultPluginConfigurationManager::new(
            plugin_data_dir.join("configs"),
        ));
        
        let plugin_loader = PluginLoader::new(plugin_discovery.clone());
        let resource_monitor = ResourceMonitor::new(plugin_security.clone(), Duration::from_secs(30));
        let permission_validator = PermissionValidator::new(plugin_security.clone());
        let config_template_manager = ConfigurationTemplateManager::new();
        
        Self {
            plugin_manager,
            plugin_discovery,
            plugin_security,
            plugin_config,
            plugin_loader,
            resource_monitor,
            permission_validator,
            config_template_manager,
            plugin_install_dir,
            plugin_data_dir,
            temp_dir,
        }
    }
    
    /// Create plugin directories if they don't exist
    async fn ensure_directories(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.plugin_install_dir).await?;
        tokio::fs::create_dir_all(&self.plugin_data_dir).await?;
        tokio::fs::create_dir_all(&self.temp_dir).await?;
        Ok(())
    }
    
    /// Initialize security context for a plugin
    async fn initialize_plugin_security(&self, instance_id: PluginInstanceId, metadata: &PluginMetadata, config: &PluginConfiguration) -> Result<()> {
        self.plugin_security.initialize_context(
            instance_id,
            metadata.permissions.clone(),
            config.resource_limits.clone(),
        ).await?;
        
        // Start resource monitoring if the plugin is enabled
        if config.enabled {
            self.resource_monitor.start_monitoring(instance_id).await?;
        }
        
        Ok(())
    }
}

#[async_trait]
impl PluginService for DefaultPluginService {
    async fn initialize(&self) -> Result<()> {
        // Ensure plugin directories exist
        self.ensure_directories().await?;
        
        // Discover and load existing plugins
        let discovered_plugins = self.plugin_discovery.discover_plugins(&self.plugin_install_dir).await?;
        
        for metadata in discovered_plugins {
            // Load configuration for the plugin
            let config = match self.plugin_config.get_default_configuration(&metadata).await {
                Ok(config) => config,
                Err(_) => {
                    // Use template if available
                    if let Some(template) = self.config_template_manager.get_template(&metadata.name) {
                        template.template.clone()
                    } else {
                        PluginConfiguration {
                            enabled: false, // Default to disabled for safety
                            auto_start: false,
                            settings: std::collections::HashMap::new(),
                            resource_limits: ResourceLimits::default(),
                        }
                    }
                }
            };
            
            // Load the plugin
            let plugin_path = self.plugin_install_dir.join(&metadata.name);
            if let Ok(instance_id) = self.plugin_manager.load_plugin(&plugin_path.to_string_lossy(), config.clone()).await {
                // Initialize security context
                if let Err(e) = self.initialize_plugin_security(instance_id, &metadata, &config).await {
                    eprintln!("Failed to initialize security for plugin {}: {}", metadata.name, e);
                }
            }
        }
        
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        // Get all loaded plugins
        let plugins = self.plugin_manager.get_loaded_plugins().await?;
        
        // Shutdown all plugins
        for plugin in plugins {
            if let Err(e) = self.plugin_manager.unload_plugin(plugin.instance_id).await {
                eprintln!("Error unloading plugin {}: {}", plugin.instance_id, e);
            }
            
            // Stop resource monitoring
            if let Err(e) = self.resource_monitor.stop_monitoring(plugin.instance_id).await {
                eprintln!("Error stopping monitoring for plugin {}: {}", plugin.instance_id, e);
            }
            
            // Remove security context
            if let Err(e) = self.plugin_security.remove_context(plugin.instance_id).await {
                eprintln!("Error removing security context for plugin {}: {}", plugin.instance_id, e);
            }
        }
        
        Ok(())
    }
    
    async fn install_plugin(&self, plugin_id: &str, version: &str) -> Result<PluginInstanceId> {
        // Download and install the plugin
        let plugin_path = self.plugin_loader.install_from_marketplace(
            plugin_id,
            version,
            &self.plugin_install_dir,
        ).await?;
        
        // Load the plugin metadata
        let metadata = self.plugin_discovery.validate_plugin(&plugin_path).await?;
        
        // Get default configuration
        let config = self.plugin_config.get_default_configuration(&metadata).await?;
        
        // Load the plugin
        let instance_id = self.plugin_manager.load_plugin(&plugin_path.to_string_lossy(), config.clone()).await?;
        
        // Initialize security context
        self.initialize_plugin_security(instance_id, &metadata, &config).await?;
        
        // Save configuration
        self.plugin_config.save_configuration(instance_id, &config).await?;
        
        Ok(instance_id)
    }
    
    async fn install_plugin_from_path(&self, plugin_path: &Path, config: Option<PluginConfiguration>) -> Result<PluginInstanceId> {
        // Validate the plugin
        let metadata = self.plugin_discovery.validate_plugin(plugin_path).await?;
        
        // Use provided config or get default
        let config = if let Some(config) = config {
            config
        } else {
            self.plugin_config.get_default_configuration(&metadata).await?
        };
        
        // Load the plugin
        let instance_id = self.plugin_manager.load_plugin(&plugin_path.to_string_lossy(), config.clone()).await?;
        
        // Initialize security context
        self.initialize_plugin_security(instance_id, &metadata, &config).await?;
        
        // Save configuration
        self.plugin_config.save_configuration(instance_id, &config).await?;
        
        Ok(instance_id)
    }
    
    async fn uninstall_plugin(&self, instance_id: PluginInstanceId) -> Result<()> {
        // Stop resource monitoring
        self.resource_monitor.stop_monitoring(instance_id).await?;
        
        // Remove security context
        self.plugin_security.remove_context(instance_id).await?;
        
        // Unload the plugin
        self.plugin_manager.unload_plugin(instance_id).await?;
        
        // Remove configuration
        let config_path = self.plugin_data_dir.join("configs").join(format!("{}.json", instance_id));
        if config_path.exists() {
            tokio::fs::remove_file(config_path).await?;
        }
        
        Ok(())
    }
    
    async fn enable_plugin(&self, instance_id: PluginInstanceId) -> Result<()> {
        // Update configuration
        let mut config = self.plugin_config.load_configuration(instance_id).await?;
        config.enabled = true;
        self.plugin_config.save_configuration(instance_id, &config).await?;
        
        // Start the plugin
        self.plugin_manager.start_plugin(instance_id).await?;
        
        // Start resource monitoring
        self.resource_monitor.start_monitoring(instance_id).await?;
        
        Ok(())
    }
    
    async fn disable_plugin(&self, instance_id: PluginInstanceId) -> Result<()> {
        // Stop resource monitoring
        self.resource_monitor.stop_monitoring(instance_id).await?;
        
        // Stop the plugin
        self.plugin_manager.stop_plugin(instance_id).await?;
        
        // Update configuration
        let mut config = self.plugin_config.load_configuration(instance_id).await?;
        config.enabled = false;
        self.plugin_config.save_configuration(instance_id, &config).await?;
        
        Ok(())
    }
    
    async fn get_installed_plugins(&self) -> Result<Vec<PluginInstance>> {
        self.plugin_manager.get_loaded_plugins().await
    }
    
    async fn search_marketplace(&self, filters: PluginSearchFilters) -> Result<Vec<PluginRegistryEntry>> {
        self.plugin_discovery.search_marketplace(filters).await
    }
    
    async fn get_plugin_config(&self, instance_id: PluginInstanceId) -> Result<PluginConfiguration> {
        self.plugin_config.load_configuration(instance_id).await
    }
    
    async fn update_plugin_config(&self, instance_id: PluginInstanceId, config: PluginConfiguration) -> Result<()> {
        // Validate the configuration if we have the plugin metadata
        if let Ok(Some(instance)) = self.plugin_manager.get_plugin_instance(instance_id).await {
            self.plugin_config.validate_configuration(&instance.metadata, &config).await?;
        }
        
        // Save the configuration
        self.plugin_config.save_configuration(instance_id, &config).await?;
        
        // Update the plugin manager configuration
        self.plugin_manager.update_plugin_config(instance_id, config.clone()).await?;
        
        // Update resource limits in security
        self.plugin_security.update_resource_limits(instance_id, config.resource_limits).await?;
        
        Ok(())
    }
    
    async fn get_plugin_health(&self, instance_id: PluginInstanceId) -> Result<PluginHealth> {
        self.plugin_manager.get_plugin_health(instance_id).await
    }
    
    async fn get_plugin_resource_usage(&self, instance_id: PluginInstanceId) -> Result<ResourceUsage> {
        self.plugin_security.get_resource_usage(instance_id).await
    }
    
    async fn broadcast_event(&self, event: PluginEvent) -> Result<Vec<(PluginInstanceId, PluginResponse)>> {
        self.plugin_manager.broadcast_event(event).await
    }
    
    async fn send_event_to_plugin(&self, instance_id: PluginInstanceId, event: PluginEvent) -> Result<PluginResponse> {
        self.plugin_manager.send_event(instance_id, event).await
    }
    
    async fn grant_permission(&self, instance_id: PluginInstanceId, permission: PluginPermission) -> Result<()> {
        self.plugin_security.grant_permission(instance_id, permission).await
    }
    
    async fn revoke_permission(&self, instance_id: PluginInstanceId, permission: PluginPermission) -> Result<()> {
        self.plugin_security.revoke_permission(instance_id, &permission).await
    }
    
    async fn update_resource_limits(&self, instance_id: PluginInstanceId, limits: ResourceLimits) -> Result<()> {
        self.plugin_security.update_resource_limits(instance_id, limits).await
    }
}