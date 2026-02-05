use crate::models::specification::{ProjectSpecification, Requirement, Task, SpecType};
use crate::repositories::SpecificationRepository;
use crate::services::SpecificationParser;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;
use std::sync::Arc;

/// Service interface for specification operations
#[async_trait]
pub trait SpecificationService: Send + Sync {
    async fn import_specification_from_file(
        &self,
        project_id: String,
        file_path: &str,
        content: &str,
    ) -> Result<ProjectSpecification, McpError>;
    
    async fn get_specification(&self, id: &str) -> Result<Option<ProjectSpecification>, McpError>;
    async fn get_specifications_by_project(&self, project_id: &str) -> Result<Vec<ProjectSpecification>, McpError>;
    async fn get_specifications_by_type(&self, project_id: &str, spec_type: SpecType) -> Result<Vec<ProjectSpecification>, McpError>;
    async fn update_specification(&self, spec: ProjectSpecification) -> Result<ProjectSpecification, McpError>;
    async fn delete_specification(&self, id: &str) -> Result<bool, McpError>;
    
    async fn get_requirements_by_spec(&self, spec_id: &str) -> Result<Vec<Requirement>, McpError>;
    async fn get_tasks_by_spec(&self, spec_id: &str) -> Result<Vec<Task>, McpError>;
    async fn get_tasks_by_status(&self, spec_id: &str, status: &str) -> Result<Vec<Task>, McpError>;
    
    async fn link_requirement_to_context(&self, requirement_id: &str, context_id: &str) -> Result<(), McpError>;
    async fn link_task_to_context(&self, task_id: &str, context_id: &str) -> Result<(), McpError>;
    async fn link_task_to_requirement(&self, task_id: &str, requirement_id: &str) -> Result<(), McpError>;
    
    async fn validate_specification(&self, spec: &ProjectSpecification) -> Result<Vec<String>, McpError>;
    async fn sync_specification_with_file(&self, spec_id: &str, file_content: &str) -> Result<ProjectSpecification, McpError>;
}

/// Default implementation of SpecificationService
pub struct DefaultSpecificationService {
    repository: Arc<dyn SpecificationRepository>,
}

