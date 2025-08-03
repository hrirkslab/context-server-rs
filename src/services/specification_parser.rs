use crate::models::specification::{
    AcceptanceCriterion, CriterionType, ProjectSpecification, Requirement, RequirementStatus,
    SpecContent, SpecFormat, SpecType, Task, TaskStatus, TaskType,
};
use anyhow::{anyhow, Result};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

/// Parser for different specification formats
pub struct SpecificationParser;

impl SpecificationParser {
    /// Parse specification content based on format
    pub fn parse_specification(
        project_id: String,
        file_path: &str,
        content: &str,
    ) -> Result<ProjectSpecification> {
        let format = Self::detect_format(file_path)?;
        let spec_type = SpecType::from_filename(
            std::path::Path::new(file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown"),
        );

        let spec_content = match format {
            SpecFormat::Markdown => Self::parse_markdown(content)?,
            SpecFormat::YAML => Self::parse_yaml(content)?,
            SpecFormat::JSON => Self::parse_json(content)?,
            SpecFormat::PlainText => Self::parse_plain_text(content)?,
            SpecFormat::Custom(_) => Self::parse_plain_text(content)?,
        };

        let title = Self::extract_title(&spec_content, &spec_type);

        let mut spec = ProjectSpecification::new(project_id, spec_type, title, spec_content);
        spec.file_path = Some(file_path.to_string());

        Ok(spec)
    }

    /// Detect format from file extension
    fn detect_format(file_path: &str) -> Result<SpecFormat> {
        let extension = std::path::Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow!("Could not determine file extension"))?;

        Ok(SpecFormat::from_extension(extension))
    }

    /// Parse Markdown content
    fn parse_markdown(content: &str) -> Result<SpecContent> {
        let mut sections = HashMap::new();
        let mut current_section = String::new();
        let mut current_content = String::new();

        // Regex for markdown headers
        let header_regex = Regex::new(r"^(#{1,6})\s+(.+)$").unwrap();

        for line in content.lines() {
            if let Some(captures) = header_regex.captures(line) {
                // Save previous section if it exists
                if !current_section.is_empty() && !current_content.trim().is_empty() {
                    sections.insert(current_section.clone(), current_content.trim().to_string());
                }

                // Start new section
                let level = captures[1].len();
                let title = captures[2].to_string();
                current_section = format!("h{}-{}", level, Self::slugify(&title));
                current_content = String::new();
            } else {
                current_content.push_str(line);
                current_content.push('\n');
            }
        }

        // Save last section
        if !current_section.is_empty() && !current_content.trim().is_empty() {
            sections.insert(current_section, current_content.trim().to_string());
        }

        Ok(SpecContent::new(SpecFormat::Markdown, content.to_string()).with_sections(sections))
    }

    /// Parse YAML content
    fn parse_yaml(content: &str) -> Result<SpecContent> {
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(content)
            .map_err(|e| anyhow!("Failed to parse YAML: {}", e))?;

        let json_value: Value = serde_json::to_value(yaml_value)
            .map_err(|e| anyhow!("Failed to convert YAML to JSON: {}", e))?;

        let mut sections = HashMap::new();
        Self::extract_yaml_sections(&json_value, "", &mut sections);

        Ok(SpecContent::new(SpecFormat::YAML, content.to_string())
            .with_sections(sections)
            .with_metadata("parsed_yaml".to_string(), json_value))
    }

    /// Parse JSON content
    fn parse_json(content: &str) -> Result<SpecContent> {
        let json_value: Value = serde_json::from_str(content)
            .map_err(|e| anyhow!("Failed to parse JSON: {}", e))?;

        let mut sections = HashMap::new();
        Self::extract_json_sections(&json_value, "", &mut sections);

        Ok(SpecContent::new(SpecFormat::JSON, content.to_string())
            .with_sections(sections)
            .with_metadata("parsed_json".to_string(), json_value.clone()))
    }

    /// Parse plain text content
    fn parse_plain_text(content: &str) -> Result<SpecContent> {
        let mut sections = HashMap::new();
        sections.insert("content".to_string(), content.to_string());

        Ok(SpecContent::new(SpecFormat::PlainText, content.to_string()).with_sections(sections))
    }

