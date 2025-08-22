#!/usr/bin/env rust-script
//! Demo script showing IDE Integration Plugin in action
//!
//! This demonstrates how the plugin would work in a real IDE environment

use std::collections::HashMap;
use std::path::Path;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 IDE Integration Plugin Demo");
    println!("===============================\n");

    // Simulate IDE workspace
    let workspace = setup_demo_workspace().await?;
    println!(
        "📁 Created demo workspace at: {}\n",
        workspace.path().display()
    );

    // Demo 1: File Analysis
    println!("🔍 Demo 1: Real-time File Analysis");
    println!("-----------------------------------");
    demo_file_analysis(&workspace).await?;

    // Demo 2: Context Suggestions
    println!("\n💡 Demo 2: Context Suggestions");
    println!("------------------------------");
    demo_context_suggestions(&workspace).await?;

    // Demo 3: Event Handling
    println!("\n⚡ Demo 3: IDE Event Handling");
    println!("-----------------------------");
    demo_event_handling(&workspace).await?;

    // Demo 4: Multi-language Support
    println!("\n🌍 Demo 4: Multi-language Support");
    println!("----------------------------------");
    demo_multi_language_support(&workspace).await?;

    println!("\n✅ Demo completed! The IDE Integration Plugin successfully:");
    println!("   • Analyzed code files in real-time");
    println!("   • Generated intelligent context suggestions");
    println!("   • Handled IDE events (file open, save, change)");
    println!("   • Supported multiple programming languages");
    println!("   • Extracted meaningful context from code patterns");

    Ok(())
}