impl DefaultSpecificationService {
    pub fn new(repository: Arc<dyn SpecificationRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl SpecificationService for DefaultSpecificationService {
    async fn import_specification_from_file(
        &self,
        project_id: String,
        file_path: &str,
        content: &str,
    ) -> Result<ProjectSpecification, McpError> {
        // Parse the specification
        let mut spec = SpecificationParser::parse_specification(project_id, file_path, content)
            .map_err(|e| McpError::internal_error(format!("Failed to parse specification: {}", e), None))?;

        // Validate the specification
        let validation_issues = SpecificationParser::validate_specification(&spec)
            .map_err(|e| McpError::internal_error(format!("Failed to validate specification: {}", e), None))?;

        if !validation_issues.is_empty() {
            tracing::warn!("Specification validation issues: {:?}", validation_issues);
        }

        // Store the specification
        let stored_spec = self.repository.create_specification(&spec).await?;

        // Parse and store requirements if this is a requirements spec
        if spec.spec_type == SpecType::Requirements {
            let requirements = SpecificationParser::parse_requirements_from_markdown(content, spec.id.clone())
                .map_err(|e| McpError::internal_error(format!("Failed to parse requirements: {}", e), None))?;

            for requirement in requirements {
                let stored_req = self.repository.create_requirement(&requirement).await?;
                spec.requirements.push(stored_req.id);
            }
        }

        // Parse and store tasks if this is a tasks spec
        if spec.spec_type == SpecType::Tasks {
            let tasks = SpecificationParser::parse_tasks_from_markdown(content, spec.id.clone())
                .map_err(|e| McpError::internal_error(format!("Failed to parse tasks: {}", e), None))?;

            for task in tasks {
                let stored_task = self.repository.create_task(&task).await?;
                spec.tasks.push(stored_task.id);
            }
        }

        // Update the specification with the linked requirements and tasks
        if !spec.requirements.is_empty() || !spec.tasks.is_empty() {
            self.repository.update_specification(&spec).await
        } else {
            Ok(stored_spec)
        }
    }

    async fn get_specification(&self, id: &str) -> Result<Option<ProjectSpecification>, McpError> {
        let mut spec = match self.repository.find_specification_by_id(id).await? {
            Some(spec) => spec,
            None => return Ok(None),
        };

        // Load requirements
        let requirements = self.repository.find_requirements_by_spec(id).await?;
        spec.requirements = requirements.into_iter().map(|r| r.id).collect();

        // Load tasks
        let tasks = self.repository.find_tasks_by_spec(id).await?;
        spec.tasks = tasks.into_iter().map(|t| t.id).collect();

        Ok(Some(spec))
    }

    async fn get_specifications_by_project(&self, project_id: &str) -> Result<Vec<ProjectSpecification>, McpError> {
        let mut specifications = self.repository.find_specifications_by_project(project_id).await?;

        // Load requirements and tasks for each specification
        for spec in &mut specifications {
            let requirements = self.repository.find_requirements_by_spec(&spec.id).await?;
            spec.requirements = requirements.into_iter().map(|r| r.id).collect();

            let tasks = self.repository.find_tasks_by_spec(&spec.id).await?;
            spec.tasks = tasks.into_iter().map(|t| t.id).collect();
        }

        Ok(specifications)
    }

    async fn get_specifications_by_type(&self, project_id: &str, spec_type: SpecType) -> Result<Vec<ProjectSpecification>, McpError> {
        let mut specifications = self.repository.find_specifications_by_type(project_id, spec_type.as_str()).await?;

        // Load requirements and tasks for each specification
        for spec in &mut specifications {
            let requirements = self.repository.find_requirements_by_spec(&spec.id).await?;
            spec.requirements = requirements.into_iter().map(|r| r.id).collect();

            let tasks = self.repository.find_tasks_by_spec(&spec.id).await?;
            spec.tasks = tasks.into_iter().map(|t| t.id).collect();
        }

        Ok(specifications)
    }

    async fn update_specification(&self, spec: ProjectSpecification) -> Result<ProjectSpecification, McpError> {
        self.repository.update_specification(&spec).await
    }

    async fn delete_specification(&self, id: &str) -> Result<bool, McpError> {
        self.repository.delete_specification(id).await
    }

    async fn get_requirements_by_spec(&self, spec_id: &str) -> Result<Vec<Requirement>, McpError> {
        self.repository.find_requirements_by_spec(spec_id).await
    }

    async fn get_tasks_by_spec(&self, spec_id: &str) -> Result<Vec<Task>, McpError> {
        self.repository.find_tasks_by_spec(spec_id).await
    }

    async fn get_tasks_by_status(&self, spec_id: &str, status: &str) -> Result<Vec<Task>, McpError> {
        self.repository.find_tasks_by_status(spec_id, status).await
    }

    async fn link_requirement_to_context(&self, requirement_id: &str, context_id: &str) -> Result<(), McpError> {
        self.repository.link_requirement_to_context(requirement_id, context_id).await
    }

    async fn link_task_to_context(&self, task_id: &str, context_id: &str) -> Result<(), McpError> {
        self.repository.link_task_to_context(task_id, context_id).await
    }

    async fn link_task_to_requirement(&self, task_id: &str, requirement_id: &str) -> Result<(), McpError> {
        self.repository.link_task_to_requirement(task_id, requirement_id).await
    }

    async fn validate_specification(&self, spec: &ProjectSpecification) -> Result<Vec<String>, McpError> {
        SpecificationParser::validate_specification(spec)
            .map_err(|e| McpError::internal_error(format!("Failed to validate specification: {}", e), None))
    }

    async fn sync_specification_with_file(&self, spec_id: &str, file_content: &str) -> Result<ProjectSpecification, McpError> {
        let mut spec = self.repository.find_specification_by_id(spec_id).await?
            .ok_or_else(|| McpError::resource_not_found(format!("Specification not found: {}", spec_id), None))?;

        // Parse the new content
        let file_path = spec.file_path.as_deref().unwrap_or("unknown");
        let parsed_spec = SpecificationParser::parse_specification(spec.project_id.clone(), file_path, file_content)
            .map_err(|e| McpError::internal_error(format!("Failed to parse specification: {}", e), None))?;

        // Update the specification content
        spec.update_content(parsed_spec.content);

        // If this is a requirements spec, sync requirements
        if spec.spec_type == SpecType::Requirements {
            let new_requirements = SpecificationParser::parse_requirements_from_markdown(file_content, spec.id.clone())
                .map_err(|e| McpError::internal_error(format!("Failed to parse requirements: {}", e), None))?;

            // For simplicity, we'll replace all requirements
            // In a production system, you might want to do a more sophisticated merge
            for req_id in &spec.requirements {
                self.repository.delete_requirement(req_id).await?;
            }

            let mut new_req_ids = Vec::new();
            for requirement in new_requirements {
                let stored_req = self.repository.create_requirement(&requirement).await?;
                new_req_ids.push(stored_req.id);
            }
            spec.requirements = new_req_ids;
        }

        // If this is a tasks spec, sync tasks
        if spec.spec_type == SpecType::Tasks {
            let new_tasks = SpecificationParser::parse_tasks_from_markdown(file_content, spec.id.clone())
                .map_err(|e| McpError::internal_error(format!("Failed to parse tasks: {}", e), None))?;

            // For simplicity, we'll replace all tasks
            // In a production system, you might want to do a more sophisticated merge
            for task_id in &spec.tasks {
                self.repository.delete_task(task_id).await?;
            }

            let mut new_task_ids = Vec::new();
            for task in new_tasks {
                let stored_task = self.repository.create_task(&task).await?;
                new_task_ids.push(stored_task.id);
            }
            spec.tasks = new_task_ids;
        }

        // Update the specification
        self.repository.update_specification(&spec).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::specification::{SpecContent, SpecFormat, SpecStatus};
    use crate::repositories::SpecificationRepository;
    use async_trait::async_trait;
    use std::collections::HashMap;

    // Mock repository for testing
    struct MockSpecificationRepository;

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

        async fn create_requirement(&self, requirement: &Requirement) -> Result<Requirement, McpError> {
            Ok(requirement.clone())
        }

        async fn find_requirement_by_id(&self, _id: &str) -> Result<Option<Requirement>, McpError> {
            Ok(None)
        }

        async fn find_requirements_by_spec(&self, _spec_id: &str) -> Result<Vec<Requirement>, McpError> {
            Ok(Vec::new())
        }

        async fn update_requirement(&self, requirement: &Requirement) -> Result<Requirement, McpError> {
            Ok(requirement.clone())
        }

        async fn delete_requirement(&self, _id: &str) -> Result<bool, McpError> {
            Ok(true)
        }

        async fn create_task(&self, task: &Task) -> Result<Task, McpError> {
            Ok(task.clone())
        }

        async fn find_task_by_id(&self, _id: &str) -> Result<Option<Task>, McpError> {
            Ok(None)
        }

        async fn find_tasks_by_spec(&self, _spec_id: &str) -> Result<Vec<Task>, McpError> {
            Ok(Vec::new())
        }

        async fn find_tasks_by_status(&self, _spec_id: &str, _status: &str) -> Result<Vec<Task>, McpError> {
            Ok(Vec::new())
        }

        async fn update_task(&self, task: &Task) -> Result<Task, McpError> {
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
    async fn test_import_specification_from_file() {
        let repository = Arc::new(MockSpecificationRepository);
        let service = DefaultSpecificationService::new(repository);

        let content = r#"
# Test Specification

This is a test specification.

## Overview

This section contains the overview.
"#;

        let result = service.import_specification_from_file(
            "test-project".to_string(),
            "test.md",
            content,
        ).await;

        assert!(result.is_ok());
        let spec = result.unwrap();
        assert_eq!(spec.project_id, "test-project");
        assert_eq!(spec.title, "Test Specification");
    }

    #[tokio::test]
    async fn test_validate_specification() {
        let repository = Arc::new(MockSpecificationRepository);
        let service = DefaultSpecificationService::new(repository);

        let spec = ProjectSpecification::new(
            "test".to_string(),
            SpecType::Requirements,
            "Test Spec".to_string(),
            SpecContent::new(SpecFormat::Markdown, "# Test\n\nRequirement 1\n\nAcceptance Criteria".to_string()),
        );

        let result = service.validate_specification(&spec).await;
        assert!(result.is_ok());
        let issues = result.unwrap();
        assert!(issues.is_empty());
    }
}