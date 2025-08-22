use crate::models::plugin::{PluginPermission, ResourceLimits, ResourceUsage, PluginInstanceId};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Security manager for plugin sandboxing and resource management
#[async_trait]
pub trait PluginSecurity: Send + Sync {
    /// Check if a plugin has the required permission
    async fn check_permission(&self, instance_id: PluginInstanceId, permission: &PluginPermission) -> Result<bool>;
    
    /// Grant a permission to a plugin
    async fn grant_permission(&self, instance_id: PluginInstanceId, permission: PluginPermission) -> Result<()>;
    
    /// Revoke a permission from a plugin
    async fn revoke_permission(&self, instance_id: PluginInstanceId, permission: &PluginPermission) -> Result<()>;
    
    /// Check if a plugin is within resource limits
    async fn check_resource_limits(&self, instance_id: PluginInstanceId, current_usage: &ResourceUsage) -> Result<bool>;
    
    /// Update resource limits for a plugin
    async fn update_resource_limits(&self, instance_id: PluginInstanceId, limits: ResourceLimits) -> Result<()>;
    
    /// Get current resource usage for a plugin
    async fn get_resource_usage(&self, instance_id: PluginInstanceId) -> Result<ResourceUsage>;
    
    /// Start resource monitoring for a plugin
    async fn start_monitoring(&self, instance_id: PluginInstanceId) -> Result<()>;
    
    /// Stop resource monitoring for a plugin
    async fn stop_monitoring(&self, instance_id: PluginInstanceId) -> Result<()>;
    
    /// Terminate a plugin if it exceeds resource limits
    async fn enforce_limits(&self, instance_id: PluginInstanceId) -> Result<()>;
    
    /// Initialize security context for a plugin
    async fn initialize_context(&self, instance_id: PluginInstanceId, permissions: Vec<PluginPermission>, limits: ResourceLimits) -> Result<()>;
    
    /// Remove security context for a plugin
    async fn remove_context(&self, instance_id: PluginInstanceId) -> Result<()>;
}

/// Plugin security context
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub instance_id: PluginInstanceId,
    pub granted_permissions: Vec<PluginPermission>,
    pub resource_limits: ResourceLimits,
    pub current_usage: ResourceUsage,
    pub monitoring_started: Option<Instant>,
}

/// Default implementation of plugin security
pub struct DefaultPluginSecurity {
    security_contexts: Arc<RwLock<HashMap<PluginInstanceId, SecurityContext>>>,
    monitoring_interval: Duration,
}

impl DefaultPluginSecurity {
    /// Create a new plugin security manager
    pub fn new(monitoring_interval: Duration) -> Self {
        Self {
            security_contexts: Arc::new(RwLock::new(HashMap::new())),
            monitoring_interval,
        }
    }
    

    
    /// Update resource usage for a plugin
    async fn update_usage(&self, instance_id: PluginInstanceId, usage: ResourceUsage) -> Result<()> {
        let mut contexts = self.security_contexts.write().await;
        if let Some(context) = contexts.get_mut(&instance_id) {
            context.current_usage = usage;
        }
        Ok(())
    }
    
    /// Check if resource usage exceeds limits
    fn exceeds_limits(usage: &ResourceUsage, limits: &ResourceLimits) -> bool {
        if let Some(max_memory) = limits.max_memory_mb {
            if usage.memory_mb > max_memory as f64 {
                return true;
            }
        }
        
        if let Some(max_cpu) = limits.max_cpu_percent {
            if usage.cpu_percent > max_cpu {
                return true;
            }
        }
        
        if let Some(max_network) = limits.max_network_requests_per_minute {
            if usage.network_requests_count > max_network {
                return true;
            }
        }
        
        if let Some(max_file_ops) = limits.max_file_operations_per_minute {
            if usage.file_operations_count > max_file_ops {
                return true;
            }
        }
        
        false
    }
}