async fn setup_demo_workspace() -> Result<TempDir, Box<dyn std::error::Error>> {
    let workspace = TempDir::new()?;

    // Create a realistic project structure
    tokio::fs::create_dir_all(workspace.path().join("src")).await?;
    tokio::fs::create_dir_all(workspace.path().join("tests")).await?;
    tokio::fs::create_dir_all(workspace.path().join("docs")).await?;

    // Rust service file
    tokio::fs::write(
        workspace.path().join("src/user_service.rs"),
        r#"//! User management service
//! 
//! This module handles user authentication and profile management.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// User authentication service
pub struct UserService {
    users: HashMap<String, User>,
    session_timeout: u64,
}

/// User profile data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl UserService {
    /// Create a new user service
    pub fn new(session_timeout: u64) -> Self {
        Self {
            users: HashMap::new(),
            session_timeout,
        }
    }

    /// Authenticate user with credentials
    pub async fn authenticate(&self, username: &str, password: &str) -> Result<User, AuthError> {
        // TODO: Implement proper password hashing
        if let Some(user) = self.users.get(username) {
            Ok(user.clone())
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }

    /// Create a new user account
    pub async fn create_user(&mut self, username: String, email: String) -> Result<User, AuthError> {
        if self.users.contains_key(&username) {
            return Err(AuthError::UserExists);
        }

        let user = User {
            id: uuid::Uuid::new_v4().to_string(),
            username: username.clone(),
            email,
            created_at: chrono::Utc::now(),
        };

        self.users.insert(username, user.clone());
        Ok(user)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User already exists")]
    UserExists,
    #[error("Database error: {0}")]
    DatabaseError(String),
}
"#,
    ).await?;

    // TypeScript API file
    tokio::fs::write(
        workspace.path().join("src/api.ts"),
        r#"/**
 * REST API endpoints for the application
 * 
 * This module defines all HTTP endpoints and their handlers.
 */

import express from 'express';
import { UserService } from './user-service';

export class ApiServer {
    private app: express.Application;
    private userService: UserService;
    private port: number;

    constructor(port: number = 3000) {
        this.app = express();
        this.userService = new UserService();
        this.port = port;
        this.setupMiddleware();
        this.setupRoutes();
    }

    /**
     * Configure Express middleware
     */
    private setupMiddleware(): void {
        this.app.use(express.json());
        this.app.use(express.urlencoded({ extended: true }));
        
        // CORS middleware
        this.app.use((req, res, next) => {
            res.header('Access-Control-Allow-Origin', '*');
            res.header('Access-Control-Allow-Headers', 'Content-Type, Authorization');
            next();
        });
    }

    /**
     * Setup API routes
     */
    private setupRoutes(): void {
        // Health check endpoint
        this.app.get('/health', (req, res) => {
            res.json({ status: 'healthy', timestamp: new Date().toISOString() });
        });

        // User authentication
        this.app.post('/auth/login', async (req, res) => {
            try {
                const { username, password } = req.body;
                const user = await this.userService.authenticate(username, password);
                res.json({ success: true, user });
            } catch (error) {
                res.status(401).json({ success: false, error: error.message });
            }
        });

        // User registration
        this.app.post('/auth/register', async (req, res) => {
            try {
                const { username, email, password } = req.body;
                const user = await this.userService.createUser(username, email, password);
                res.status(201).json({ success: true, user });
            } catch (error) {
                res.status(400).json({ success: false, error: error.message });
            }
        });
    }

    /**
     * Start the API server
     */
    public async start(): Promise<void> {
        return new Promise((resolve) => {
            this.app.listen(this.port, () => {
                console.log(`API server running on port ${this.port}`);
                resolve();
            });
        });
    }
}

// Error types
export class ApiError extends Error {
    constructor(
        message: string,
        public statusCode: number = 500,
        public code?: string
    ) {
        super(message);
        this.name = 'ApiError';
    }
}
"#,
    )
    .await?;

    // Python data processor
    tokio::fs::write(
        workspace.path().join("src/data_processor.py"),
        r#"""
Data processing utilities for analytics and reporting.

This module provides functions for processing user data, generating reports,
and performing statistical analysis.
"""

import pandas as pd
import numpy as np
from typing import Dict, List, Optional, Union
from datetime import datetime, timedelta
import logging

logger = logging.getLogger(__name__)

class DataProcessor:
    """
    Main data processing class for handling user analytics.
    
    This class provides methods for data cleaning, transformation,
    and statistical analysis of user behavior data.
    """
    
    def __init__(self, config: Optional[Dict] = None):
        """Initialize the data processor with optional configuration."""
        self.config = config or {}
        self.cache = {}
        self.processed_count = 0
        
    def process_user_data(self, raw_data: List[Dict]) -> pd.DataFrame:
        """
        Process raw user data into a clean DataFrame.
        
        Args:
            raw_data: List of user data dictionaries
            
        Returns:
            Cleaned and processed DataFrame
            
        Raises:
            ValueError: If input data is invalid
            ProcessingError: If processing fails
        """
        if not raw_data:
            raise ValueError("Input data cannot be empty")
            
        try:
            df = pd.DataFrame(raw_data)
            
            # Data cleaning
            df = self._clean_data(df)
            
            # Feature engineering
            df = self._add_features(df)
            
            # Validation
            self._validate_processed_data(df)
            
            self.processed_count += len(df)
            logger.info(f"Processed {len(df)} records successfully")
            
            return df
            
        except Exception as e:
            logger.error(f"Data processing failed: {str(e)}")
            raise ProcessingError(f"Failed to process data: {str(e)}")
    
    def _clean_data(self, df: pd.DataFrame) -> pd.DataFrame:
        """Clean and normalize the input data."""
        # Remove duplicates
        df = df.drop_duplicates()
        
        # Handle missing values
        df = df.fillna(method='forward')
        
        # Normalize text fields
        text_columns = df.select_dtypes(include=['object']).columns
        for col in text_columns:
            df[col] = df[col].str.strip().str.lower()
            
        return df
    
    def _add_features(self, df: pd.DataFrame) -> pd.DataFrame:
        """Add computed features to the dataset."""
        # Add timestamp features if datetime column exists
        if 'created_at' in df.columns:
            df['created_at'] = pd.to_datetime(df['created_at'])
            df['day_of_week'] = df['created_at'].dt.dayofweek
            df['hour_of_day'] = df['created_at'].dt.hour
            
        # Add user activity score
        if 'login_count' in df.columns and 'session_duration' in df.columns:
            df['activity_score'] = df['login_count'] * df['session_duration']
            
        return df
    
    def _validate_processed_data(self, df: pd.DataFrame) -> None:
        """Validate the processed data meets quality requirements."""
        if df.empty:
            raise ProcessingError("Processed data is empty")
            
        # Check for required columns
        required_columns = self.config.get('required_columns', [])
        missing_columns = set(required_columns) - set(df.columns)
        if missing_columns:
            raise ProcessingError(f"Missing required columns: {missing_columns}")
    
    def generate_report(self, df: pd.DataFrame) -> Dict:
        """Generate a summary report from processed data."""
        report = {
            'total_records': len(df),
            'processing_timestamp': datetime.now().isoformat(),
            'data_quality_score': self._calculate_quality_score(df),
            'summary_stats': df.describe().to_dict(),
        }
        
        return report
    
    def _calculate_quality_score(self, df: pd.DataFrame) -> float:
        """Calculate a data quality score (0-1)."""
        # Simple quality score based on completeness
        total_cells = df.size
        non_null_cells = df.count().sum()
        return non_null_cells / total_cells if total_cells > 0 else 0.0

class ProcessingError(Exception):
    """Custom exception for data processing errors."""
    pass

# Utility functions
def validate_data_format(data: Union[List, Dict]) -> bool:
    """Validate that input data has the correct format."""
    if isinstance(data, list):
        return all(isinstance(item, dict) for item in data)
    return isinstance(data, dict)

def calculate_user_metrics(df: pd.DataFrame) -> Dict:
    """Calculate key user metrics from processed data."""
    metrics = {}
    
    if 'user_id' in df.columns:
        metrics['unique_users'] = df['user_id'].nunique()
        
    if 'created_at' in df.columns:
        df['created_at'] = pd.to_datetime(df['created_at'])
        metrics['date_range'] = {
            'start': df['created_at'].min().isoformat(),
            'end': df['created_at'].max().isoformat()
        }
        
    return metrics
"#,
    )
    .await?;

    Ok(workspace)
}

async fn demo_file_analysis(workspace: &TempDir) -> Result<(), Box<dyn std::error::Error>> {
    let files = [
        ("src/user_service.rs", "rust"),
        ("src/api.ts", "typescript"),
        ("src/data_processor.py", "python"),
    ];

    for (file_path, expected_lang) in files {
        let full_path = workspace.path().join(file_path);
        let content = tokio::fs::read_to_string(&full_path).await?;

        println!("📄 Analyzing: {}", file_path);

        // Simulate language detection
        let detected_lang = detect_language(&full_path)?;
        println!("   Language: {} ✓", detected_lang);

        // Simulate pattern analysis
        let patterns = analyze_patterns(&content, &detected_lang);
        println!("   Patterns found:");
        for (pattern_type, count) in patterns {
            println!("     • {}: {} matches", pattern_type, count);
        }

        // Simulate context extraction
        let contexts = extract_contexts(&content, &detected_lang);
        println!("   Context extracted:");
        for context in contexts.iter().take(3) {
            println!("     • {}: {}", context.context_type, context.title);
        }

        println!();
    }

    Ok(())
}

async fn demo_context_suggestions(workspace: &TempDir) -> Result<(), Box<dyn std::error::Error>> {
    let rust_file = workspace.path().join("src/user_service.rs");
    let content = tokio::fs::read_to_string(&rust_file).await?;

    println!("🔍 Analyzing user_service.rs for suggestions...");

    // Simulate suggestion generation
    let suggestions = generate_suggestions(&content, "rust");

    for suggestion in suggestions {
        println!("💡 {}", suggestion.title);
        println!("   Type: {:?}", suggestion.suggestion_type);
        println!("   Priority: {:?}", suggestion.priority);
        println!("   Description: {}", suggestion.description);
        println!("   Actions:");
        for action in &suggestion.actions {
            println!("     • {} - {}", action.title, action.description);
        }
        println!();
    }

    Ok(())
}

async fn demo_event_handling(_workspace: &TempDir) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎬 Simulating IDE events...");

    // Simulate various IDE events
    let events = vec![
        ("file_opened", "src/user_service.rs"),
        ("file_changed", "src/user_service.rs"),
        ("file_saved", "src/user_service.rs"),
        ("cursor_moved", "src/user_service.rs:45:12"),
        ("debug_started", "src/user_service.rs"),
    ];

    for (event_type, context) in events {
        println!("⚡ Event: {} ({})", event_type, context);

        // Simulate event processing
        let response = process_ide_event(event_type, context).await;
        println!("   Response: {}", response);
        println!();
    }

    Ok(())
}

async fn demo_multi_language_support(
    workspace: &TempDir,
) -> Result<(), Box<dyn std::error::Error>> {
    let files = ["src/user_service.rs", "src/api.ts", "src/data_processor.py"];

    println!("🌍 Testing multi-language analysis...");

    for file_path in files {
        let full_path = workspace.path().join(file_path);
        let content = tokio::fs::read_to_string(&full_path).await?;
        let language = detect_language(&full_path)?;

        println!("📁 {}", file_path);
        println!("   Language: {}", language);

        // Language-specific analysis
        match language.as_str() {
            "rust" => {
                let structs = count_pattern(&content, r"(?m)^\s*(?:pub\s+)?struct\s+\w+");
                let functions = count_pattern(&content, r"(?m)^\s*(?:pub\s+)?fn\s+\w+");
                let impls = count_pattern(&content, r"(?m)^\s*impl\s+");
                println!(
                    "   • Structs: {}, Functions: {}, Impls: {}",
                    structs, functions, impls
                );
            }
            "typescript" => {
                let classes = count_pattern(&content, r"(?m)^\s*(?:export\s+)?class\s+\w+");
                let interfaces = count_pattern(&content, r"(?m)^\s*(?:export\s+)?interface\s+\w+");
                let functions = count_pattern(
                    &content,
                    r"(?m)^\s*(?:async\s+)?(?:public\s+|private\s+)?\w+\s*\(",
                );
                println!(
                    "   • Classes: {}, Interfaces: {}, Functions: {}",
                    classes, interfaces, functions
                );
            }
            "python" => {
                let classes = count_pattern(&content, r"(?m)^\s*class\s+\w+");
                let functions = count_pattern(&content, r"(?m)^\s*def\s+\w+");
                let docstrings = count_pattern(&content, r#"(?s)""".*?""""#);
                println!(
                    "   • Classes: {}, Functions: {}, Docstrings: {}",
                    classes, functions, docstrings
                );
            }
            _ => println!("   • Language not specifically supported"),
        }
        println!();
    }

    Ok(())
}

// Helper functions (simplified versions of the actual plugin logic)

fn detect_language(file_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or("No file extension")?;

    let language = match extension {
        "rs" => "rust",
        "ts" => "typescript",
        "js" => "javascript",
        "py" => "python",
        "java" => "java",
        "cpp" | "cc" | "cxx" => "cpp",
        "cs" => "csharp",
        _ => return Err(format!("Unsupported extension: {}", extension).into()),
    };

    Ok(language.to_string())
}

fn analyze_patterns(content: &str, language: &str) -> Vec<(String, usize)> {
    let mut patterns = Vec::new();

    match language {
        "rust" => {
            patterns.push(("Functions".to_string(), count_pattern(content, r"fn\s+\w+")));
            patterns.push((
                "Structs".to_string(),
                count_pattern(content, r"struct\s+\w+"),
            ));
            patterns.push(("Enums".to_string(), count_pattern(content, r"enum\s+\w+")));
            patterns.push(("Doc Comments".to_string(), count_pattern(content, r"///.*")));
        }
        "typescript" => {
            patterns.push((
                "Functions".to_string(),
                count_pattern(content, r"function\s+\w+|=>\s*{"),
            ));
            patterns.push((
                "Classes".to_string(),
                count_pattern(content, r"class\s+\w+"),
            ));
            patterns.push((
                "Interfaces".to_string(),
                count_pattern(content, r"interface\s+\w+"),
            ));
            patterns.push((
                "JSDoc".to_string(),
                count_pattern(content, r"/\*\*[\s\S]*?\*/"),
            ));
        }
        "python" => {
            patterns.push((
                "Functions".to_string(),
                count_pattern(content, r"def\s+\w+"),
            ));
            patterns.push((
                "Classes".to_string(),
                count_pattern(content, r"class\s+\w+"),
            ));
            patterns.push((
                "Docstrings".to_string(),
                count_pattern(content, r#"(?s)""".*?""""#),
            ));
        }
        _ => {}
    }

    patterns
}

fn count_pattern(content: &str, pattern: &str) -> usize {
    regex::Regex::new(pattern)
        .map(|re| re.find_iter(content).count())
        .unwrap_or(0)
}

#[derive(Debug)]
struct ExtractedContext {
    context_type: String,
    title: String,
    content: String,
}

fn extract_contexts(content: &str, language: &str) -> Vec<ExtractedContext> {
    let mut contexts = Vec::new();

    match language {
        "rust" => {
            // Extract function documentation
            let func_regex =
                regex::Regex::new(r"(?m)^\s*///\s*(.*?)\n\s*(?:pub\s+)?fn\s+(\w+)").unwrap();
            for cap in func_regex.captures_iter(content) {
                contexts.push(ExtractedContext {
                    context_type: "business_rule".to_string(),
                    title: format!("Function: {}", &cap[2]),
                    content: cap[1].to_string(),
                });
            }

            // Extract struct definitions
            let struct_regex = regex::Regex::new(r"(?m)^\s*(?:pub\s+)?struct\s+(\w+)").unwrap();
            for cap in struct_regex.captures_iter(content) {
                contexts.push(ExtractedContext {
                    context_type: "architectural_decision".to_string(),
                    title: format!("Struct: {}", &cap[1]),
                    content: format!("Data structure definition for {}", &cap[1]),
                });
            }
        }
        "typescript" => {
            // Extract JSDoc comments
            let jsdoc_regex = regex::Regex::new(
                r"/\*\*\s*(.*?)\s*\*/\s*(?:export\s+)?(?:class|function|interface)\s+(\w+)",
            )
            .unwrap();
            for cap in jsdoc_regex.captures_iter(content) {
                contexts.push(ExtractedContext {
                    context_type: "business_rule".to_string(),
                    title: format!("Component: {}", &cap[2]),
                    content: cap[1].replace("*", "").trim().to_string(),
                });
            }
        }
        "python" => {
            // Extract class docstrings
            let class_regex =
                regex::Regex::new(r#"(?s)class\s+(\w+).*?"""\s*(.*?)\s*""""#).unwrap();
            for cap in class_regex.captures_iter(content) {
                contexts.push(ExtractedContext {
                    context_type: "architectural_decision".to_string(),
                    title: format!("Class: {}", &cap[1]),
                    content: cap[2].trim().to_string(),
                });
            }
        }
        _ => {}
    }

    contexts
}

#[derive(Debug)]
struct ContextSuggestion {
    title: String,
    description: String,
    suggestion_type: SuggestionType,
    priority: SuggestionPriority,
    actions: Vec<SuggestionAction>,
}

#[derive(Debug)]
enum SuggestionType {
    MissingDocumentation,
    ArchitecturalDecision,
    BusinessRule,
    PerformanceRequirement,
}

#[derive(Debug)]
enum SuggestionPriority {
    Low,
    Medium,
    High,
}

#[derive(Debug)]
struct SuggestionAction {
    title: String,
    description: String,
}

fn generate_suggestions(content: &str, language: &str) -> Vec<ContextSuggestion> {
    let mut suggestions = Vec::new();

    match language {
        "rust" => {
            // Check for undocumented public functions
            let undoc_funcs = regex::Regex::new(r"(?m)^\s*pub fn\s+(\w+)")
                .unwrap()
                .captures_iter(content)
                .filter(|cap| {
                    let func_name = &cap[1];
                    !content.contains(&format!("/// {}", func_name))
                        && !content.contains(&format!("//! {}", func_name))
                })
                .count();

            if undoc_funcs > 0 {
                suggestions.push(ContextSuggestion {
                    title: "Missing Documentation".to_string(),
                    description: format!(
                        "Found {} public functions without documentation",
                        undoc_funcs
                    ),
                    suggestion_type: SuggestionType::MissingDocumentation,
                    priority: SuggestionPriority::Medium,
                    actions: vec![
                        SuggestionAction {
                            title: "Add Documentation".to_string(),
                            description: "Add /// comments to document function purpose"
                                .to_string(),
                        },
                        SuggestionAction {
                            title: "Generate Template".to_string(),
                            description: "Auto-generate documentation templates".to_string(),
                        },
                    ],
                });
            }

            // Check for complex error handling
            if content.contains("Result<") && content.contains("Error") {
                suggestions.push(ContextSuggestion {
                    title: "Error Handling Pattern".to_string(),
                    description: "Document error handling strategy and recovery mechanisms"
                        .to_string(),
                    suggestion_type: SuggestionType::ArchitecturalDecision,
                    priority: SuggestionPriority::High,
                    actions: vec![SuggestionAction {
                        title: "Document Error Strategy".to_string(),
                        description: "Create context entry for error handling approach".to_string(),
                    }],
                });
            }
        }
        _ => {}
    }

    suggestions
}

async fn process_ide_event(event_type: &str, context: &str) -> String {
    match event_type {
        "file_opened" => format!("✅ Analyzed file and extracted context"),
        "file_changed" => format!("🔄 Updated context based on changes"),
        "file_saved" => format!("💾 Triggered context validation and suggestions"),
        "cursor_moved" => format!("👆 Provided contextual suggestions for current location"),
        "debug_started" => format!("🐛 Enhanced debugging with context information"),
        _ => format!("❓ Unknown event processed"),
    }
}
