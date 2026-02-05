/// CLI Router - Orchestration and command routing
/// Follows Dependency Inversion: depends on CliCommand abstraction, not concrete types
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::sync::Arc;
use crate::cli::commands::CliCommand;
use crate::cli::handlers::{QueryCommand, ListCommand, SearchCommand, GetCommand};
use crate::cli::output::get_formatter;

#[derive(Parser)]
#[command(name = "context-server-rs")]
#[command(about = "Context Server for AI Agents and IDEs", long_about = None)]
#[command(version)]
#[command(after_help = "EXAMPLES:\n  # Query all contexts for a project\n  context-server-rs query -p myproject\n\n  # List business rules for a project\n  context-server-rs list business_rule -p myproject\n\n  # Search across all contexts\n  context-server-rs search payment -p myproject\n\n  # Get specific context by ID\n  context-server-rs get rule-001 -p myproject\n\n  # Output in different formats\n  context-server-rs query -f yaml -p myproject\n  context-server-rs list security_policy -f text -p myproject")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(global = true, short, long, help = "Database path")]
    pub db: Option<String>,

    #[arg(global = true, short, long, default_value = "json", help = "Output format: json, text, yaml")]
    pub format: String,

    #[arg(global = true, short, long, help = "Project name (required for CLI commands, defaults to 'default' if not specified)")]
    pub project: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Serve as HTTP/MCP server
    #[command(about = "Start MCP server (default mode)")]
    Serve {
        #[arg(short, long, default_value = "9000")]
        port: u16,
    },

    /// Query all contexts for a project
    #[command(about = "Query all contexts (business rules, architectural decisions, performance requirements, security policies, features)")]
    Query {
        #[arg(short, long, help = "Optional task name to filter by")]
        task: Option<String>,
    },

    /// List contexts by type
    #[command(about = "List contexts of a specific type")]
    List {
        #[arg(help = "Entity type: business_rule | architectural_decision | performance_requirement | security_policy | feature")]
        r#type: String,
    },

    /// Full-text search across all contexts
    #[command(about = "Search across all contexts - searches names, descriptions, and titles")]
    Search {
        #[arg(help = "Search query (case-insensitive)")]
        query: String,
    },

    /// Get a specific context by ID
    #[command(about = "Retrieve a specific context by its ID")]
    Get {
        #[arg(help = "Entity ID (e.g., rule-001, ad-002, perf-003)")]
        id: String,
    },
}

pub struct CliRouter {
    db_path: String,
    format: String,
    project: Option<String>,
}

impl CliRouter {
    pub fn new(db_path: String, format: String, project: Option<String>) -> Self {
        Self { db_path, format, project }
    }

    /// Route command to appropriate handler - Dependency Inversion
    /// All handlers implement CliCommand trait abstraction
    pub async fn route(&self, command: Commands) -> Result<()> {
        // Create command object (abstraction dependency)
        let cmd: Arc<dyn CliCommand> = match command {
            Commands::Query { task } => Arc::new(
                QueryCommand::new(self.db_path.clone(), task, self.project.clone())
            ),
            Commands::List { r#type } => Arc::new(
                ListCommand::new(self.db_path.clone(), r#type, self.project.clone())
            ),
            Commands::Search { query } => Arc::new(
                SearchCommand::new(self.db_path.clone(), query, self.project.clone())
            ),
            Commands::Get { id } => Arc::new(
                GetCommand::new(self.db_path.clone(), id)
            ),
            Commands::Serve { port: _ } => {
                // Serve mode handled separately in main
                return Ok(());
            }
        };

        // Execute through abstraction
        let result = cmd.execute()?;

        // Format and output
        let formatter = get_formatter(&self.format);
        let output = formatter.format(result);
        println!("{}", output);

        Ok(())
    }
}
