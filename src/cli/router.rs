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
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(global = true, short, long, help = "Database path")]
    pub db: Option<String>,

    #[arg(global = true, short, long, default_value = "json", help = "Output format: json, text, yaml")]
    pub format: String,

    #[arg(global = true, short, long, help = "Project name")]
    pub project: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Serve as HTTP/MCP server
    #[command(about = "Start MCP server")]
    Serve {
        #[arg(short, long, default_value = "9000")]
        port: u16,
    },

    /// Query contexts by task and project
    #[command(about = "Query contexts by task")]
    Query {
        #[arg(short, long, help = "Task name")]
        task: Option<String>,
    },

    /// List all contexts of a type
    #[command(about = "List contexts by type")]
    List {
        #[arg(help = "Entity type: business_rule, architectural_decision, performance_requirement, security_policy, feature")]
        r#type: String,
    },

    /// Search across all contexts
    #[command(about = "Search contexts")]
    Search {
        #[arg(help = "Search query")]
        query: String,
    },

    /// Get a specific context by ID
    #[command(about = "Get context by ID")]
    Get {
        #[arg(help = "Entity ID")]
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
