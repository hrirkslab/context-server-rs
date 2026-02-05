use crate::models::plugin::{PluginEvent, PluginInstanceId, PluginResponse};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

/// Event system for plugin communication
#[async_trait]
pub trait PluginEventSystem: Send + Sync {
    /// Register a plugin to receive events
    async fn register_plugin(&self, instance_id: PluginInstanceId) -> Result<()>;
    
    /// Unregister a plugin from receiving events
    async fn unregister_plugin(&self, instance_id: PluginInstanceId) -> Result<()>;
    
    /// Emit an event to all registered plugins
    async fn emit_event(&self, event: PluginEvent) -> Result<Vec<(PluginInstanceId, PluginResponse)>>;
    
    /// Emit an event to a specific plugin
    async fn emit_event_to_plugin(&self, instance_id: PluginInstanceId, event: PluginEvent) -> Result<PluginResponse>;
    
    /// Subscribe to events of a specific type
    async fn subscribe_to_event_type(&self, instance_id: PluginInstanceId, event_type: EventType) -> Result<()>;
    
    /// Unsubscribe from events of a specific type
    async fn unsubscribe_from_event_type(&self, instance_id: PluginInstanceId, event_type: EventType) -> Result<()>;
    
    /// Get event statistics
    async fn get_event_stats(&self) -> Result<EventStats>;
}

/// Event type enumeration for filtering
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventType {
    ContextCreated,
    ContextUpdated,
    ContextDeleted,
    ProjectCreated,
    ProjectUpdated,
    QueryExecuted,
    SystemStartup,
    SystemShutdown,
    Custom(String),
}

impl From<&PluginEvent> for EventType {
    fn from(event: &PluginEvent) -> Self {
        match event {
            PluginEvent::ContextCreated { .. } => EventType::ContextCreated,
            PluginEvent::ContextUpdated { .. } => EventType::ContextUpdated,
            PluginEvent::ContextDeleted { .. } => EventType::ContextDeleted,
            PluginEvent::ProjectCreated { .. } => EventType::ProjectCreated,
            PluginEvent::ProjectUpdated { .. } => EventType::ProjectUpdated,
            PluginEvent::QueryExecuted { .. } => EventType::QueryExecuted,
            PluginEvent::SystemStartup => EventType::SystemStartup,
            PluginEvent::SystemShutdown => EventType::SystemShutdown,
            PluginEvent::Custom { event_type, .. } => EventType::Custom(event_type.clone()),
        }
    }
}

/// Event statistics
#[derive(Debug, Clone)]
pub struct EventStats {
    pub total_events_emitted: u64,
    pub events_by_type: HashMap<EventType, u64>,
    pub plugin_response_stats: HashMap<PluginInstanceId, PluginStats>,
    pub average_response_time_ms: f64,
}

/// Plugin-specific statistics
#[derive(Debug, Clone)]
pub struct PluginStats {
    pub events_received: u64,
    pub successful_responses: u64,
    pub error_responses: u64,
    pub average_response_time_ms: f64,
}

/// Event subscription information
#[derive(Debug, Clone)]
struct EventSubscription {
    instance_id: PluginInstanceId,
    event_types: Vec<EventType>,
}

/// Default implementation of the plugin event system
pub struct DefaultPluginEventSystem {
    /// Event broadcaster
    event_sender: broadcast::Sender<(PluginEvent, Option<PluginInstanceId>)>,
    
    /// Plugin subscriptions
    subscriptions: Arc<RwLock<HashMap<PluginInstanceId, Vec<EventType>>>>,
    
    /// Event statistics
    stats: Arc<RwLock<EventStats>>,
    
    /// Plugin manager reference for sending events
    plugin_manager: Arc<dyn crate::services::PluginManager>,
}

impl DefaultPluginEventSystem {
    /// Create a new plugin event system
    pub fn new(plugin_manager: Arc<dyn crate::services::PluginManager>) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            event_sender,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(EventStats {
                total_events_emitted: 0,
                events_by_type: HashMap::new(),
                plugin_response_stats: HashMap::new(),
                average_response_time_ms: 0.0,
            })),
            plugin_manager,
        }
    }
    
    /// Check if a plugin is subscribed to an event type
    async fn is_subscribed(&self, instance_id: PluginInstanceId, event_type: &EventType) -> bool {
        let subscriptions = self.subscriptions.read().await;
        if let Some(plugin_subscriptions) = subscriptions.get(&instance_id) {
            plugin_subscriptions.contains(event_type)
        } else {
            false
        }
    }
    
    /// Update event statistics
    async fn update_stats(&self, event: &PluginEvent, responses: &[(PluginInstanceId, PluginResponse)]) {
        let mut stats = self.stats.write().await;
        
        stats.total_events_emitted += 1;
        
        let event_type = EventType::from(event);
        *stats.events_by_type.entry(event_type).or_insert(0) += 1;
        
        for (instance_id, response) in responses {
            let plugin_stats = stats.plugin_response_stats.entry(*instance_id).or_insert(PluginStats {
                events_received: 0,
                successful_responses: 0,
                error_responses: 0,
                average_response_time_ms: 0.0,
            });
            
            plugin_stats.events_received += 1;
            
            match response {
                PluginResponse::Success | PluginResponse::ContextContribution(_) | PluginResponse::EventHandled => {
                    plugin_stats.successful_responses += 1;
                }
                PluginResponse::Error(_) => {
                    plugin_stats.error_responses += 1;
                }
                PluginResponse::EventIgnored => {
                    // Don't count as success or error
                }
            }
        }
    }
}

#[async_trait]
impl PluginEventSystem for DefaultPluginEventSystem {
    async fn register_plugin(&self, instance_id: PluginInstanceId) -> Result<()> {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(instance_id, Vec::new());
        Ok(())
    }
    
