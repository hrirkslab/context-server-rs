use crate::services::websocket_types::*;
use anyhow::{anyhow, Result};
use chrono::Utc;
use dashmap::DashMap;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Change broadcasting system for real-time synchronization
#[derive(Clone)]
pub struct ChangeBroadcaster {
    /// Broadcast channel for sending changes to all subscribers
    change_sender: broadcast::Sender<ContextChange>,
    /// Client subscriptions with filters
    pub subscriptions: Arc<DashMap<ClientId, Vec<SyncFilters>>>,
    /// Change queue for reliable delivery
    pub change_queue: Arc<DashMap<ClientId, Vec<QueuedChange>>>,
    /// Change history for delta calculation
    pub change_history: Arc<DashMap<String, ChangeHistory>>,
    /// Metrics for monitoring
    metrics: Arc<BroadcastMetrics>,
}

/// Queued change for reliable delivery
#[derive(Debug, Clone)]
pub struct QueuedChange {
    pub change_id: Uuid,
    pub change: ContextChange,
    pub queued_at: chrono::DateTime<chrono::Utc>,
    pub retry_count: u32,
    pub target_clients: Vec<ClientId>,
}

/// Change history for delta calculation
#[derive(Debug, Clone)]
pub struct ChangeHistory {
    entity_id: String,
    entity_type: String,
    versions: Vec<VersionedChange>,
    last_updated: chrono::DateTime<chrono::Utc>,
}

/// Versioned change for history tracking
#[derive(Debug, Clone)]
struct VersionedChange {
    version: u32,
    change: ContextChange,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Broadcasting metrics
#[derive(Debug, Default)]
pub struct BroadcastMetrics {
    pub total_changes_broadcast: std::sync::atomic::AtomicU64,
    pub total_clients_notified: std::sync::atomic::AtomicU64,
    pub failed_deliveries: std::sync::atomic::AtomicU64,
    pub delta_calculations: std::sync::atomic::AtomicU64,
    pub queue_size: std::sync::atomic::AtomicU64,
}

/// Change event for internal processing
#[derive(Debug, Clone)]
pub struct ChangeEvent {
    pub entity_type: String,
    pub entity_id: String,
    pub project_id: String,
    pub change_type: ChangeType,
    pub old_value: Option<Value>,
    pub new_value: Option<Value>,
    pub client_id: ClientId,
    pub feature_area: Option<String>,
}

impl ChangeBroadcaster {
    /// Create a new change broadcaster
    pub fn new() -> Self {
        let (change_sender, _) = broadcast::channel(1000);
        
        Self {
            change_sender,
            subscriptions: Arc::new(DashMap::new()),
            change_queue: Arc::new(DashMap::new()),
            change_history: Arc::new(DashMap::new()),
            metrics: Arc::new(BroadcastMetrics::default()),
        }
    }

    /// Start the change broadcaster with background processing
    pub async fn start(&self) -> Result<()> {
        info!("Starting change broadcaster");
        
        // Start queue processing task
        self.start_queue_processing().await;
        
        // Start metrics collection task
        self.start_metrics_collection().await;
        
        Ok(())
    }

    /// Subscribe a client to changes with filters
    pub async fn subscribe(&self, client_id: ClientId, filters: Vec<SyncFilters>) -> Result<()> {
        debug!("Client {} subscribing with {} filters", client_id, filters.len());
        
        self.subscriptions.insert(client_id, filters);
        
        // Initialize change queue for this client
        self.change_queue.insert(client_id, Vec::new());
        
        Ok(())
    }

    /// Unsubscribe a client from changes
    pub async fn unsubscribe(&self, client_id: ClientId) -> Result<()> {
        debug!("Client {} unsubscribing", client_id);
        
        self.subscriptions.remove(&client_id);
        self.change_queue.remove(&client_id);
        
        Ok(())
    }

    /// Update client subscription filters
    pub async fn update_subscription(&self, client_id: ClientId, filters: Vec<SyncFilters>) -> Result<()> {
        debug!("Updating subscription for client {} with {} filters", client_id, filters.len());
        
        if let Some(mut subscription) = self.subscriptions.get_mut(&client_id) {
            *subscription = filters;
        } else {
            return Err(anyhow!("Client {} not found in subscriptions", client_id));
        }
        
        Ok(())
    }

