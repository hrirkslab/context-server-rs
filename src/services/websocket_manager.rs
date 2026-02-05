use crate::services::websocket_types::*;
use anyhow::{anyhow, Result};
use chrono::Utc;
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{interval, Duration};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// WebSocket connection manager for real-time synchronization
pub struct WebSocketManager {
    /// Active client connections
    pub connections: Arc<DashMap<ClientId, ClientConnection>>,
    /// Broadcast channel for sending changes to all clients
    change_broadcaster: broadcast::Sender<ContextChange>,
    /// Message queue for reliable delivery
    pub message_queue: Arc<DashMap<ClientId, Vec<QueuedMessage>>>,
    /// Connection health monitoring
    pub health_monitor: Arc<DashMap<ClientId, ConnectionHealth>>,
}

/// Individual client connection
#[derive(Clone)]
pub struct ClientConnection {
    pub client_id: ClientId,
    pub project_id: String,
    pub client_info: ClientInfo,
    pub subscriptions: Vec<SyncFilters>,
    pub message_sender: mpsc::UnboundedSender<WebSocketMessage>,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

/// Queued message for reliable delivery
#[derive(Debug, Clone)]
pub struct QueuedMessage {
    message_id: MessageId,
    message: WebSocketMessage,
    queued_at: chrono::DateTime<chrono::Utc>,
    retry_count: u32,
}

/// Connection health information
#[derive(Debug, Clone)]
pub struct ConnectionHealth {
    last_ping: chrono::DateTime<chrono::Utc>,
    last_pong: chrono::DateTime<chrono::Utc>,
    missed_pings: u32,
    is_healthy: bool,
}

impl WebSocketManager {
    /// Create a new WebSocket manager
    pub fn new() -> Self {
        let (change_broadcaster, _) = broadcast::channel(1000);
        
        Self {
            connections: Arc::new(DashMap::new()),
            change_broadcaster,
            message_queue: Arc::new(DashMap::new()),
            health_monitor: Arc::new(DashMap::new()),
        }
    }

    /// Start the WebSocket manager with health monitoring
    pub async fn start(&self) -> Result<()> {
        info!("Starting WebSocket manager");
        
        // Start health monitoring task
        self.start_health_monitoring().await;
        
        // Start message queue processing
        self.start_queue_processing().await;
        
        Ok(())
    }

