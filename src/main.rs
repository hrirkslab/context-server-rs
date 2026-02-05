mod api;
mod container;
mod context_server;
mod context_server_solid;
mod db;
mod enhanced_context_server;
mod infrastructure;
mod models;
mod repositories;
mod services;
mod cli;

use anyhow::Result;
use clap::Parser;
use cli::router::{Cli, Commands, CliRouter};
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

    let config_dir = home_dir.join(".config").join("context-server-rs");

    // Create the config directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
        tracing::debug!("Created config directory: {}", config_dir.display());
    }

    Ok(config_dir)
}

/// Get the database path, either from CLI argument or default location
fn get_db_path(cli_db: Option<String>) -> Result<String> {
    if let Some(db_path) = cli_db {
        Ok(db_path)
    } else {
        let config_dir = get_config_dir()?;
        let db_path = config_dir.join("context.db");
        db_path
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Invalid database path"))
    }
}

/// Enhanced MCP Context Server for AI Code Generation with SOLID Architecture
///
/// Dual-mode binary supporting:
/// - MCP Server Mode: When invoked without arguments or with 'serve' (default)
/// - CLI Mode: OpenClaw integration via query/list/search/get commands
///
/// This server provides curated project context that AI agents cannot automatically discover.
/// It stores business rules, architectural decisions, conventions, security policies, and other
/// high-value context to help AI agents generate better production-quality code.
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging - adjust level based on mode (query is CLI, serve is server)
    let is_cli_mode = std::env::args().any(|arg| 
        arg == "query" || arg == "list" || arg == "search" || arg == "get"
    );
    
    let env_filter = if is_cli_mode {
        // Quiet logging for CLI mode
        EnvFilter::from_default_env().add_directive(tracing::Level::WARN.into())
    } else {
        // Verbose logging for server mode
        EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into())
    };

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    // Parse CLI arguments
    let cli = Cli::parse();
    
    // Get database path
    let db_path = get_db_path(cli.db)?;

    tracing::debug!("Using database: {}", db_path);

    // Initialize SQLite database
    match init_db(&db_path) {
        Ok(_) => {
            tracing::debug!("Database initialized at {}", db_path);
        }
        Err(e) => {
            tracing::error!("Failed to initialize DB: {}", e);
            return Err(e.into());
        }
    }

    // Route based on command
    match &cli.command {
        Commands::Serve { port: _ } => {
            // Run MCP server mode
            tracing::info!("Starting MCP Context Server");
            
            let service = EnhancedContextMcpServer::new(&db_path)?
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
        _ => {
            // Run CLI mode: Query, List, Search, Get
            let router = CliRouter::new(db_path, cli.format, cli.project);
            router.route(cli.command).await?;
            
            Ok(())
        }
    }
}