#[async_trait]
impl PluginSecurity for DefaultPluginSecurity {
    async fn check_permission(&self, instance_id: PluginInstanceId, permission: &PluginPermission) -> Result<bool> {
        let contexts = self.security_contexts.read().await;
        if let Some(context) = contexts.get(&instance_id) {
            Ok(context.granted_permissions.contains(permission))
        } else {
            Err(anyhow!("Security context not found for plugin: {}", instance_id))
        }
    }
    
    async fn grant_permission(&self, instance_id: PluginInstanceId, permission: PluginPermission) -> Result<()> {
        let mut contexts = self.security_contexts.write().await;
        if let Some(context) = contexts.get_mut(&instance_id) {
            if !context.granted_permissions.contains(&permission) {
                context.granted_permissions.push(permission);
            }
            Ok(())
        } else {
            Err(anyhow!("Security context not found for plugin: {}", instance_id))
        }
    }
    
    async fn revoke_permission(&self, instance_id: PluginInstanceId, permission: &PluginPermission) -> Result<()> {
        let mut contexts = self.security_contexts.write().await;
        if let Some(context) = contexts.get_mut(&instance_id) {
            context.granted_permissions.retain(|p| p != permission);
            Ok(())
        } else {
            Err(anyhow!("Security context not found for plugin: {}", instance_id))
        }
    }
    
    async fn check_resource_limits(&self, instance_id: PluginInstanceId, current_usage: &ResourceUsage) -> Result<bool> {
        let contexts = self.security_contexts.read().await;
        if let Some(context) = contexts.get(&instance_id) {
            Ok(!Self::exceeds_limits(current_usage, &context.resource_limits))
        } else {
            Err(anyhow!("Security context not found for plugin: {}", instance_id))
        }
    }
    
    async fn update_resource_limits(&self, instance_id: PluginInstanceId, limits: ResourceLimits) -> Result<()> {
        let mut contexts = self.security_contexts.write().await;
        if let Some(context) = contexts.get_mut(&instance_id) {
            context.resource_limits = limits;
            Ok(())
        } else {
            Err(anyhow!("Security context not found for plugin: {}", instance_id))
        }
    }
    
    async fn get_resource_usage(&self, instance_id: PluginInstanceId) -> Result<ResourceUsage> {
        let contexts = self.security_contexts.read().await;
        if let Some(context) = contexts.get(&instance_id) {
            Ok(context.current_usage.clone())
        } else {
            Err(anyhow!("Security context not found for plugin: {}", instance_id))
        }
    }
    
    async fn start_monitoring(&self, instance_id: PluginInstanceId) -> Result<()> {
        let mut contexts = self.security_contexts.write().await;
        if let Some(context) = contexts.get_mut(&instance_id) {
            context.monitoring_started = Some(Instant::now());
            Ok(())
        } else {
            Err(anyhow!("Security context not found for plugin: {}", instance_id))
        }
    }
    
    async fn stop_monitoring(&self, instance_id: PluginInstanceId) -> Result<()> {
        let mut contexts = self.security_contexts.write().await;
        if let Some(context) = contexts.get_mut(&instance_id) {
            context.monitoring_started = None;
            Ok(())
        } else {
            Err(anyhow!("Security context not found for plugin: {}", instance_id))
        }
    }
    
    async fn enforce_limits(&self, instance_id: PluginInstanceId) -> Result<()> {
        let contexts = self.security_contexts.read().await;
        if let Some(context) = contexts.get(&instance_id) {
            if Self::exceeds_limits(&context.current_usage, &context.resource_limits) {
                // In a real implementation, this would terminate or throttle the plugin
                eprintln!("Plugin {} exceeds resource limits: {:?}", instance_id, context.current_usage);
                return Err(anyhow!("Plugin {} exceeds resource limits", instance_id));
            }
        }
        Ok(())
    }
    
