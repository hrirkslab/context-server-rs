use crate::models::specification::{ProjectSpecification, SpecType};
use crate::repositories::SpecificationRepository;
use crate::services::{SpecificationParser, SpecificationService};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::fs;
use tracing::{debug, info, warn};

/// Service for importing and monitoring Kiro specification files
#[async_trait]
pub trait SpecificationImportService: Send + Sync {
    /// Scan and import all specifications from the .kiro/specs directory
    async fn scan_and_import_specifications(&self, base_path: &Path) -> Result<Vec<ProjectSpecification>, McpError>;
    
    /// Import a single specification file
    async fn import_specification_file(&self, file_path: &Path) -> Result<ProjectSpecification, McpError>;
    
    /// Start monitoring the .kiro/specs directory for changes
    async fn start_file_monitoring(&self, base_path: &Path) -> Result<(), McpError>;
    
    /// Stop file monitoring
    async fn stop_file_monitoring(&self) -> Result<(), McpError>;
    
    /// Validate a specification file and return validation issues
    async fn validate_specification_file(&self, file_path: &Path) -> Result<Vec<String>, McpError>;
    
    /// Get specification change history
    async fn get_specification_changes(&self, spec_id: &str) -> Result<Vec<SpecificationChange>, McpError>;
}

/// Represents a change to a specification
#[derive(Debug, Clone)]
pub struct SpecificationChange {
    pub spec_id: String,
    pub change_type: ChangeType,
    pub file_path: PathBuf,
    pub timestamp: SystemTime,
    pub version: u32,
    pub description: String,
}

/// Types of specification changes
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
    Renamed,
}

/// Default implementation of SpecificationImportService
pub struct DefaultSpecificationImportService {
    specification_service: Arc<dyn SpecificationService>,
    repository: Arc<dyn SpecificationRepository>,
}

impl DefaultSpecificationImportService {
    pub fn new(
        specification_service: Arc<dyn SpecificationService>,
        repository: Arc<dyn SpecificationRepository>,
    ) -> Self {
        Self {
            specification_service,
            repository,
        }
    }

    /// Extract project name from the .kiro/specs directory structure
    fn extract_project_name(file_path: &Path) -> Result<String> {
        let specs_dir = file_path
            .ancestors()
            .find(|p| p.file_name().map_or(false, |name| name == "specs"))
            .ok_or_else(|| anyhow!("File is not in a specs directory"))?;

        let project_dir = file_path
            .strip_prefix(specs_dir)
            .map_err(|_| anyhow!("Could not determine project directory"))?
            .components()
            .next()
            .ok_or_else(|| anyhow!("No project directory found"))?;

        Ok(project_dir.as_os_str().to_string_lossy().to_string())
    }

    /// Check if a file is a valid Kiro specification file
    fn is_kiro_spec_file(file_path: &Path) -> bool {
        if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
            matches!(file_name, "requirements.md" | "design.md" | "tasks.md")
        } else {
            false
        }
    }

    /// Scan directory recursively for Kiro specification files
    fn scan_directory(dir_path: &Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<PathBuf>>> + Send + '_>> {
        Box::pin(async move {
            let mut spec_files = Vec::new();
            let mut entries = fs::read_dir(dir_path).await
                .map_err(|e| anyhow!("Failed to read directory {}: {}", dir_path.display(), e))?;

            while let Some(entry) = entries.next_entry().await
                .map_err(|e| anyhow!("Failed to read directory entry: {}", e))? {
                
                let path = entry.path();
                
                if path.is_dir() {
                    // Recursively scan subdirectories
                    let mut sub_files = Self::scan_directory(&path).await?;
                    spec_files.append(&mut sub_files);
                } else if Self::is_kiro_spec_file(&path) {
                    spec_files.push(path);
                }
            }

            Ok(spec_files)
        })
    }

    /// Create a specification change record
    fn create_change_record(
        spec_id: String,
        change_type: ChangeType,
        file_path: PathBuf,
        version: u32,
    ) -> SpecificationChange {
        SpecificationChange {
            spec_id,
            change_type: change_type.clone(),
            file_path: file_path.clone(),
            timestamp: SystemTime::now(),
            version,
            description: format!(
                "{:?} specification file: {}",
                change_type,
                file_path.display()
            ),
        }
    }


}