    /// Broadcast a change event to all subscribed clients
    pub async fn broadcast_change(&self, event: ChangeEvent) -> Result<()> {
        debug!("Broadcasting change for entity {}/{}", event.entity_type, event.entity_id);
        
        // Calculate delta if this is an update
        let delta = if event.change_type == ChangeType::Update {
            self.calculate_delta(&event).await?
        } else {
            None
        };

        // Create context change
        let context_change = ContextChange {
            change_id: Uuid::new_v4(),
            change_type: event.change_type.clone(),
            entity_type: event.entity_type.clone(),
            entity_id: event.entity_id.clone(),
            project_id: event.project_id.clone(),
            feature_area: event.feature_area.clone(),
            delta,
            full_entity: event.new_value.clone(),
            metadata: ChangeMetadata {
                user_id: None,
                client_id: event.client_id,
                timestamp: Utc::now(),
                version: self.get_next_version(&event.entity_id).await,
                conflict_resolution: None,
            },
        };

        // Update change history
        self.update_change_history(&context_change).await;

        // Find matching clients
        let matching_clients = self.find_matching_clients(&context_change).await;
        
        if matching_clients.is_empty() {
            debug!("No clients match filters for change {}", context_change.change_id);
            return Ok(());
        }

        // Try to broadcast immediately
        let immediate_success = self.try_immediate_broadcast(&context_change).await;
        
        // Queue for clients that couldn't receive immediately
        if !immediate_success {
            self.queue_change(&context_change, &matching_clients).await?;
        }

        // Update metrics
        self.metrics.total_changes_broadcast.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.metrics.total_clients_notified.fetch_add(matching_clients.len() as u64, std::sync::atomic::Ordering::Relaxed);

        Ok(())
    }

    /// Calculate delta between old and new values
    pub async fn calculate_delta(&self, event: &ChangeEvent) -> Result<Option<Value>> {
        self.metrics.delta_calculations.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        let old_value = match &event.old_value {
            Some(val) => val,
            None => return Ok(None),
        };

        let new_value = match &event.new_value {
            Some(val) => val,
            None => return Ok(None),
        };

        // Simple delta calculation - in production, use a more sophisticated diff algorithm
        let delta = serde_json::json!({
            "old": old_value,
            "new": new_value,
            "changed_fields": self.find_changed_fields(old_value, new_value)
        });

        Ok(Some(delta))
    }

    /// Find changed fields between two JSON values
    fn find_changed_fields(&self, old: &Value, new: &Value) -> Vec<String> {
        let mut changed_fields = Vec::new();
        
        match (old, new) {
            (Value::Object(old_obj), Value::Object(new_obj)) => {
                // Check for changed or new fields
                for (key, new_val) in new_obj {
                    if let Some(old_val) = old_obj.get(key) {
                        if old_val != new_val {
                            changed_fields.push(key.clone());
                        }
                    } else {
                        changed_fields.push(key.clone());
                    }
                }
                
                // Check for removed fields
                for key in old_obj.keys() {
                    if !new_obj.contains_key(key) {
                        changed_fields.push(format!("removed_{}", key));
                    }
                }
            }
            _ => {
                // For non-object values, consider the entire value changed
                changed_fields.push("value".to_string());
            }
        }
        
        changed_fields
    }

    /// Find clients that match the change filters
    pub async fn find_matching_clients(&self, change: &ContextChange) -> Vec<ClientId> {
        let mut matching_clients = Vec::new();
        
        for subscription in self.subscriptions.iter() {
            let client_id = *subscription.key();
            let filters = subscription.value();
            
            // Check if any filter matches
            let matches = filters.iter().any(|filter| filter.matches(change));
            
            if matches {
                matching_clients.push(client_id);
            }
        }
        
        matching_clients
    }

    /// Try to broadcast change immediately
    async fn try_immediate_broadcast(&self, change: &ContextChange) -> bool {
        match self.change_sender.send(change.clone()) {
            Ok(receiver_count) => {
                debug!("Immediately broadcast change {} to {} receivers", change.change_id, receiver_count);
                true
            }
            Err(_) => {
                warn!("Failed to immediately broadcast change {}", change.change_id);
                false
            }
        }
    }

