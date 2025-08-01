use crate::services::websocket_manager::WebSocketManager;
use crate::services::websocket_types::*;
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info};

/// WebSocket server for real-time synchronization
pub struct WebSocketServer {
    manager: Arc<WebSocketManager>,
    bind_address: SocketAddr,
}

impl WebSocketServer {
    /// Create a new WebSocket server
    pub fn new(bind_address: SocketAddr) -> Self {
        Self {
            manager: Arc::new(WebSocketManager::new()),
            bind_address,
        }
    }

    /// Get a reference to the WebSocket manager
    pub fn manager(&self) -> Arc<WebSocketManager> {
        self.manager.clone()
    }

    /// Start the WebSocket server
    pub async fn start(&self) -> Result<()> {
        info!("Starting WebSocket server on {}", self.bind_address);

        // Start the manager
        self.manager.start().await?;

        // Create TCP listener
        let listener = TcpListener::bind(self.bind_address).await?;
        info!("WebSocket server listening on {}", self.bind_address);

        // Accept connections
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New connection from {}", addr);
                    let manager = self.manager.clone();
                    
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(manager, stream).await {
                            error!("Error handling connection from {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Handle a new connection
    async fn handle_connection(
        manager: Arc<WebSocketManager>,
        stream: TcpStream,
    ) -> Result<()> {
        manager.handle_connection(stream).await
    }
}

/// WebSocket service trait for integration with the context server
#[async_trait::async_trait]
pub trait WebSocketService {
    /// Broadcast a context change to all connected clients
    async fn broadcast_change(&self, change: ContextChange) -> Result<()>;
    
    /// Get connection status for a client
    async fn get_connection_status(&self, client_id: ClientId) -> Option<ConnectionStatus>;
    
    /// Get sync status for a project
    async fn get_sync_status(&self, project_id: &str) -> SyncStatus;
    
    /// Get all connected clients for a project
    async fn get_project_clients(&self, project_id: &str) -> Vec<ClientId>;
}

/// Implementation of WebSocketService for WebSocketManager
#[async_trait::async_trait]
impl WebSocketService for WebSocketManager {
    async fn broadcast_change(&self, change: ContextChange) -> Result<()> {
        self.broadcast_change(change).await
    }
    
    async fn get_connection_status(&self, client_id: ClientId) -> Option<ConnectionStatus> {
        self.get_connection_status(client_id).await
    }
    
    async fn get_sync_status(&self, project_id: &str) -> SyncStatus {
        self.get_sync_status(project_id).await
    }
    
    async fn get_project_clients(&self, project_id: &str) -> Vec<ClientId> {
        self.connections.iter()
            .filter_map(|entry| {
                if entry.value().project_id == project_id {
                    Some(*entry.key())
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Configuration for the WebSocket server
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    pub bind_address: SocketAddr,
    pub max_connections: usize,
    pub heartbeat_interval: std::time::Duration,
    pub message_queue_size: usize,
    pub enable_compression: bool,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:8080".parse().unwrap(),
            max_connections: 1000,
            heartbeat_interval: std::time::Duration::from_secs(30),
            message_queue_size: 1000,
            enable_compression: true,
        }
    }
}

/// Helper functions for creating context changes
pub mod change_helpers {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    /// Create a context change for entity creation
    pub fn create_change(
        entity_type: &str,
        entity_id: &str,
        project_id: &str,
        feature_area: Option<&str>,
        full_entity: serde_json::Value,
        client_id: ClientId,
    ) -> ContextChange {
        ContextChange {
            change_id: Uuid::new_v4(),
            change_type: ChangeType::Create,
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            project_id: project_id.to_string(),
            feature_area: feature_area.map(|s| s.to_string()),
            delta: None,
            full_entity: Some(full_entity),
            metadata: ChangeMetadata {
                user_id: None,
                client_id,
                timestamp: Utc::now(),
                version: 1,
                conflict_resolution: None,
            },
        }
    }

    /// Create a context change for entity update
    pub fn update_change(
        entity_type: &str,
        entity_id: &str,
        project_id: &str,
        feature_area: Option<&str>,
        delta: serde_json::Value,
        full_entity: serde_json::Value,
        client_id: ClientId,
        version: u32,
    ) -> ContextChange {
        ContextChange {
            change_id: Uuid::new_v4(),
            change_type: ChangeType::Update,
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            project_id: project_id.to_string(),
            feature_area: feature_area.map(|s| s.to_string()),
            delta: Some(delta),
            full_entity: Some(full_entity),
            metadata: ChangeMetadata {
                user_id: None,
                client_id,
                timestamp: Utc::now(),
                version,
                conflict_resolution: None,
            },
        }
    }

    /// Create a context change for entity deletion
    pub fn delete_change(
        entity_type: &str,
        entity_id: &str,
        project_id: &str,
        feature_area: Option<&str>,
        client_id: ClientId,
    ) -> ContextChange {
        ContextChange {
            change_id: Uuid::new_v4(),
            change_type: ChangeType::Delete,
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            project_id: project_id.to_string(),
            feature_area: feature_area.map(|s| s.to_string()),
            delta: None,
            full_entity: None,
            metadata: ChangeMetadata {
                user_id: None,
                client_id,
                timestamp: Utc::now(),
                version: 1,
                conflict_resolution: None,
            },
        }
    }

    /// Create a context change for bulk operations
    pub fn bulk_change(
        entity_type: &str,
        project_id: &str,
        feature_area: Option<&str>,
        affected_entities: Vec<String>,
        client_id: ClientId,
    ) -> ContextChange {
        ContextChange {
            change_id: Uuid::new_v4(),
            change_type: ChangeType::Bulk,
            entity_type: entity_type.to_string(),
            entity_id: format!("bulk_{}", Uuid::new_v4()),
            project_id: project_id.to_string(),
            feature_area: feature_area.map(|s| s.to_string()),
            delta: Some(serde_json::json!({
                "affected_entities": affected_entities,
                "operation": "bulk"
            })),
            full_entity: None,
            metadata: ChangeMetadata {
                user_id: None,
                client_id,
                timestamp: Utc::now(),
                version: 1,
                conflict_resolution: None,
            },
        }
    }
}