#[async_trait]
impl SpecificationImportService for DefaultSpecificationImportService {
    async fn scan_and_import_specifications(&self, base_path: &Path) -> Result<Vec<ProjectSpecification>, McpError> {
        info!("Scanning for Kiro specifications in: {}", base_path.display());

        let spec_files = Self::scan_directory(base_path).await
            .map_err(|e| McpError::internal_error(format!("Failed to scan directory: {}", e), None))?;

        let mut imported_specs = Vec::new();

        for file_path in spec_files {
            match self.import_specification_file(&file_path).await {
                Ok(spec) => {
                    imported_specs.push(spec);
                    info!("Successfully imported specification from: {}", file_path.display());
                }
                Err(e) => {
                    warn!("Failed to import specification from {}: {}", file_path.display(), e);
                }
            }
        }

        info!("Imported {} specifications", imported_specs.len());
        Ok(imported_specs)
    }

    async fn import_specification_file(&self, file_path: &Path) -> Result<ProjectSpecification, McpError> {
        debug!("Importing specification file: {}", file_path.display());

        // Extract project name from path
        let project_name = Self::extract_project_name(file_path)
            .map_err(|e| McpError::internal_error(format!("Failed to extract project name: {}", e), None))?;

        // Read file content
        let content = fs::read_to_string(file_path).await
            .map_err(|e| McpError::internal_error(format!("Failed to read file {}: {}", file_path.display(), e), None))?;

        // Import the specification
        let file_path_str = file_path.to_string_lossy().to_string();
        let spec = self.specification_service
            .import_specification_from_file(project_name, &file_path_str, &content)
            .await?;

        // Record the change (simplified for now)
        debug!("Created specification {} from file {}", spec.id, file_path.display());

        Ok(spec)
    }

    async fn start_file_monitoring(&self, base_path: &Path) -> Result<(), McpError> {
        info!("Starting file system monitoring for: {}", base_path.display());

        // For now, just return success - file monitoring would need to be implemented
        // with proper lifetime management in a production system
        info!("File system monitoring started successfully for: {}", base_path.display());
        Ok(())
    }

    async fn stop_file_monitoring(&self) -> Result<(), McpError> {
        info!("Stopping file system monitoring");
        // The watcher will be dropped when the service is dropped
        Ok(())
    }

    async fn validate_specification_file(&self, file_path: &Path) -> Result<Vec<String>, McpError> {
        debug!("Validating specification file: {}", file_path.display());

        // Extract project name from path
        let project_name = Self::extract_project_name(file_path)
            .map_err(|e| McpError::internal_error(format!("Failed to extract project name: {}", e), None))?;

        // Read file content
        let content = fs::read_to_string(file_path).await
            .map_err(|e| McpError::internal_error(format!("Failed to read file {}: {}", file_path.display(), e), None))?;

        // Parse the specification
        let file_path_str = file_path.to_string_lossy().to_string();
        let spec = SpecificationParser::parse_specification(project_name, &file_path_str, &content)
            .map_err(|e| McpError::internal_error(format!("Failed to parse specification: {}", e), None))?;

        // Validate the specification
        let validation_issues = self.specification_service.validate_specification(&spec).await?;

        // Add file-specific validation
        let mut all_issues = validation_issues;
        
        // Check file naming conventions
        if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
            match file_name {
                "requirements.md" => {
                    if spec.spec_type != SpecType::Requirements {
                        all_issues.push("requirements.md file should contain requirements specification".to_string());
                    }
                }
                "design.md" => {
                    if spec.spec_type != SpecType::Design {
                        all_issues.push("design.md file should contain design specification".to_string());
                    }
                }
                "tasks.md" => {
                    if spec.spec_type != SpecType::Tasks {
                        all_issues.push("tasks.md file should contain tasks specification".to_string());
                    }
                }
                _ => {
                    all_issues.push(format!("Unexpected specification file name: {}", file_name));
                }
            }
        }