    async fn initialize_context(&self, instance_id: PluginInstanceId, permissions: Vec<PluginPermission>, limits: ResourceLimits) -> Result<()> {
        let context = SecurityContext {
            instance_id,
            granted_permissions: permissions,
            resource_limits: limits,
            current_usage: ResourceUsage::default(),
            monitoring_started: None,
        };
        
        let mut contexts = self.security_contexts.write().await;
        contexts.insert(instance_id, context);
        
        Ok(())
    }
    
    async fn remove_context(&self, instance_id: PluginInstanceId) -> Result<()> {
        let mut contexts = self.security_contexts.write().await;
        contexts.remove(&instance_id);
        Ok(())
    }
}

/// Resource monitor for tracking plugin resource usage
pub struct ResourceMonitor {
    security: Arc<dyn PluginSecurity>,
    monitoring_interval: Duration,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new(security: Arc<dyn PluginSecurity>, monitoring_interval: Duration) -> Self {
        Self {
            security,
            monitoring_interval,
        }
    }
    
    /// Start monitoring a plugin's resource usage
    pub async fn start_monitoring(&self, instance_id: PluginInstanceId) -> Result<()> {
        self.security.start_monitoring(instance_id).await?;
        
        // In a real implementation, this would start a background task to monitor resources
        // For now, we'll just log that monitoring started
        println!("Started resource monitoring for plugin: {}", instance_id);
        
        Ok(())
    }
    
    /// Stop monitoring a plugin's resource usage
    pub async fn stop_monitoring(&self, instance_id: PluginInstanceId) -> Result<()> {
        self.security.stop_monitoring(instance_id).await?;
        println!("Stopped resource monitoring for plugin: {}", instance_id);
        Ok(())
    }
    
    /// Get current resource usage for a plugin
    pub async fn get_current_usage(&self, instance_id: PluginInstanceId) -> Result<ResourceUsage> {
        // In a real implementation, this would collect actual system metrics
        // For now, return mock data
        Ok(ResourceUsage {
            memory_mb: 50.0,
            cpu_percent: 2.5,
            network_requests_count: 10,
            file_operations_count: 5,
            last_updated: chrono::Utc::now(),
        })
    }
}

/// Permission validator for checking plugin permissions
pub struct PermissionValidator {
    security: Arc<dyn PluginSecurity>,
}

impl PermissionValidator {
    /// Create a new permission validator
    pub fn new(security: Arc<dyn PluginSecurity>) -> Self {
        Self { security }
    }
    
    /// Validate that a plugin has the required permissions for an operation
    pub async fn validate_operation(&self, instance_id: PluginInstanceId, required_permissions: &[PluginPermission]) -> Result<()> {
        for permission in required_permissions {
            if !self.security.check_permission(instance_id, permission).await? {
                return Err(anyhow!("Plugin {} lacks required permission: {:?}", instance_id, permission));
            }
        }
        Ok(())
    }
    
    /// Check if a plugin can perform a context operation
    pub async fn can_access_context(&self, instance_id: PluginInstanceId, write_access: bool) -> Result<bool> {
        let read_permission = self.security.check_permission(instance_id, &PluginPermission::ReadContext).await?;
        
        if write_access {
            let write_permission = self.security.check_permission(instance_id, &PluginPermission::WriteContext).await?;
            Ok(read_permission && write_permission)
        } else {
            Ok(read_permission)
        }
    }
    
    /// Check if a plugin can access the network
    pub async fn can_access_network(&self, instance_id: PluginInstanceId) -> Result<bool> {
        self.security.check_permission(instance_id, &PluginPermission::NetworkAccess).await
    }
    
    /// Check if a plugin can access the file system
    pub async fn can_access_filesystem(&self, instance_id: PluginInstanceId, write_access: bool) -> Result<bool> {
        let read_permission = self.security.check_permission(instance_id, &PluginPermission::FileSystemRead).await?;
        
        if write_access {
            let write_permission = self.security.check_permission(instance_id, &PluginPermission::FileSystemWrite).await?;
            Ok(read_permission && write_permission)
        } else {
            Ok(read_permission)
        }
    }
}