mod container;
mod context_server;
mod context_server_solid;
mod db;
mod enhanced_context_server;
mod infrastructure;
mod models;
mod repositories;
mod services;

use anyhow::Result;
use db::init::init_db;
use enhanced_context_server::EnhancedContextMcpServer;
use rmcp::{transport::stdio, ServiceExt};
use std::fs;
use std::path::PathBuf;
use tracing_subscriber::{self, EnvFilter};

/// Get the config directory path for the context server
fn get_config_dir() -> Result<PathBuf> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    let config_dir = home_dir.join("config").join("context-server-rs");

    // Create the config directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
        tracing::info!("Created config directory: {}", config_dir.display());
    }

    Ok(config_dir)
}

/// Enhanced MCP Context Server for AI Code Generation with SOLID Architecture
///
/// This server provides curated project context that AI agents cannot automatically discover.
/// It stores business rules, architectural decisions, conventions, security policies, and other
/// high-value context to help AI agents generate better production-quality code.
///
/// Features comprehensive CRUD operations for all entities, bulk operations, and follows
/// SOLID principles with dependency injection and service/repository patterns.
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting MCP Context Server");

    // Get config directory and database path
    let config_dir = get_config_dir()?;
    let db_path = config_dir.join("context.db");

    tracing::info!("Using config directory: {}", config_dir.display());

    // Initialize SQLite database
    let db_path_str = db_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid database path"))?;

    match init_db(db_path_str) {
        Ok(_) => tracing::info!("Database initialized at {}", db_path.display()),
        Err(e) => {
            tracing::error!("Failed to initialize DB: {}", e);
            return Err(e.into());
        }
    }

    // Create and start the enhanced MCP server
    let db_path_str = db_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid database path"))?;

    let service = EnhancedContextMcpServer::new(db_path_str)?
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("Failed to serve MCP server: {:?}", e);
        })?;

    tracing::info!("Enhanced MCP Context Server started successfully");

    // Wait for the service to complete
    service.waiting().await?;

    Ok(())
}