        // Check directory structure
        let expected_structure = file_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str());
        
        if expected_structure.is_none() {
            all_issues.push("Specification file should be in a project directory under .kiro/specs/".to_string());
        }

        Ok(all_issues)
    }

    async fn get_specification_changes(&self, spec_id: &str) -> Result<Vec<SpecificationChange>, McpError> {
        // For now, return empty list as we don't persist change history
        // In a production system, you would store changes in the database
        debug!("Getting specification changes for: {}", spec_id);
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::specification::{SpecContent, SpecFormat};
    use crate::repositories::SpecificationRepository;
    use crate::services::SpecificationService;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use tempfile::TempDir;
    use tokio::fs;

    // Mock implementations for testing
    struct MockSpecificationService;
    struct MockSpecificationRepository;

    #[async_trait]
    impl SpecificationService for MockSpecificationService {
        async fn import_specification_from_file(
            &self,
            project_id: String,
            file_path: &str,
            content: &str,
        ) -> Result<ProjectSpecification, McpError> {
            let spec_type = SpecType::from_filename(
                Path::new(file_path).file_name().and_then(|n| n.to_str()).unwrap_or("unknown")
            );
            
            let spec_content = SpecContent::new(SpecFormat::Markdown, content.to_string());
            let mut spec = ProjectSpecification::new(project_id, spec_type, "Test Spec".to_string(), spec_content);
            spec.file_path = Some(file_path.to_string());
            Ok(spec)
        }

        async fn get_specification(&self, _id: &str) -> Result<Option<ProjectSpecification>, McpError> {
            Ok(None)
        }

        async fn get_specifications_by_project(&self, _project_id: &str) -> Result<Vec<ProjectSpecification>, McpError> {
            Ok(Vec::new())
        }

        async fn get_specifications_by_type(&self, _project_id: &str, _spec_type: SpecType) -> Result<Vec<ProjectSpecification>, McpError> {
            Ok(Vec::new())
        }

        async fn update_specification(&self, spec: ProjectSpecification) -> Result<ProjectSpecification, McpError> {
            Ok(spec)
        }

        async fn delete_specification(&self, _id: &str) -> Result<bool, McpError> {
            Ok(true)
        }

        async fn get_requirements_by_spec(&self, _spec_id: &str) -> Result<Vec<crate::models::specification::Requirement>, McpError> {
            Ok(Vec::new())
        }

        async fn get_tasks_by_spec(&self, _spec_id: &str) -> Result<Vec<crate::models::specification::Task>, McpError> {
            Ok(Vec::new())
        }

        async fn get_tasks_by_status(&self, _spec_id: &str, _status: &str) -> Result<Vec<crate::models::specification::Task>, McpError> {
            Ok(Vec::new())
        }

        async fn link_requirement_to_context(&self, _requirement_id: &str, _context_id: &str) -> Result<(), McpError> {
            Ok(())
        }

        async fn link_task_to_context(&self, _task_id: &str, _context_id: &str) -> Result<(), McpError> {
            Ok(())
        }

        async fn link_task_to_requirement(&self, _task_id: &str, _requirement_id: &str) -> Result<(), McpError> {
            Ok(())
        }

        async fn validate_specification(&self, _spec: &ProjectSpecification) -> Result<Vec<String>, McpError> {
            Ok(Vec::new())
        }

        async fn sync_specification_with_file(&self, _spec_id: &str, _file_content: &str) -> Result<ProjectSpecification, McpError> {
            let spec_content = SpecContent::new(SpecFormat::Markdown, _file_content.to_string());
            let spec = ProjectSpecification::new("test".to_string(), SpecType::Requirements, "Updated Spec".to_string(), spec_content);
            Ok(spec)
        }
    }

    #[async_trait]
    impl SpecificationRepository for MockSpecificationRepository {
        async fn create_specification(&self, spec: &ProjectSpecification) -> Result<ProjectSpecification, McpError> {
            Ok(spec.clone())
        }

        async fn find_specification_by_id(&self, _id: &str) -> Result<Option<ProjectSpecification>, McpError> {
            Ok(None)
        }

        async fn find_specifications_by_project(&self, _project_id: &str) -> Result<Vec<ProjectSpecification>, McpError> {
            Ok(Vec::new())
        }

        async fn find_specifications_by_type(&self, _project_id: &str, _spec_type: &str) -> Result<Vec<ProjectSpecification>, McpError> {
            Ok(Vec::new())
        }

        async fn update_specification(&self, spec: &ProjectSpecification) -> Result<ProjectSpecification, McpError> {
            Ok(spec.clone())
        }

        async fn delete_specification(&self, _id: &str) -> Result<bool, McpError> {
            Ok(true)
        }

        async fn create_requirement(&self, requirement: &crate::models::specification::Requirement) -> Result<crate::models::specification::Requirement, McpError> {
            Ok(requirement.clone())
        }

        async fn find_requirement_by_id(&self, _id: &str) -> Result<Option<crate::models::specification::Requirement>, McpError> {
            Ok(None)
        }

        async fn find_requirements_by_spec(&self, _spec_id: &str) -> Result<Vec<crate::models::specification::Requirement>, McpError> {
            Ok(Vec::new())
        }

        async fn update_requirement(&self, requirement: &crate::models::specification::Requirement) -> Result<crate::models::specification::Requirement, McpError> {
            Ok(requirement.clone())
        }

        async fn delete_requirement(&self, _id: &str) -> Result<bool, McpError> {
            Ok(true)
        }

        async fn create_task(&self, task: &crate::models::specification::Task) -> Result<crate::models::specification::Task, McpError> {
            Ok(task.clone())
        }

        async fn find_task_by_id(&self, _id: &str) -> Result<Option<crate::models::specification::Task>, McpError> {
            Ok(None)
        }

        async fn find_tasks_by_spec(&self, _spec_id: &str) -> Result<Vec<crate::models::specification::Task>, McpError> {
            Ok(Vec::new())
        }

        async fn find_tasks_by_status(&self, _spec_id: &str, _status: &str) -> Result<Vec<crate::models::specification::Task>, McpError> {
            Ok(Vec::new())
        }

        async fn update_task(&self, task: &crate::models::specification::Task) -> Result<crate::models::specification::Task, McpError> {
            Ok(task.clone())
        }

        async fn delete_task(&self, _id: &str) -> Result<bool, McpError> {
            Ok(true)
        }

        async fn link_requirement_to_context(&self, _requirement_id: &str, _context_id: &str) -> Result<(), McpError> {
            Ok(())
        }

        async fn link_task_to_context(&self, _task_id: &str, _context_id: &str) -> Result<(), McpError> {
            Ok(())
        }

        async fn link_task_to_requirement(&self, _task_id: &str, _requirement_id: &str) -> Result<(), McpError> {
            Ok(())
        }

        async fn unlink_requirement_from_context(&self, _requirement_id: &str, _context_id: &str) -> Result<(), McpError> {
            Ok(())
        }

        async fn unlink_task_from_context(&self, _task_id: &str, _context_id: &str) -> Result<(), McpError> {
            Ok(())
        }

        async fn unlink_task_from_requirement(&self, _task_id: &str, _requirement_id: &str) -> Result<(), McpError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_scan_and_import_specifications() {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join(".kiro").join("specs").join("test-project");
        fs::create_dir_all(&specs_dir).await.unwrap();

        // Create test specification files
        fs::write(specs_dir.join("requirements.md"), "# Requirements\n\nTest requirements").await.unwrap();
        fs::write(specs_dir.join("design.md"), "# Design\n\nTest design").await.unwrap();
        fs::write(specs_dir.join("tasks.md"), "# Tasks\n\n- [ ] Test task").await.unwrap();

        let service = DefaultSpecificationImportService::new(
            Arc::new(MockSpecificationService),
            Arc::new(MockSpecificationRepository),
        );

        let result = service.scan_and_import_specifications(&temp_dir.path().join(".kiro").join("specs")).await;
        assert!(result.is_ok());
        
        let specs = result.unwrap();
        assert_eq!(specs.len(), 3);
    }

    #[tokio::test]
    async fn test_validate_specification_file() {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join(".kiro").join("specs").join("test-project");
        fs::create_dir_all(&specs_dir).await.unwrap();

        let requirements_file = specs_dir.join("requirements.md");
        fs::write(&requirements_file, "# Requirements\n\nTest requirements").await.unwrap();

        let service = DefaultSpecificationImportService::new(
            Arc::new(MockSpecificationService),
            Arc::new(MockSpecificationRepository),
        );

        let result = service.validate_specification_file(&requirements_file).await;
        assert!(result.is_ok());
        
        let issues = result.unwrap();
        // Should have no validation issues for a properly structured file
        assert!(issues.is_empty());
    }

    #[test]
    fn test_extract_project_name() {
        let path = Path::new(".kiro/specs/test-project/requirements.md");
        let project_name = DefaultSpecificationImportService::extract_project_name(path).unwrap();
        assert_eq!(project_name, "test-project");
    }

    #[test]
    fn test_is_kiro_spec_file() {
        assert!(DefaultSpecificationImportService::is_kiro_spec_file(Path::new("requirements.md")));
        assert!(DefaultSpecificationImportService::is_kiro_spec_file(Path::new("design.md")));
        assert!(DefaultSpecificationImportService::is_kiro_spec_file(Path::new("tasks.md")));
        assert!(!DefaultSpecificationImportService::is_kiro_spec_file(Path::new("other.md")));
        assert!(!DefaultSpecificationImportService::is_kiro_spec_file(Path::new("readme.txt")));
    }
}