    async fn unregister_plugin(&self, instance_id: PluginInstanceId) -> Result<()> {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.remove(&instance_id);
        Ok(())
    }
    
    async fn emit_event(&self, event: PluginEvent) -> Result<Vec<(PluginInstanceId, PluginResponse)>> {
        let event_type = EventType::from(&event);
        let subscriptions = self.subscriptions.read().await;
        let mut responses = Vec::new();
        
        // Find all plugins subscribed to this event type
        for (instance_id, plugin_event_types) in subscriptions.iter() {
            if plugin_event_types.is_empty() || plugin_event_types.contains(&event_type) {
                // Send event to plugin
                match self.plugin_manager.send_event(*instance_id, event.clone()).await {
                    Ok(response) => {
                        responses.push((*instance_id, response));
                    }
                    Err(e) => {
                        eprintln!("Error sending event to plugin {}: {}", instance_id, e);
                        responses.push((*instance_id, PluginResponse::Error(e.to_string())));
                    }
                }
            }
        }
        
        // Update statistics
        self.update_stats(&event, &responses).await;
        
        // Broadcast event to any listeners
        let _ = self.event_sender.send((event, None));
        
        Ok(responses)
    }
    
    async fn emit_event_to_plugin(&self, instance_id: PluginInstanceId, event: PluginEvent) -> Result<PluginResponse> {
        let response = self.plugin_manager.send_event(instance_id, event.clone()).await?;
        
        // Update statistics
        self.update_stats(&event, &[(instance_id, response.clone())]).await;
        
        // Broadcast event to any listeners
        let _ = self.event_sender.send((event, Some(instance_id)));
        
        Ok(response)
    }
    
    async fn subscribe_to_event_type(&self, instance_id: PluginInstanceId, event_type: EventType) -> Result<()> {
        let mut subscriptions = self.subscriptions.write().await;
        let plugin_subscriptions = subscriptions.entry(instance_id).or_insert_with(Vec::new);
        
        if !plugin_subscriptions.contains(&event_type) {
            plugin_subscriptions.push(event_type);
        }
        
        Ok(())
    }
    
    async fn unsubscribe_from_event_type(&self, instance_id: PluginInstanceId, event_type: EventType) -> Result<()> {
        let mut subscriptions = self.subscriptions.write().await;
        if let Some(plugin_subscriptions) = subscriptions.get_mut(&instance_id) {
            plugin_subscriptions.retain(|t| t != &event_type);
        }
        
        Ok(())
    }
    
    async fn get_event_stats(&self) -> Result<EventStats> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }
}

/// Event listener for external systems
pub struct PluginEventListener {
    receiver: broadcast::Receiver<(PluginEvent, Option<PluginInstanceId>)>,
}

impl PluginEventListener {
    /// Create a new event listener
    pub fn new(event_system: &DefaultPluginEventSystem) -> Self {
        let receiver = event_system.event_sender.subscribe();
        Self { receiver }
    }
    
    /// Wait for the next event
    pub async fn next_event(&mut self) -> Result<(PluginEvent, Option<PluginInstanceId>)> {
        loop {
            match self.receiver.recv().await {
                Ok(event) => return Ok(event),
                Err(broadcast::error::RecvError::Closed) => {
                    return Err(anyhow::anyhow!("Event channel closed"));
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    eprintln!("Event listener lagged, skipped {} events", skipped);
                    // Continue the loop to try receiving again
                    continue;
                }
            }
        }
    }
}

/// Event filter for selective event processing
#[derive(Debug, Clone)]
pub struct EventFilter {
    pub event_types: Option<Vec<EventType>>,
    pub project_ids: Option<Vec<String>>,
    pub context_types: Option<Vec<String>>,
}

impl EventFilter {
    /// Create a new event filter
    pub fn new() -> Self {
        Self {
            event_types: None,
            project_ids: None,
            context_types: None,
        }
    }
    
    /// Filter by event types
    pub fn with_event_types(mut self, event_types: Vec<EventType>) -> Self {
        self.event_types = Some(event_types);
        self
    }
    
    /// Filter by project IDs
    pub fn with_project_ids(mut self, project_ids: Vec<String>) -> Self {
        self.project_ids = Some(project_ids);
        self
    }
    
    /// Filter by context types
    pub fn with_context_types(mut self, context_types: Vec<String>) -> Self {
        self.context_types = Some(context_types);
        self
    }
    
    /// Check if an event matches this filter
    pub fn matches(&self, event: &PluginEvent) -> bool {
        // Check event type filter
        if let Some(ref allowed_types) = self.event_types {
            let event_type = EventType::from(event);
            if !allowed_types.contains(&event_type) {
                return false;
            }
        }
        
        // Check project ID filter
        if let Some(ref allowed_projects) = self.project_ids {
            let project_id = match event {
                PluginEvent::ContextCreated { project_id, .. } |
                PluginEvent::ContextUpdated { project_id, .. } |
                PluginEvent::ContextDeleted { project_id, .. } => Some(project_id),
                _ => None,
            };
            
            if let Some(project_id) = project_id {
                if !allowed_projects.contains(project_id) {
                    return false;
                }
            }
        }
        
        // Check context type filter
        if let Some(ref allowed_context_types) = self.context_types {
            let context_type = match event {
                PluginEvent::ContextCreated { context_type, .. } |
                PluginEvent::ContextUpdated { context_type, .. } |
                PluginEvent::ContextDeleted { context_type, .. } => Some(context_type),
                _ => None,
            };
            
            if let Some(context_type) = context_type {
                if !allowed_context_types.contains(context_type) {
                    return false;
                }
            }
        }
        
        true
    }
}

impl Default for EventFilter {
    fn default() -> Self {
        Self::new()
    }
}