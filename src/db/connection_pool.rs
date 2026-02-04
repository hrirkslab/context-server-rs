/// Connection pooling module for SQLite database
/// Provides efficient connection management with pooling

use parking_lot::RwLock;
use rusqlite::Connection;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Pooled connection metadata
#[derive(Clone)]
pub struct PooledConnectionMetadata {
    pub last_used: Instant,
    pub lease_count: usize,
}

/// Simple connection pool for SQLite
/// Manages connections with configurable pool size and timeout
pub struct ConnectionPool {
    db_path: String,
    connections: Arc<RwLock<Vec<(Connection, PooledConnectionMetadata)>>>,
    config: PoolConfig,
}

/// Connection pool configuration
#[derive(Clone, Debug)]
pub struct PoolConfig {
    /// Minimum number of connections to maintain
    pub min_connections: usize,
    /// Maximum number of connections in pool
    pub max_connections: usize,
    /// Connection idle timeout before removal
    pub connection_timeout: Duration,
    /// Time to wait for available connection
    pub acquire_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 2,
            max_connections: 10,
            connection_timeout: Duration::from_secs(300), // 5 minutes
            acquire_timeout: Duration::from_secs(30),
        }
    }
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new(db_path: &str, config: PoolConfig) -> rusqlite::Result<Self> {
        let connections = Arc::new(RwLock::new(Vec::with_capacity(config.max_connections)));

        // Initialize with minimum connections
        let mut conns = connections.write();
        for _ in 0..config.min_connections {
            let conn = Connection::open(db_path)?;
            conns.push((
                conn,
                PooledConnectionMetadata {
                    last_used: Instant::now(),
                    lease_count: 0,
                },
            ));
        }
        drop(conns);

        debug!(
            "Initialized connection pool for {} with {} minimum connections",
            db_path, config.min_connections
        );

        Ok(Self {
            db_path: db_path.to_string(),
            connections,
            config,
        })
    }

    /// Create a connection pool with default configuration
    pub fn with_defaults(db_path: &str) -> rusqlite::Result<Self> {
        Self::new(db_path, PoolConfig::default())
    }

    /// Acquire a connection from the pool
    pub fn get_connection(&self) -> rusqlite::Result<Connection> {
        let start_time = Instant::now();
        
        loop {
            let mut conns = self.connections.write();

            // Try to find and reuse an available connection
            if let Some(pos) = conns.iter().position(|(_, meta)| meta.lease_count == 0) {
                let (conn, _meta) = conns.remove(pos);
                debug!("Reused connection from pool, remaining pool size: {}", conns.len());
                // Don't put it back - caller owns it now
                return Ok(conn);
            }

            // Check if we can create a new connection
            if conns.len() < self.config.max_connections {
                let conn = Connection::open(&self.db_path)?;
                debug!("Created new connection, pool size: {}", conns.len() + 1);
                return Ok(conn);
            }

            drop(conns);

            // Check timeout
            if start_time.elapsed() > self.config.acquire_timeout {
                warn!("Timeout acquiring connection from pool");
                return Err(rusqlite::Error::ExecuteReturnedResults);
            }

            // Backoff and retry
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Release a connection back to the pool
    pub fn release_connection(&self, conn: Connection) {
        let mut conns = self.connections.write();
        
        // Check if connection should be discarded due to idle timeout
        if conns.len() >= self.config.max_connections {
            debug!("Discarding connection as pool is at max capacity");
            return;
        }

        conns.push((
            conn,
            PooledConnectionMetadata {
                last_used: Instant::now(),
                lease_count: 0,
            },
        ));
        debug!("Released connection back to pool, pool size: {}", conns.len());
    }

    /// Clean up idle connections beyond minimum
    pub fn cleanup_idle(&self) -> usize {
        let mut conns = self.connections.write();
        let original_len = conns.len();

        conns.retain(|(_, meta)| {
            if meta.lease_count == 0 && meta.last_used.elapsed() > self.config.connection_timeout {
                false
            } else {
                true
            }
        });

        let removed = original_len - conns.len();
        if removed > 0 {
            debug!("Cleaned up {} idle connections", removed);
        }
        removed
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        let conns = self.connections.read();
        let active_connections = conns.iter().filter(|(_, meta)| meta.lease_count > 0).count();
        let idle_connections = conns.len() - active_connections;

        PoolStats {
            total_connections: conns.len(),
            active_connections,
            idle_connections,
            max_connections: self.config.max_connections,
        }
    }
}

/// Statistics about the connection pool
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub idle_connections: usize,
    pub max_connections: usize,
}

impl PoolStats {
    /// Calculate pool utilization percentage
    pub fn utilization_percent(&self) -> f64 {
        if self.max_connections == 0 {
            0.0
        } else {
            (self.active_connections as f64 / self.max_connections as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_connection_pool_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_path_str = db_path.to_str().unwrap();

        let pool = ConnectionPool::with_defaults(db_path_str);
        assert!(pool.is_ok());
    }

    #[test]
    fn test_connection_pool_stats() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_path_str = db_path.to_str().unwrap();

        let pool = ConnectionPool::with_defaults(db_path_str).unwrap();
        let stats = pool.stats();

        assert!(stats.total_connections >= 2);
        assert_eq!(stats.active_connections, 0);
    }

    #[test]
    fn test_connection_pool_utilization() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_path_str = db_path.to_str().unwrap();

        let pool = ConnectionPool::new(
            db_path_str,
            PoolConfig {
                min_connections: 2,
                max_connections: 5,
                ..Default::default()
            },
        )
        .unwrap();

        let stats = pool.stats();
        assert!(stats.utilization_percent() >= 0.0);
        assert!(stats.utilization_percent() <= 100.0);
    }
}
