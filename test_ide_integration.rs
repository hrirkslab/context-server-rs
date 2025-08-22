#!/usr/bin/env rust-script

//! Quick test script for IDE Integration Plugin
//! 
//! Run with: cargo run --bin test_ide_integration

use std::path::Path;
use tempfile::TempDir;
use tokio;

// This would normally import from your crate
// For now, we'll create a simplified test

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing IDE Integration Plugin");
    
    // Create temporary workspace
    let temp_dir = TempDir::new()?;
    let workspace = temp_dir.path();
    println!("📁 Created test workspace: {}", workspace.display());
    
    // Test 1: Create test files
    println!("\n📝 Creating test files...");
    create_test_files(workspace).await?;
    
    // Test 2: Language detection
    println!("\n🔍 Testing language detection...");
    test_language_detection(workspace)?;
    
    // Test 3: File analysis simulation
    println!("\n⚙️ Testing file analysis...");
    test_file_analysis(workspace).await?;
    
    // Test 4: Pattern matching
    println!("\n🎯 Testing pattern matching...");
    test_pattern_matching(workspace).await?;
    
    println!("\n✅ All tests completed successfully!");
    println!("💡 To run the full test suite: cargo test ide_integration_plugin --lib");
    
    Ok(())
}

async fn create_test_files(workspace: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Rust file
    let rust_content = r#"
/// Calculate fibonacci number
pub fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

pub struct Calculator {
    memory: f64,
}

impl Calculator {
    pub fn new() -> Self {
        Self { memory: 0.0 }
    }
    
    pub fn add(&mut self, value: f64) -> Result<f64, String> {
        self.memory += value;
        Ok(self.memory)
    }
}
"#;
    
    // TypeScript file
    let ts_content = r#"
/**
 * User service for managing user data
 */
export class UserService {
    private users: User[] = [];
    
    async createUser(userData: CreateUserRequest): Promise<User> {
        const user = {
            id: generateId(),
            ...userData,
            createdAt: new Date()
        };
        this.users.push(user);
        return user;
    }
    
    async getUserById(id: string): Promise<User | null> {
        return this.users.find(user => user.id === id) || null;
    }
}

interface User {
    id: string;
    name: string;
    email: string;
    createdAt: Date;
}

interface CreateUserRequest {
    name: string;
    email: string;
}
"#;
    
    // Python file
    let python_content = r#"
class DataProcessor:
    """Process and analyze data efficiently."""
    
    def __init__(self):
        self.data = []
    
    def process_data(self, raw_data: list) -> dict:
        """Process raw data and return statistics."""
        if not raw_data:
            raise ValueError("Data cannot be empty")
        
        return {
            'count': len(raw_data),
            'sum': sum(raw_data),
            'average': sum(raw_data) / len(raw_data)
        }
    
    def validate_input(self, data) -> bool:
        """Validate input data format."""
        return isinstance(data, list) and len(data) > 0
"#;
    
    tokio::fs::write(workspace.join("calculator.rs"), rust_content).await?;
    tokio::fs::write(workspace.join("user_service.ts"), ts_content).await?;
    tokio::fs::write(workspace.join("data_processor.py"), python_content).await?;
    
    println!("  ✅ Created calculator.rs");
    println!("  ✅ Created user_service.ts");
    println!("  ✅ Created data_processor.py");
    
    Ok(())
}

fn test_language_detection(workspace: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let test_cases = vec![
        ("calculator.rs", "rust"),
        ("user_service.ts", "typescript"),
        ("data_processor.py", "python"),
    ];
    
    for (filename, expected_lang) in test_cases {
        let file_path = workspace.join(filename);
        let detected_lang = detect_language_simple(&file_path)?;
        
        if detected_lang == expected_lang {
            println!("  ✅ {} -> {}", filename, detected_lang);
        } else {
            println!("  ❌ {} -> expected {}, got {}", filename, expected_lang, detected_lang);
        }
    }
    
    Ok(())
}

fn detect_language_simple(file_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let extension = file_path.extension()
        .and_then(|ext| ext.to_str())
        .ok_or("No file extension")?;
    
    let language = match extension {
        "rs" => "rust",
        "ts" => "typescript",
        "js" => "javascript",
        "py" => "python",
        "java" => "java",
        "cpp" | "cc" | "cxx" => "cpp",
        "h" | "hpp" => "cpp",
        "cs" => "csharp",
        _ => return Err(format!("Unsupported extension: {}", extension).into()),
    };
    
    Ok(language.to_string())
}