    /// Extract sections from YAML value recursively
    fn extract_yaml_sections(value: &Value, prefix: &str, sections: &mut HashMap<String, String>) {
        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    let section_key = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };

                    match val {
                        Value::String(s) => {
                            sections.insert(section_key, s.clone());
                        }
                        Value::Array(arr) => {
                            let content = arr
                                .iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<_>>()
                                .join("\n");
                            sections.insert(section_key, content);
                        }
                        _ => {
                            Self::extract_yaml_sections(val, &section_key, sections);
                        }
                    }
                }
            }
            Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    let section_key = format!("{}[{}]", prefix, i);
                    Self::extract_yaml_sections(val, &section_key, sections);
                }
            }
            _ => {
                if !prefix.is_empty() {
                    sections.insert(prefix.to_string(), value.to_string());
                }
            }
        }
    }

    /// Extract sections from JSON value recursively
    fn extract_json_sections(value: &Value, prefix: &str, sections: &mut HashMap<String, String>) {
        Self::extract_yaml_sections(value, prefix, sections); // Same logic as YAML
    }

    /// Extract title from parsed content
    fn extract_title(content: &SpecContent, spec_type: &SpecType) -> String {
        // Try to find title in sections
        for (key, value) in &content.parsed_sections {
            if key.contains("title") || key.starts_with("h1-") {
                return value.lines().next().unwrap_or("Untitled").to_string();
            }
        }

        // Fallback to spec type
        match spec_type {
            SpecType::Requirements => "Requirements Document".to_string(),
            SpecType::Design => "Design Document".to_string(),
            SpecType::Tasks => "Implementation Plan".to_string(),
            _ => format!("{} Specification", spec_type.as_str()),
        }
    }

    /// Convert string to URL-friendly slug
    fn slugify(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Parse requirements from markdown content
    pub fn parse_requirements_from_markdown(content: &str, spec_id: String) -> Result<Vec<Requirement>> {
        let mut requirements = Vec::new();
        let mut current_requirement: Option<Requirement> = None;
        let mut in_acceptance_criteria = false;

        // Regex patterns
        let requirement_header = Regex::new(r"^###\s+Requirement\s+(\d+)").unwrap();
        let user_story = Regex::new(r"^\*\*User Story:\*\*\s+(.+)$").unwrap();
        let acceptance_criteria_header = Regex::new(r"^####\s+Acceptance Criteria").unwrap();
        let criteria_item = Regex::new(r"^\d+\.\s+(.+)$").unwrap();

        for line in content.lines() {
            let line = line.trim();

            // Check for requirement header
            if let Some(captures) = requirement_header.captures(line) {
                // Save previous requirement
                if let Some(req) = current_requirement.take() {
                    requirements.push(req);
                }

                // Start new requirement
                let req_number = captures[1].to_string();
                current_requirement = Some(Requirement::new(
                    spec_id.clone(),
                    format!("Requirement {}", req_number),
                    String::new(),
                ));
                in_acceptance_criteria = false;
            }
            // Check for user story
            else if let Some(captures) = user_story.captures(line) {
                if let Some(ref mut req) = current_requirement {
                    req.user_story = Some(captures[1].to_string());
                    req.description = captures[1].to_string();
                }
            }
            // Check for acceptance criteria header
            else if acceptance_criteria_header.is_match(line) {
                in_acceptance_criteria = true;
            }
            // Check for criteria items
            else if in_acceptance_criteria {
                if let Some(captures) = criteria_item.captures(line) {
                    if let Some(ref mut req) = current_requirement {
                        let criterion = AcceptanceCriterion::new(
                            captures[1].to_string(),
                            CriterionType::Functional,
                        );
                        req.add_acceptance_criterion(criterion);
                    }
                }
            }
        }

        // Save last requirement
        if let Some(req) = current_requirement {
            requirements.push(req);
        }

        Ok(requirements)
    }

    /// Parse tasks from markdown content
    pub fn parse_tasks_from_markdown(content: &str, spec_id: String) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();
        let mut task_stack: Vec<(usize, String)> = Vec::new(); // (level, task_id)

        // Regex patterns
        let task_item = Regex::new(r"^(\s*)-\s+\[([x\-\s])\]\s+(\d+(?:\.\d+)*\.?)\s+(.+)$").unwrap();
        let task_details = Regex::new(r"^\s*-\s+(.+)$").unwrap();
        let requirements_ref = Regex::new(r"_Requirements:\s+([^_]+)_").unwrap();

        let mut current_task: Option<Task> = None;
        let mut in_task_details = false;

        for line in content.lines() {
            // Check for task item
            if let Some(captures) = task_item.captures(line) {
                // Save previous task
                if let Some(task) = current_task.take() {
                    tasks.push(task);
                }

                let indent_level = captures[1].len() / 2; // Assuming 2 spaces per level
                let status_char = &captures[2];
                let task_number = captures[3].to_string();
                let task_title = captures[4].to_string();

                let status = TaskStatus::from_checkbox_status(status_char);
                let task_type = Self::infer_task_type(&task_title);

                let mut task = Task::new(spec_id.clone(), task_title, String::new());
                task.status = status;
                task.task_type = task_type;

                // Handle task hierarchy
                task_stack.truncate(indent_level);
                if let Some((_, parent_id)) = task_stack.last() {
                    task.parent_task = Some(parent_id.clone());
                }
                task_stack.push((indent_level, task.id.clone()));

                current_task = Some(task);
                in_task_details = true;
            }
            // Check for task details
            else if in_task_details {
                if let Some(captures) = task_details.captures(line) {
                    if let Some(ref mut task) = current_task {
                        let detail = captures[1].to_string();
                        
                        // Check for requirements reference
                        if let Some(req_captures) = requirements_ref.captures(&detail) {
                            let req_refs = req_captures[1].to_string();
                            task.metadata.custom_fields.insert(
                                "requirements".to_string(),
                                serde_json::Value::String(req_refs),
                            );
                        } else {
                            // Add to description
                            if !task.description.is_empty() {
                                task.description.push('\n');
                            }
                            task.description.push_str(&detail);
                        }
                    }
                } else if line.trim().is_empty() {
                    in_task_details = false;
                }
            }
        }

        // Save last task
        if let Some(task) = current_task {
            tasks.push(task);
        }

        // Set up parent-child relationships
        let mut parent_child_pairs = Vec::new();
        for task in &tasks {
            if let Some(parent_id) = &task.parent_task {
                parent_child_pairs.push((parent_id.clone(), task.id.clone()));
            }
        }
        
        for (parent_id, child_id) in parent_child_pairs {
            if let Some(parent) = tasks.iter_mut().find(|t| t.id == parent_id) {
                parent.add_subtask(child_id);
            }
        }

        Ok(tasks)
    }

    /// Infer task type from title
    fn infer_task_type(title: &str) -> TaskType {
        let title_lower = title.to_lowercase();
        
        if title_lower.contains("test") {
            TaskType::Testing
        } else if title_lower.contains("document") || title_lower.contains("doc") {
            TaskType::Documentation
        } else if title_lower.contains("research") || title_lower.contains("investigate") {
            TaskType::Research
        } else if title_lower.contains("design") {
            TaskType::Design
        } else if title_lower.contains("review") {
            TaskType::Review
        } else if title_lower.contains("deploy") {
            TaskType::Deployment
        } else {
            TaskType::Implementation
        }
    }

    /// Validate specification content
    pub fn validate_specification(spec: &ProjectSpecification) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        // Basic validation
        if spec.title.trim().is_empty() {
            issues.push("Specification title is empty".to_string());
        }

        if spec.content.raw_content.trim().is_empty() {
            issues.push("Specification content is empty".to_string());
        }

        // Format-specific validation
        match spec.content.format {
            SpecFormat::Markdown => {
                Self::validate_markdown_spec(&spec.content, &mut issues);
            }
            SpecFormat::YAML => {
                Self::validate_yaml_spec(&spec.content, &mut issues);
            }
            SpecFormat::JSON => {
                Self::validate_json_spec(&spec.content, &mut issues);
            }
            _ => {}
        }

        // Type-specific validation
        match spec.spec_type {
            SpecType::Requirements => {
                Self::validate_requirements_spec(&spec.content, &mut issues);
            }
            SpecType::Tasks => {
                Self::validate_tasks_spec(&spec.content, &mut issues);
            }
            _ => {}
        }

        Ok(issues)
    }

    /// Validate markdown specification
    fn validate_markdown_spec(content: &SpecContent, issues: &mut Vec<String>) {
        if !content.raw_content.contains('#') {
            issues.push("Markdown specification should contain headers".to_string());
        }
    }

    /// Validate YAML specification
    fn validate_yaml_spec(content: &SpecContent, issues: &mut Vec<String>) {
        if content.parsed_sections.is_empty() {
            issues.push("YAML specification could not be parsed into sections".to_string());
        }
    }

    /// Validate JSON specification
    fn validate_json_spec(content: &SpecContent, issues: &mut Vec<String>) {
        if content.parsed_sections.is_empty() {
            issues.push("JSON specification could not be parsed into sections".to_string());
        }
    }

    /// Validate requirements specification
    fn validate_requirements_spec(content: &SpecContent, issues: &mut Vec<String>) {
        let has_requirements = content.raw_content.contains("Requirement") 
            || content.raw_content.contains("requirement");
        
        if !has_requirements {
            issues.push("Requirements specification should contain requirement definitions".to_string());
        }

        let has_acceptance_criteria = content.raw_content.contains("Acceptance Criteria") 
            || content.raw_content.contains("acceptance criteria");
        
        if !has_acceptance_criteria {
            issues.push("Requirements specification should contain acceptance criteria".to_string());
        }
    }

    /// Validate tasks specification
    fn validate_tasks_spec(content: &SpecContent, issues: &mut Vec<String>) {
        let has_checkboxes = content.raw_content.contains("[ ]") 
            || content.raw_content.contains("[x]") 
            || content.raw_content.contains("[-]");
        
        if !has_checkboxes {
            issues.push("Tasks specification should contain checkbox items".to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_specification() {
        let content = r#"
# Test Specification

This is a test specification.

## Overview

This section contains the overview.

## Requirements

This section contains requirements.
"#;

        let spec = SpecificationParser::parse_specification(
            "test-project".to_string(),
            "test.md",
            content,
        ).unwrap();

        assert_eq!(spec.spec_type, SpecType::Custom("test.md".to_string()));
        assert_eq!(spec.title, "This is a test specification.");
        assert_eq!(spec.content.format, SpecFormat::Markdown);
        assert!(spec.content.parsed_sections.contains_key("h1-test-specification"));
    }

    #[test]
    fn test_parse_requirements_from_markdown() {
        let content = r#"
### Requirement 1

**User Story:** As a user, I want to login, so that I can access the system.

#### Acceptance Criteria

1. WHEN user enters valid credentials THEN system SHALL authenticate user
2. WHEN user enters invalid credentials THEN system SHALL show error message
"#;

        let requirements = SpecificationParser::parse_requirements_from_markdown(
            content,
            "spec-1".to_string(),
        ).unwrap();

        assert_eq!(requirements.len(), 1);
        assert_eq!(requirements[0].title, "Requirement 1");
        assert_eq!(requirements[0].acceptance_criteria.len(), 2);
    }

    #[test]
    fn test_parse_tasks_from_markdown() {
        let content = r#"
- [x] 1. Complete first task
  - This task is completed
  - _Requirements: 1.1, 1.2_

- [ ] 2. Start second task
  - [ ] 2.1 Subtask one
  - [ ] 2.2 Subtask two
"#;

        let tasks = SpecificationParser::parse_tasks_from_markdown(
            content,
            "spec-1".to_string(),
        ).unwrap();



        assert_eq!(tasks.len(), 4);
        assert_eq!(tasks[0].status, TaskStatus::Completed);
        assert_eq!(tasks[1].status, TaskStatus::NotStarted);
        // Find the parent task "Start second task"
        let parent_task = tasks.iter().find(|t| t.title.contains("Start second task")).unwrap();
        assert_eq!(parent_task.subtasks.len(), 2);
    }

    #[test]
    fn test_validate_specification() {
        let spec = ProjectSpecification::new(
            "test".to_string(),
            SpecType::Requirements,
            "Test Spec".to_string(),
            SpecContent::new(SpecFormat::Markdown, "# Test\n\nRequirement 1\n\nAcceptance Criteria".to_string()),
        );

        let issues = SpecificationParser::validate_specification(&spec).unwrap();
        assert!(issues.is_empty());
    }
}