    /// Queue change for reliable delivery
    pub async fn queue_change(&self, change: &ContextChange, target_clients: &[ClientId]) -> Result<()> {
        let queued_change = QueuedChange {
            change_id: change.change_id,
            change: change.clone(),
            queued_at: Utc::now(),
            retry_count: 0,
            target_clients: target_clients.to_vec(),
        };

        for &client_id in target_clients {
            if let Some(mut queue) = self.change_queue.get_mut(&client_id) {
                queue.push(queued_change.clone());
                self.metrics.queue_size.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }

        debug!("Queued change {} for {} clients", change.change_id, target_clients.len());
        Ok(())
    }

    /// Update change history for delta calculation
    async fn update_change_history(&self, change: &ContextChange) {
        let history_key = format!("{}:{}", change.entity_type, change.entity_id);
        
        let versioned_change = VersionedChange {
            version: change.metadata.version,
            change: change.clone(),
            timestamp: change.metadata.timestamp,
        };

        if let Some(mut history) = self.change_history.get_mut(&history_key) {
            history.versions.push(versioned_change);
            history.last_updated = Utc::now();
            
            // Keep only last 10 versions to prevent memory bloat
            if history.versions.len() > 10 {
                history.versions.remove(0);
            }
        } else {
            let new_history = ChangeHistory {
                entity_id: change.entity_id.clone(),
                entity_type: change.entity_type.clone(),
                versions: vec![versioned_change],
                last_updated: Utc::now(),
            };
            self.change_history.insert(history_key, new_history);
        }
    }

    /// Get next version number for an entity
    async fn get_next_version(&self, entity_id: &str) -> u32 {
        // Simple version increment - in production, use database sequence
        let _history_key = format!("*:{}", entity_id); // Wildcard for entity type
        
        let mut max_version = 0;
        for history in self.change_history.iter() {
            if history.key().ends_with(&format!(":{}", entity_id)) {
                if let Some(last_version) = history.versions.last() {
                    max_version = max_version.max(last_version.version);
                }
            }
        }
        
        max_version + 1
    }

    /// Get broadcast receiver for listening to changes
    pub fn subscribe_to_changes(&self) -> broadcast::Receiver<ContextChange> {
        self.change_sender.subscribe()
    }

    /// Get queued changes for a client
    pub async fn get_queued_changes(&self, client_id: ClientId) -> Vec<QueuedChange> {
        self.change_queue.get(&client_id)
            .map(|queue| queue.clone())
            .unwrap_or_default()
    }

    /// Remove queued change after successful delivery
    pub async fn acknowledge_change(&self, client_id: ClientId, change_id: Uuid) -> Result<()> {
        if let Some(mut queue) = self.change_queue.get_mut(&client_id) {
            let initial_len = queue.len();
            queue.retain(|queued| queued.change_id != change_id);
            
            if queue.len() < initial_len {
                self.metrics.queue_size.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                debug!("Acknowledged change {} for client {}", change_id, client_id);
            }
        }
        
        Ok(())
    }

    /// Get broadcasting metrics
    pub fn get_metrics(&self) -> BroadcastMetrics {
        BroadcastMetrics {
            total_changes_broadcast: std::sync::atomic::AtomicU64::new(
                self.metrics.total_changes_broadcast.load(std::sync::atomic::Ordering::Relaxed)
            ),
            total_clients_notified: std::sync::atomic::AtomicU64::new(
                self.metrics.total_clients_notified.load(std::sync::atomic::Ordering::Relaxed)
            ),
            failed_deliveries: std::sync::atomic::AtomicU64::new(
                self.metrics.failed_deliveries.load(std::sync::atomic::Ordering::Relaxed)
            ),
            delta_calculations: std::sync::atomic::AtomicU64::new(
                self.metrics.delta_calculations.load(std::sync::atomic::Ordering::Relaxed)
            ),
            queue_size: std::sync::atomic::AtomicU64::new(
                self.metrics.queue_size.load(std::sync::atomic::Ordering::Relaxed)
            ),
        }
    }

    /// Start queue processing background task
    async fn start_queue_processing(&self) {
        let change_queue = self.change_queue.clone();
        let change_sender = self.change_sender.clone();
        let metrics = self.metrics.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));

            loop {
                interval.tick().await;

                for mut queue_entry in change_queue.iter_mut() {
                    let client_id = *queue_entry.key();
                    let queue = queue_entry.value_mut();

                    let mut to_remove = Vec::new();

                    for (index, queued_change) in queue.iter_mut().enumerate() {
                        // Try to resend the change
                        match change_sender.send(queued_change.change.clone()) {
                            Ok(_) => {
                                to_remove.push(index);
                                debug!("Successfully resent queued change {} to client {}", 
                                      queued_change.change_id, client_id);
                            }
                            Err(_) => {
                                queued_change.retry_count += 1;
                                
                                // Remove changes that have been retried too many times
                                if queued_change.retry_count > 5 {
                                    to_remove.push(index);
                                    metrics.failed_deliveries.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                    warn!("Dropping change {} for client {} after {} retries", 
                                          queued_change.change_id, client_id, queued_change.retry_count);
                                }
                            }
                        }
                    }

                    // Remove successfully sent or expired changes
                    for &index in to_remove.iter().rev() {
                        queue.remove(index);
                        metrics.queue_size.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }
            }
        });
    }

    /// Start metrics collection background task
    async fn start_metrics_collection(&self) {
        let metrics = self.metrics.clone();
        let change_history = self.change_history.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

            loop {
                interval.tick().await;

                // Clean up old change history
                let now = Utc::now();
                let mut to_remove = Vec::new();

                for history_entry in change_history.iter() {
                    let history = history_entry.value();
                    if now.signed_duration_since(history.last_updated).num_hours() > 24 {
                        to_remove.push(history_entry.key().clone());
                    }
                }

                for key in to_remove {
                    change_history.remove(&key);
                }

                // Log metrics
                info!("Broadcast metrics - Changes: {}, Clients notified: {}, Failed: {}, Queue size: {}",
                      metrics.total_changes_broadcast.load(std::sync::atomic::Ordering::Relaxed),
                      metrics.total_clients_notified.load(std::sync::atomic::Ordering::Relaxed),
                      metrics.failed_deliveries.load(std::sync::atomic::Ordering::Relaxed),
                      metrics.queue_size.load(std::sync::atomic::Ordering::Relaxed));
            }
        });
    }
}

impl Default for ChangeBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}