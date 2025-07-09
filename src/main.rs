mod db;
mod models;
mod context_server;

use context_server::ContextMcpServer;
use db::init::init_db;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::{self, EnvFilter};
use anyhow::Result;

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

    // Initialize SQLite database
    let db_path = "context.db";
    match init_db(db_path) {
        Ok(_) => tracing::info!("Database initialized at {}", db_path),
        Err(e) => {
            tracing::error!("Failed to initialize DB: {}", e);
            return Err(e.into());
        }
    }

    // Create and start the MCP server
    let service = ContextMcpServer::new(db_path)?
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