    /// Handle a new WebSocket connection
    pub async fn handle_connection(
        &self,
        stream: tokio::net::TcpStream,
    ) -> Result<()> {
        let ws_stream = accept_async(stream).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        let client_id = Uuid::new_v4();
        info!("New WebSocket connection: {}", client_id);

        // Create message channel for this client
        let (message_sender, mut message_receiver) = mpsc::unbounded_channel();

        // Spawn task to handle outgoing messages
        let client_id_clone = client_id;
        tokio::spawn(async move {
            while let Some(message) = message_receiver.recv().await {
                let json_message = match serde_json::to_string(&message) {
                    Ok(json) => json,
                    Err(e) => {
                        error!("Failed to serialize message: {}", e);
                        continue;
                    }
                };

                if let Err(e) = ws_sender.send(Message::Text(json_message)).await {
                    error!("Failed to send message to client {}: {}", client_id_clone, e);
                    break;
                }
            }
        });

        // Handle incoming messages
        let connections = self.connections.clone();
        let change_broadcaster = self.change_broadcaster.clone();
        let message_queue = self.message_queue.clone();
        let health_monitor = self.health_monitor.clone();

        tokio::spawn(async move {
            let mut authenticated = false;
            let mut client_connection: Option<ClientConnection> = None;

            while let Some(msg) = ws_receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<WebSocketMessage>(&text) {
                            Ok(ws_message) => {
                                match Self::handle_message(
                                    client_id,
                                    ws_message,
                                    &mut authenticated,
                                    &mut client_connection,
                                    &connections,
                                    &message_sender,
                                    &change_broadcaster,
                                    &message_queue,
                                    &health_monitor,
                                ).await {
                                    Ok(_) => {},
                                    Err(e) => {
                                        error!("Error handling message from client {}: {}", client_id, e);
                                        let error_msg = WebSocketMessage::Error {
                                            code: "MESSAGE_ERROR".to_string(),
                                            message: e.to_string(),
                                            details: None,
                                        };
                                        let _ = message_sender.send(error_msg);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to parse message from client {}: {}", client_id, e);
                                let error_msg = WebSocketMessage::Error {
                                    code: "PARSE_ERROR".to_string(),
                                    message: "Invalid message format".to_string(),
                                    details: Some(serde_json::json!({"error": e.to_string()})),
                                };
                                let _ = message_sender.send(error_msg);
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        info!("Client {} disconnected", client_id);
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error for client {}: {}", client_id, e);
                        break;
                    }
                    _ => {}
                }
            }

            // Clean up connection
            connections.remove(&client_id);
            message_queue.remove(&client_id);
            health_monitor.remove(&client_id);
            info!("Cleaned up connection for client {}", client_id);
        });

        Ok(())
    }

    /// Handle individual WebSocket messages
    async fn handle_message(
        client_id: ClientId,
        message: WebSocketMessage,
        authenticated: &mut bool,
        client_connection: &mut Option<ClientConnection>,
        connections: &Arc<DashMap<ClientId, ClientConnection>>,
        message_sender: &mpsc::UnboundedSender<WebSocketMessage>,
        _change_broadcaster: &broadcast::Sender<ContextChange>,
        message_queue: &Arc<DashMap<ClientId, Vec<QueuedMessage>>>,
        health_monitor: &Arc<DashMap<ClientId, ConnectionHealth>>,
    ) -> Result<()> {
        match message {
            WebSocketMessage::Auth { token: _, project_id, client_info } => {
                // Simple authentication - in production, validate token
                *authenticated = true;
                
                let connection = ClientConnection {
                    client_id,
                    project_id: project_id.clone(),
                    client_info: client_info.clone(),
                    subscriptions: Vec::new(),
                    message_sender: message_sender.clone(),
                    connected_at: Utc::now(),
                    last_activity: Utc::now(),
                };

                connections.insert(client_id, connection);
                *client_connection = connections.get(&client_id).map(|entry| entry.value().clone());

                // Initialize health monitoring
                health_monitor.insert(client_id, ConnectionHealth {
                    last_ping: Utc::now(),
                    last_pong: Utc::now(),
                    missed_pings: 0,
                    is_healthy: true,
                });

                // Initialize message queue
                message_queue.insert(client_id, Vec::new());

                let response = WebSocketMessage::AuthResponse {
                    success: true,
                    client_id,
                    message: "Authentication successful".to_string(),
                };
                message_sender.send(response)?;

                info!("Client {} authenticated for project {}", client_id, project_id);
            }

            WebSocketMessage::Subscribe { filters } => {
                if !*authenticated {
                    return Err(anyhow!("Client not authenticated"));
                }

                if let Some(mut connection) = connections.get_mut(&client_id) {
                    connection.subscriptions.push(filters.clone());
                    connection.last_activity = Utc::now();
                    debug!("Client {} subscribed to filters: {:?}", client_id, filters);
                }
            }

            WebSocketMessage::Unsubscribe { filters } => {
                if !*authenticated {
                    return Err(anyhow!("Client not authenticated"));
                }

                if let Some(mut connection) = connections.get_mut(&client_id) {
                    connection.subscriptions.retain(|sub| {
                        // Simple comparison - in production, implement proper filter matching
                        !std::ptr::eq(sub, &filters)
                    });
                    connection.last_activity = Utc::now();
                    debug!("Client {} unsubscribed from filters", client_id);
                }
            }

            WebSocketMessage::Ack { message_id } => {
                // Remove acknowledged message from queue
                if let Some(mut queue) = message_queue.get_mut(&client_id) {
                    queue.retain(|msg| msg.message_id != message_id);
                }
                debug!("Client {} acknowledged message {}", client_id, message_id);
            }

            WebSocketMessage::Ping { timestamp: _ } => {
                if let Some(mut health) = health_monitor.get_mut(&client_id) {
                    health.last_ping = Utc::now();
                }

                let pong = WebSocketMessage::Pong {
                    timestamp: Utc::now(),
                };
                message_sender.send(pong)?;
            }

            WebSocketMessage::Pong { timestamp: _ } => {
                if let Some(mut health) = health_monitor.get_mut(&client_id) {
                    health.last_pong = Utc::now();
                    health.missed_pings = 0;
                    health.is_healthy = true;
                }
            }

            _ => {
                warn!("Unhandled message type from client {}", client_id);
            }
        }

        Ok(())
    }

    /// Broadcast a context change to all subscribed clients
    pub async fn broadcast_change(&self, change: ContextChange) -> Result<()> {
        debug!("Broadcasting change: {:?}", change.change_id);

        for connection in self.connections.iter() {
            let client_id = *connection.key();
            let client_connection = connection.value();

            // Check if client is subscribed to this change
            let should_send = client_connection.subscriptions.iter()
                .any(|filter| filter.matches(&change));

            if should_send {
                let message_id = Uuid::new_v4();
                let message = WebSocketMessage::ContextChange {
                    message_id,
                    change: change.clone(),
                    timestamp: Utc::now(),
                };

                // Try to send immediately
                if client_connection.message_sender.send(message.clone()).is_err() {
                    // If immediate send fails, queue the message
                    self.queue_message(client_id, message_id, message).await;
                }
            }
        }

        // Also send to broadcast channel for other components
        let _ = self.change_broadcaster.send(change);

        Ok(())
    }

    /// Queue a message for reliable delivery
    async fn queue_message(&self, client_id: ClientId, message_id: MessageId, message: WebSocketMessage) {
        if let Some(mut queue) = self.message_queue.get_mut(&client_id) {
            queue.push(QueuedMessage {
                message_id,
                message,
                queued_at: Utc::now(),
                retry_count: 0,
            });
        }
    }

    /// Get connection status for a client
    pub async fn get_connection_status(&self, client_id: ClientId) -> Option<ConnectionStatus> {
        let connection = self.connections.get(&client_id)?;
        let health = self.health_monitor.get(&client_id)?;
        let queue_size = self.message_queue.get(&client_id)
            .map(|queue| queue.len())
            .unwrap_or(0);

        Some(ConnectionStatus {
            client_id,
            connected_at: connection.connected_at,
            last_activity: connection.last_activity,
            project_id: connection.project_id.clone(),
            subscriptions: connection.subscriptions.clone(),
            message_queue_size: queue_size,
            is_healthy: health.is_healthy,
        })
    }

    /// Get sync status for a project
    pub async fn get_sync_status(&self, project_id: &str) -> SyncStatus {
        let connected_clients = self.connections.iter()
            .filter(|entry| entry.value().project_id == project_id)
            .count() as u32;

        let pending_changes = self.message_queue.iter()
            .map(|entry| entry.value().len())
            .sum::<usize>() as u32;

        // Determine sync health based on connected clients and pending changes
        let sync_health = if connected_clients == 0 {
            SyncHealth::Unhealthy
        } else if pending_changes > 100 {
            SyncHealth::Degraded
        } else {
            SyncHealth::Healthy
        };

        SyncStatus {
            project_id: project_id.to_string(),
            connected_clients,
            pending_changes,
            last_sync: Some(Utc::now()), // In production, track actual last sync
            sync_health,
        }
    }

    /// Start health monitoring background task
    async fn start_health_monitoring(&self) {
        let connections = self.connections.clone();
        let health_monitor = self.health_monitor.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30)); // Check every 30 seconds

