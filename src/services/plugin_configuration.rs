use crate::models::plugin::{PluginConfiguration, PluginInstanceId, PluginMetadata, ResourceLimits};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Plugin configuration manager
#[async_trait]
pub trait PluginConfigurationManager: Send + Sync {
    /// Load configuration for a plugin
    async fn load_configuration(&self, instance_id: PluginInstanceId) -> Result<PluginConfiguration>;
    
    /// Save configuration for a plugin
    async fn save_configuration(&self, instance_id: PluginInstanceId, config: &PluginConfiguration) -> Result<()>;
    
    /// Get default configuration for a plugin
    async fn get_default_configuration(&self, metadata: &PluginMetadata) -> Result<PluginConfiguration>;
    
    /// Validate configuration against plugin schema
    async fn validate_configuration(&self, metadata: &PluginMetadata, config: &PluginConfiguration) -> Result<()>;
    
    /// Update specific configuration setting
    async fn update_setting(&self, instance_id: PluginInstanceId, key: &str, value: Value) -> Result<()>;
    
    /// Get specific configuration setting
    async fn get_setting(&self, instance_id: PluginInstanceId, key: &str) -> Result<Option<Value>>;
    
    /// Reset configuration to defaults
    async fn reset_to_defaults(&self, instance_id: PluginInstanceId, metadata: &PluginMetadata) -> Result<()>;
    
    /// Export configuration
    async fn export_configuration(&self, instance_id: PluginInstanceId) -> Result<String>;
    
    /// Import configuration
    async fn import_configuration(&self, instance_id: PluginInstanceId, config_data: &str) -> Result<()>;
}

/// Default implementation of plugin configuration manager
pub struct DefaultPluginConfigurationManager {
    config_directory: PathBuf,
}

impl DefaultPluginConfigurationManager {
    /// Create a new plugin configuration manager
    pub fn new(config_directory: PathBuf) -> Self {
        Self { config_directory }
    }
    
    /// Get the configuration file path for a plugin instance
    fn get_config_path(&self, instance_id: PluginInstanceId) -> PathBuf {
        self.config_directory.join(format!("{}.json", instance_id))
    }
    
    /// Ensure the configuration directory exists
    async fn ensure_config_directory(&self) -> Result<()> {
        if !self.config_directory.exists() {
            fs::create_dir_all(&self.config_directory).await?;
        }
        Ok(())
    }
    
