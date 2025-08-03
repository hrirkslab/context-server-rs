#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_project_specification_creation() {
        let content = SpecContent::new(
            SpecFormat::Markdown,
            "# Test Spec\n\nThis is a test specification.".to_string(),
        );

        let spec = ProjectSpecification::new(
            "project-1".to_string(),
            SpecType::Feature,
            "Test Feature".to_string(),
            content,
        );

        assert_eq!(spec.project_id, "project-1");
        assert_eq!(spec.spec_type, SpecType::Feature);
        assert_eq!(spec.title, "Test Feature");
        assert_eq!(spec.status, SpecStatus::Draft);
        assert_eq!(spec.version, 1);
        assert!(spec.requirements.is_empty());
        assert!(spec.tasks.is_empty());
    }

    #[test]
    fn test_project_specification_update_content() {
        let content = SpecContent::new(
            SpecFormat::Markdown,
            "# Test Spec\n\nThis is a test specification.".to_string(),
        );

        let mut spec = ProjectSpecification::new(
            "project-1".to_string(),
            SpecType::Feature,
            "Test Feature".to_string(),
            content,
        );

        let original_version = spec.version;
        let original_updated_at = spec.updated_at;

        // Wait a bit to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        let new_content = SpecContent::new(
            SpecFormat::Markdown,
            "# Updated Test Spec\n\nThis is an updated test specification.".to_string(),
        );

        spec.update_content(new_content);

        assert_eq!(spec.version, original_version + 1);
        assert!(spec.updated_at > original_updated_at);
        assert!(spec.content.raw_content.contains("Updated"));
    }

    #[test]
    fn test_spec_type_from_filename() {
        assert_eq!(SpecType::from_filename("requirements.md"), SpecType::Requirements);
        assert_eq!(SpecType::from_filename("design.md"), SpecType::Design);
        assert_eq!(SpecType::from_filename("tasks.md"), SpecType::Tasks);
        assert_eq!(SpecType::from_filename("api-spec.md"), SpecType::API);
        assert_eq!(SpecType::from_filename("architecture-doc.md"), SpecType::Architecture);
        assert_eq!(SpecType::from_filename("unknown.md"), SpecType::Custom("unknown.md".to_string()));
    }

    #[test]
    fn test_spec_format_from_extension() {
        assert_eq!(SpecFormat::from_extension("md"), SpecFormat::Markdown);
        assert_eq!(SpecFormat::from_extension("yaml"), SpecFormat::YAML);
        assert_eq!(SpecFormat::from_extension("json"), SpecFormat::JSON);
        assert_eq!(SpecFormat::from_extension("txt"), SpecFormat::PlainText);
        assert_eq!(SpecFormat::from_extension("unknown"), SpecFormat::Custom("unknown".to_string()));
    }

    #[test]
    fn test_requirement_creation() {
        let requirement = Requirement::new(
            "spec-1".to_string(),
            "User Authentication".to_string(),
            "Users should be able to authenticate".to_string(),
        );

        assert_eq!(requirement.spec_id, "spec-1");
        assert_eq!(requirement.title, "User Authentication");
        assert_eq!(requirement.status, RequirementStatus::Draft);
        assert_eq!(requirement.priority, Priority::Medium);
        assert!(requirement.acceptance_criteria.is_empty());
        assert!(requirement.linked_tasks.is_empty());
        assert!(requirement.linked_context.is_empty());
    }

    #[test]
    fn test_requirement_with_user_story() {
        let requirement = Requirement::new(
            "spec-1".to_string(),
            "User Authentication".to_string(),
            "Users should be able to authenticate".to_string(),
        ).with_user_story("As a user, I want to login so that I can access the system".to_string());

        assert!(requirement.user_story.is_some());
        assert_eq!(
            requirement.user_story.unwrap(),
            "As a user, I want to login so that I can access the system"
        );
    }

    #[test]
    fn test_requirement_add_acceptance_criterion() {
        let mut requirement = Requirement::new(
            "spec-1".to_string(),
            "User Authentication".to_string(),
            "Users should be able to authenticate".to_string(),
        );

        let criterion = AcceptanceCriterion::new(
            "WHEN user enters valid credentials THEN system SHALL authenticate user".to_string(),
            CriterionType::Functional,
        );

        requirement.add_acceptance_criterion(criterion);

        assert_eq!(requirement.acceptance_criteria.len(), 1);
        assert_eq!(
            requirement.acceptance_criteria[0].description,
            "WHEN user enters valid credentials THEN system SHALL authenticate user"
        );
    }

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "spec-1".to_string(),
            "Implement login form".to_string(),
            "Create a login form with username and password fields".to_string(),
        );

        assert_eq!(task.spec_id, "spec-1");
        assert_eq!(task.title, "Implement login form");
        assert_eq!(task.status, TaskStatus::NotStarted);
        assert_eq!(task.task_type, TaskType::Implementation);
        assert_eq!(task.progress, 0.0);
        assert!(task.dependencies.is_empty());
        assert!(task.subtasks.is_empty());
        assert!(task.started_at.is_none());
        assert!(task.completed_at.is_none());
    }

    #[test]
    fn test_task_update_status() {
        let mut task = Task::new(
            "spec-1".to_string(),
            "Implement login form".to_string(),
            "Create a login form with username and password fields".to_string(),
        );

        let original_updated_at = task.updated_at;

        // Wait a bit to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        task.update_status(TaskStatus::InProgress);

        assert_eq!(task.status, TaskStatus::InProgress);
        assert!(task.started_at.is_some());
        assert!(task.updated_at > original_updated_at);

        task.update_status(TaskStatus::Completed);

        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.completed_at.is_some());
        assert_eq!(task.progress, 1.0);
    }

    #[test]
    fn test_task_update_progress() {
        let mut task = Task::new(
            "spec-1".to_string(),
            "Implement login form".to_string(),
            "Create a login form with username and password fields".to_string(),
        );

        task.update_progress(0.5);

        assert_eq!(task.progress, 0.5);
        assert_eq!(task.status, TaskStatus::InProgress);

        task.update_progress(1.0);

        assert_eq!(task.progress, 1.0);
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn test_task_status_from_checkbox_status() {
        assert_eq!(TaskStatus::from_checkbox_status("[ ]"), TaskStatus::NotStarted);
        assert_eq!(TaskStatus::from_checkbox_status("[x]"), TaskStatus::Completed);
        assert_eq!(TaskStatus::from_checkbox_status("[-]"), TaskStatus::InProgress);
        assert_eq!(TaskStatus::from_checkbox_status("unknown"), TaskStatus::NotStarted);
    }

    #[test]
    fn test_task_add_dependency() {
        let mut task = Task::new(
            "spec-1".to_string(),
            "Implement login form".to_string(),
            "Create a login form with username and password fields".to_string(),
        );

        task.add_dependency("task-1".to_string());
        task.add_dependency("task-2".to_string());
        task.add_dependency("task-1".to_string()); // Duplicate should be ignored

        assert_eq!(task.dependencies.len(), 2);
        assert!(task.dependencies.contains(&"task-1".to_string()));
        assert!(task.dependencies.contains(&"task-2".to_string()));
    }

    #[test]
    fn test_task_link_requirement() {
        let mut task = Task::new(
            "spec-1".to_string(),
            "Implement login form".to_string(),
            "Create a login form with username and password fields".to_string(),
        );

        task.link_requirement("req-1".to_string());
        task.link_requirement("req-2".to_string());
        task.link_requirement("req-1".to_string()); // Duplicate should be ignored

        assert_eq!(task.linked_requirements.len(), 2);
        assert!(task.linked_requirements.contains(&"req-1".to_string()));
        assert!(task.linked_requirements.contains(&"req-2".to_string()));
    }

    #[test]
    fn test_priority_score() {
        assert_eq!(Priority::Critical.score(), 1.0);
        assert_eq!(Priority::High.score(), 0.8);
        assert_eq!(Priority::Medium.score(), 0.6);
        assert_eq!(Priority::Low.score(), 0.4);
    }

    #[test]
    fn test_complexity_score() {
        assert_eq!(Complexity::Simple.score(), 0.25);
        assert_eq!(Complexity::Medium.score(), 0.5);
        assert_eq!(Complexity::Complex.score(), 0.75);
        assert_eq!(Complexity::VeryComplex.score(), 1.0);
    }

    #[test]
    fn test_spec_content_with_sections() {
        let mut sections = std::collections::HashMap::new();
        sections.insert("overview".to_string(), "This is the overview".to_string());
        sections.insert("details".to_string(), "These are the details".to_string());

        let content = SpecContent::new(
            SpecFormat::Markdown,
            "# Test\n\n## Overview\n\nThis is the overview".to_string(),
        ).with_sections(sections);

        assert_eq!(content.parsed_sections.len(), 2);
        assert_eq!(content.parsed_sections.get("overview").unwrap(), "This is the overview");
        assert_eq!(content.parsed_sections.get("details").unwrap(), "These are the details");
    }

    #[test]
    fn test_spec_content_with_metadata() {
        let content = SpecContent::new(
            SpecFormat::Markdown,
            "# Test".to_string(),
        ).with_metadata("author".to_string(), serde_json::Value::String("John Doe".to_string()));

        assert_eq!(content.metadata.len(), 1);
        assert_eq!(
            content.metadata.get("author").unwrap(),
            &serde_json::Value::String("John Doe".to_string())
        );
    }
}