            loop {
                interval.tick().await;

                let now = Utc::now();
                let mut unhealthy_clients = Vec::new();

                // Check health of all connections
                for mut health_entry in health_monitor.iter_mut() {
                    let client_id = *health_entry.key();
                    let health = health_entry.value_mut();

                    // Check if client hasn't responded to ping in a while
                    if now.signed_duration_since(health.last_pong).num_seconds() > 60 {
                        health.missed_pings += 1;
                        health.is_healthy = health.missed_pings < 3;

                        if !health.is_healthy {
                            unhealthy_clients.push(client_id);
                        }
                    }

                    // Send ping to check connectivity
                    if let Some(connection) = connections.get(&client_id) {
                        let ping = WebSocketMessage::Ping {
                            timestamp: now,
                        };
                        let _ = connection.message_sender.send(ping);
                    }
                }

                // Clean up unhealthy connections
                for client_id in unhealthy_clients {
                    warn!("Removing unhealthy client: {}", client_id);
                    connections.remove(&client_id);
                    health_monitor.remove(&client_id);
                }
            }
        });
    }

    /// Start message queue processing background task
    async fn start_queue_processing(&self) {
        let connections = self.connections.clone();
        let message_queue = self.message_queue.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5)); // Process every 5 seconds

            loop {
                interval.tick().await;

                for mut queue_entry in message_queue.iter_mut() {
                    let client_id = *queue_entry.key();
                    let queue = queue_entry.value_mut();

                    if let Some(connection) = connections.get(&client_id) {
                        let mut to_remove = Vec::new();

                        for (index, queued_msg) in queue.iter_mut().enumerate() {
                            // Retry sending the message
                            if connection.message_sender.send(queued_msg.message.clone()).is_ok() {
                                to_remove.push(index);
                            } else {
                                queued_msg.retry_count += 1;
                                
                                // Remove messages that have been retried too many times
                                if queued_msg.retry_count > 5 {
                                    to_remove.push(index);
                                    warn!("Dropping message {} for client {} after {} retries", 
                                          queued_msg.message_id, client_id, queued_msg.retry_count);
                                }
                            }
                        }

                        // Remove successfully sent or expired messages
                        for &index in to_remove.iter().rev() {
                            queue.remove(index);
                        }
                    }
                }
            }
        });
    }
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self::new()
    }
}