    /// Validate configuration value against JSON schema
    fn validate_against_schema(&self, schema: &Value, config: &PluginConfiguration) -> Result<()> {
        // In a real implementation, this would use a JSON schema validator
        // For now, we'll do basic validation
        
        if let Some(schema_obj) = schema.as_object() {
            if let Some(properties) = schema_obj.get("properties") {
                if let Some(properties_obj) = properties.as_object() {
                    for (key, _schema_def) in properties_obj {
                        if !config.settings.contains_key(key) {
                            // Check if the property is required
                            if let Some(required) = schema_obj.get("required") {
                                if let Some(required_array) = required.as_array() {
                                    if required_array.iter().any(|v| v.as_str() == Some(key)) {
                                        return Err(anyhow!("Required configuration property missing: {}", key));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl PluginConfigurationManager for DefaultPluginConfigurationManager {
    async fn load_configuration(&self, instance_id: PluginInstanceId) -> Result<PluginConfiguration> {
        let config_path = self.get_config_path(instance_id);
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path).await?;
            let config: PluginConfiguration = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            // Return default configuration if file doesn't exist
            Ok(PluginConfiguration {
                enabled: true,
                auto_start: false,
                settings: HashMap::new(),
                resource_limits: ResourceLimits::default(),
            })
        }
    }
    
    async fn save_configuration(&self, instance_id: PluginInstanceId, config: &PluginConfiguration) -> Result<()> {
        self.ensure_config_directory().await?;
        
        let config_path = self.get_config_path(instance_id);
        let content = serde_json::to_string_pretty(config)?;
        fs::write(&config_path, content).await?;
        
        Ok(())
    }
    
    async fn get_default_configuration(&self, metadata: &PluginMetadata) -> Result<PluginConfiguration> {
        let mut settings = HashMap::new();
        
        // If the plugin has a configuration schema, extract default values
        if let Some(schema) = &metadata.configuration_schema {
            if let Some(schema_obj) = schema.as_object() {
                if let Some(properties) = schema_obj.get("properties") {
                    if let Some(properties_obj) = properties.as_object() {
                        for (key, property_schema) in properties_obj {
                            if let Some(default_value) = property_schema.get("default") {
                                settings.insert(key.clone(), default_value.clone());
                            }
                        }
                    }
                }
            }
        }
        
        Ok(PluginConfiguration {
            enabled: true,
            auto_start: false,
            settings,
            resource_limits: ResourceLimits::default(),
        })
    }
    
    async fn validate_configuration(&self, metadata: &PluginMetadata, config: &PluginConfiguration) -> Result<()> {
        if let Some(schema) = &metadata.configuration_schema {
            self.validate_against_schema(schema, config)?;
        }
        
        // Validate resource limits
        if let Some(max_memory) = config.resource_limits.max_memory_mb {
            if max_memory == 0 {
                return Err(anyhow!("Invalid resource limit: max_memory_mb cannot be 0"));
            }
        }
        
        if let Some(max_cpu) = config.resource_limits.max_cpu_percent {
            if max_cpu <= 0.0 || max_cpu > 100.0 {
                return Err(anyhow!("Invalid resource limit: max_cpu_percent must be between 0 and 100"));
            }
        }
        
        Ok(())
    }
    
    async fn update_setting(&self, instance_id: PluginInstanceId, key: &str, value: Value) -> Result<()> {
        let mut config = self.load_configuration(instance_id).await?;
        config.settings.insert(key.to_string(), value);
        self.save_configuration(instance_id, &config).await?;
        Ok(())
    }
    
    async fn get_setting(&self, instance_id: PluginInstanceId, key: &str) -> Result<Option<Value>> {
        let config = self.load_configuration(instance_id).await?;
        Ok(config.settings.get(key).cloned())
    }
    
    async fn reset_to_defaults(&self, instance_id: PluginInstanceId, metadata: &PluginMetadata) -> Result<()> {
        let default_config = self.get_default_configuration(metadata).await?;
        self.save_configuration(instance_id, &default_config).await?;
        Ok(())
    }
    
    async fn export_configuration(&self, instance_id: PluginInstanceId) -> Result<String> {
        let config = self.load_configuration(instance_id).await?;
        let exported = serde_json::to_string_pretty(&config)?;
        Ok(exported)
    }
    
    async fn import_configuration(&self, instance_id: PluginInstanceId, config_data: &str) -> Result<()> {
        let config: PluginConfiguration = serde_json::from_str(config_data)?;
        self.save_configuration(instance_id, &config).await?;
        Ok(())
    }
}

/// Configuration template for common plugin types
#[derive(Debug, Clone)]
pub struct ConfigurationTemplate {
    pub name: String,
    pub description: String,
    pub template: PluginConfiguration,
    pub schema: Option<Value>,
}

/// Configuration template manager
pub struct ConfigurationTemplateManager {
    templates: HashMap<String, ConfigurationTemplate>,
}

impl ConfigurationTemplateManager {
    /// Create a new configuration template manager
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
        };
        
        // Add default templates
        manager.add_default_templates();
        manager
    }
    
    /// Add default configuration templates
    fn add_default_templates(&mut self) {
        // Git integration plugin template
        let git_template = ConfigurationTemplate {
            name: "git-integration".to_string(),
            description: "Configuration template for Git integration plugins".to_string(),
            template: PluginConfiguration {
                enabled: true,
                auto_start: true,
                settings: {
                    let mut settings = HashMap::new();
                    settings.insert("watch_branches".to_string(), Value::Array(vec![Value::String("main".to_string()), Value::String("develop".to_string())]));
                    settings.insert("auto_update_context".to_string(), Value::Bool(true));
                    settings.insert("commit_message_analysis".to_string(), Value::Bool(true));
                    settings
                },
                resource_limits: ResourceLimits {
                    max_memory_mb: Some(128),
                    max_cpu_percent: Some(5.0),
                    max_execution_time: Some(std::time::Duration::from_secs(60)),
                    max_network_requests_per_minute: Some(50),
                    max_file_operations_per_minute: Some(100),
                },
            },
            schema: None,
        };
        self.templates.insert("git-integration".to_string(), git_template);
        
        // IDE integration plugin template
        let ide_template = ConfigurationTemplate {
            name: "ide-integration".to_string(),
            description: "Configuration template for IDE integration plugins".to_string(),
            template: PluginConfiguration {
                enabled: true,
                auto_start: true,
                settings: {
                    let mut settings = HashMap::new();
                    settings.insert("real_time_updates".to_string(), Value::Bool(true));
                    settings.insert("code_analysis_depth".to_string(), Value::String("moderate".to_string()));
                    settings.insert("suggestion_frequency".to_string(), Value::String("on_save".to_string()));
                    settings
                },
                resource_limits: ResourceLimits {
                    max_memory_mb: Some(256),
                    max_cpu_percent: Some(10.0),
                    max_execution_time: Some(std::time::Duration::from_secs(30)),
                    max_network_requests_per_minute: Some(20),
                    max_file_operations_per_minute: Some(200),
                },
            },
            schema: None,
        };
        self.templates.insert("ide-integration".to_string(), ide_template);
    }
    
    /// Add a configuration template
    pub fn add_template(&mut self, template: ConfigurationTemplate) {
        self.templates.insert(template.name.clone(), template);
    }
    
    /// Get a configuration template by name
    pub fn get_template(&self, name: &str) -> Option<&ConfigurationTemplate> {
        self.templates.get(name)
    }
    
    /// List all available templates
    pub fn list_templates(&self) -> Vec<&ConfigurationTemplate> {
        self.templates.values().collect()
    }
    
    /// Create configuration from template
    pub fn create_from_template(&self, template_name: &str, overrides: Option<HashMap<String, Value>>) -> Result<PluginConfiguration> {
        if let Some(template) = self.templates.get(template_name) {
            let mut config = template.template.clone();
            
            // Apply overrides if provided
            if let Some(overrides) = overrides {
                for (key, value) in overrides {
                    config.settings.insert(key, value);
                }
            }
            
            Ok(config)
        } else {
            Err(anyhow!("Configuration template not found: {}", template_name))
        }
    }
}

impl Default for ConfigurationTemplateManager {
    fn default() -> Self {
        Self::new()
    }
}