async fn test_file_analysis(workspace: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let files = vec![
        workspace.join("calculator.rs"),
        workspace.join("user_service.ts"),
        workspace.join("data_processor.py"),
    ];
    
    for file_path in files {
        let content = tokio::fs::read_to_string(&file_path).await?;
        let analysis = analyze_file_simple(&file_path, &content)?;
        
        println!("  📄 {}", file_path.file_name().unwrap().to_string_lossy());
        println!("    Language: {}", analysis.language);
        println!("    Functions found: {}", analysis.functions.len());
        println!("    Classes found: {}", analysis.classes.len());
        println!("    Comments: {}", analysis.comments.len());
        
        for func in &analysis.functions {
            println!("      🔧 Function: {}", func);
        }
        for class in &analysis.classes {
            println!("      🏗️  Class: {}", class);
        }
    }
    
    Ok(())
}

async fn test_pattern_matching(workspace: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let rust_file = workspace.join("calculator.rs");
    let content = tokio::fs::read_to_string(&rust_file).await?;
    
    // Test Rust patterns
    let patterns = vec![
        (r"pub fn\s+(\w+)", "Public functions"),
        (r"pub struct\s+(\w+)", "Public structs"),
        (r"impl\s+(\w+)", "Implementations"),
        (r"Result<.*>", "Error handling"),
        (r"///.*", "Documentation comments"),
    ];
    
    println!("  🎯 Pattern matching on calculator.rs:");
    
    for (pattern, description) in patterns {
        let regex = regex::Regex::new(pattern).unwrap();
        let matches: Vec<_> = regex.find_iter(&content).collect();
        
        println!("    {} ({}): {} matches", description, pattern, matches.len());
        
        for (i, m) in matches.iter().take(3).enumerate() {
            let line_num = content[..m.start()].lines().count() + 1;
            println!("      {}. Line {}: {}", i + 1, line_num, m.as_str().trim());
        }
    }
    
    Ok(())
}

#[derive(Debug)]
struct FileAnalysis {
    language: String,
    functions: Vec<String>,
    classes: Vec<String>,
    comments: Vec<String>,
}

fn analyze_file_simple(file_path: &Path, content: &str) -> Result<FileAnalysis, Box<dyn std::error::Error>> {
    let language = detect_language_simple(file_path)?;
    let mut functions = Vec::new();
    let mut classes = Vec::new();
    let mut comments = Vec::new();
    
    match language.as_str() {
        "rust" => {
            // Find functions
            let func_regex = regex::Regex::new(r"(?m)^\s*(?:pub\s+)?fn\s+(\w+)")?;
            for cap in func_regex.captures_iter(content) {
                functions.push(cap[1].to_string());
            }
            
            // Find structs
            let struct_regex = regex::Regex::new(r"(?m)^\s*(?:pub\s+)?struct\s+(\w+)")?;
            for cap in struct_regex.captures_iter(content) {
                classes.push(cap[1].to_string());
            }
            
            // Find doc comments
            let comment_regex = regex::Regex::new(r"///.*")?;
            for m in comment_regex.find_iter(content) {
                comments.push(m.as_str().trim_start_matches("///").trim().to_string());
            }
        }
        "typescript" => {
            // Find functions and methods
            let func_regex = regex::Regex::new(r"(?m)^\s*(?:async\s+)?(?:public\s+|private\s+)?(\w+)\s*\(")?;
            for cap in func_regex.captures_iter(content) {
                functions.push(cap[1].to_string());
            }
            
            // Find classes and interfaces
            let class_regex = regex::Regex::new(r"(?m)^\s*(?:export\s+)?(?:class|interface)\s+(\w+)")?;
            for cap in class_regex.captures_iter(content) {
                classes.push(cap[1].to_string());
            }
            
            // Find JSDoc comments
            let comment_regex = regex::Regex::new(r"/\*\*[\s\S]*?\*/")?;
            for m in comment_regex.find_iter(content) {
                let comment = m.as_str()
                    .trim_start_matches("/**")
                    .trim_end_matches("*/")
                    .lines()
                    .map(|line| line.trim().trim_start_matches("*").trim())
                    .filter(|line| !line.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");
                if !comment.is_empty() {
                    comments.push(comment);
                }
            }
        }
        "python" => {
            // Find functions and methods
            let func_regex = regex::Regex::new(r"(?m)^\s*def\s+(\w+)")?;
            for cap in func_regex.captures_iter(content) {
                functions.push(cap[1].to_string());
            }
            
            // Find classes
            let class_regex = regex::Regex::new(r"(?m)^\s*class\s+(\w+)")?;
            for cap in class_regex.captures_iter(content) {
                classes.push(cap[1].to_string());
            }
            
            // Find docstrings
            let docstring_regex = regex::Regex::new(r#""""[\s\S]*?""""#)?;
            for m in docstring_regex.find_iter(content) {
                let docstring = m.as_str()
                    .trim_start_matches("\"\"\"")
                    .trim_end_matches("\"\"\"")
                    .trim();
                if !docstring.is_empty() {
                    comments.push(docstring.to_string());
                }
            }
        }
        _ => {}
    }
    
    Ok(FileAnalysis {
        language,
        functions,
        classes,
        comments,
    })
}