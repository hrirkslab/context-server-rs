mod db;
mod models;
mod context_server;
mod context_server_solid;
mod repositories;
mod services;
mod infrastructure;
mod container;

use context_server::ContextMcpServer;
use db::init::init_db;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::{self, EnvFilter};
use anyhow::Result;
use std::path::PathBuf;
use std::fs;

/// Get the config directory path for the context server
fn get_config_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    
    let config_dir = home_dir.join("config").join("context-server-rs");
    
    // Create the config directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
        tracing::info!("Created config directory: {}", config_dir.display());
    }
    
    Ok(config_dir)
}

/// MCP Context Server for AI Code Generation
/// 
/// This server provides curated project context that AI agents cannot automatically discover.
/// It stores business rules, architectural decisions, conventions, and other high-value context
/// to help AI agents generate better production-quality code.
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
    match init_db(db_path.to_str().unwrap()) {
        Ok(_) => tracing::info!("Database initialized at {}", db_path.display()),
        Err(e) => {
            tracing::error!("Failed to initialize DB: {}", e);
            return Err(e.into());
        }
    }

    // Create and start the MCP server
    let service = ContextMcpServer::new(db_path.to_str().unwrap())?
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("Failed to serve MCP server: {:?}", e);
        })?;

    tracing::info!("MCP Context Server started successfully");
    
    // Wait for the service to complete
    service.waiting().await?;
    
    Ok(())
}
