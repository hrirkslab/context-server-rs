use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Type aliases for better readability
pub type SpecId = String;
pub type RequirementId = String;
pub type TaskId = String;
pub type ProjectId = String;
pub type ContextId = String;

/// Project specification containing requirements, design, and tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSpecification {
    pub id: SpecId,
    pub project_id: ProjectId,
    pub spec_type: SpecType,
    pub title: String,
    pub description: Option<String>,
    pub content: SpecContent,
    pub requirements: Vec<RequirementId>,
    pub tasks: Vec<TaskId>,
    pub status: SpecStatus,
    pub version: u32,
    pub file_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: SpecMetadata,
}

impl ProjectSpecification {
    pub fn new(
        project_id: ProjectId,
        spec_type: SpecType,
        title: String,
        content: SpecContent,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            project_id,
            spec_type,
            title,
            description: None,
            content,
            requirements: Vec::new(),
            tasks: Vec::new(),
            status: SpecStatus::Draft,
            version: 1,
            file_path: None,
            created_at: now,
            updated_at: now,
            metadata: SpecMetadata::default(),
        }
    }

    pub fn update_content(&mut self, content: SpecContent) {
        self.content = content;
        self.updated_at = Utc::now();
        self.version += 1;
    }

    pub fn add_requirement(&mut self, requirement_id: RequirementId) {
        if !self.requirements.contains(&requirement_id) {
            self.requirements.push(requirement_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn add_task(&mut self, task_id: TaskId) {
        if !self.tasks.contains(&task_id) {
            self.tasks.push(task_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn update_status(&mut self, status: SpecStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }
}

/// Types of specifications
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SpecType {
    Feature,
    Architecture,
    API,
    Database,
    Security,
    Performance,
    Requirements,
    Design,
    Tasks,
    Custom(String),
}

impl SpecType {
    pub fn as_str(&self) -> &str {
        match self {
            SpecType::Feature => "feature",
            SpecType::Architecture => "architecture",
            SpecType::API => "api",
            SpecType::Database => "database",
            SpecType::Security => "security",
            SpecType::Performance => "performance",
            SpecType::Requirements => "requirements",
            SpecType::Design => "design",
            SpecType::Tasks => "tasks",
            SpecType::Custom(name) => name,
        }
    }

    pub fn from_filename(filename: &str) -> Self {
        match filename.to_lowercase().as_str() {
            "requirements.md" => SpecType::Requirements,
            "design.md" => SpecType::Design,
            "tasks.md" => SpecType::Tasks,
            _ if filename.contains("api") => SpecType::API,
            _ if filename.contains("architecture") => SpecType::Architecture,
            _ if filename.contains("security") => SpecType::Security,
            _ if filename.contains("performance") => SpecType::Performance,
            _ if filename.contains("database") => SpecType::Database,
            _ => SpecType::Custom(filename.to_string()),
        }
    }
}

/// Content of a specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecContent {
    pub format: SpecFormat,
    pub raw_content: String,
    pub parsed_sections: HashMap<String, String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl SpecContent {
    pub fn new(format: SpecFormat, raw_content: String) -> Self {
        Self {
            format,
            raw_content,
            parsed_sections: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_sections(mut self, sections: HashMap<String, String>) -> Self {
        self.parsed_sections = sections;
        self
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Format of specification content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SpecFormat {
    Markdown,
    YAML,
    JSON,
    PlainText,
    Custom(String),
}

impl SpecFormat {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "md" | "markdown" => SpecFormat::Markdown,
            "yaml" | "yml" => SpecFormat::YAML,
            "json" => SpecFormat::JSON,
            "txt" => SpecFormat::PlainText,
            _ => SpecFormat::Custom(ext.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            SpecFormat::Markdown => "markdown",
            SpecFormat::YAML => "yaml",
            SpecFormat::JSON => "json",
            SpecFormat::PlainText => "plaintext",
            SpecFormat::Custom(name) => name,
        }
    }
}

/// Status of a specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SpecStatus {
    Draft,
    InReview,
    Approved,
    InProgress,
    Completed,
    Archived,
    Deprecated,
}

impl SpecStatus {
    pub fn as_str(&self) -> &str {
        match self {
            SpecStatus::Draft => "draft",
            SpecStatus::InReview => "in_review",
            SpecStatus::Approved => "approved",
            SpecStatus::InProgress => "in_progress",
            SpecStatus::Completed => "completed",
            SpecStatus::Archived => "archived",
            SpecStatus::Deprecated => "deprecated",
        }
    }
}

/// Metadata for specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecMetadata {
    pub tags: Vec<String>,
    pub priority: Priority,
    pub complexity: Complexity,
    pub estimated_effort: Option<String>,
    pub actual_effort: Option<String>,
    pub stakeholders: Vec<String>,
    pub dependencies: Vec<SpecId>,
    pub linked_context: Vec<ContextId>,
    pub custom_fields: HashMap<String, serde_json::Value>,
}

impl Default for SpecMetadata {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            priority: Priority::Medium,
            complexity: Complexity::Medium,
            estimated_effort: None,
            actual_effort: None,
            stakeholders: Vec::new(),
            dependencies: Vec::new(),
            linked_context: Vec::new(),
            custom_fields: HashMap::new(),
        }
    }
}

/// Priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

impl Priority {
    pub fn as_str(&self) -> &str {
        match self {
            Priority::Critical => "critical",
            Priority::High => "high",
            Priority::Medium => "medium",
            Priority::Low => "low",
        }
    }

    pub fn score(&self) -> f64 {
        match self {
            Priority::Critical => 1.0,
            Priority::High => 0.8,
            Priority::Medium => 0.6,
            Priority::Low => 0.4,
        }
    }
}

/// Complexity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Complexity {
    Simple,
    Medium,
    Complex,
    VeryComplex,
}

impl Complexity {
    pub fn as_str(&self) -> &str {
        match self {
            Complexity::Simple => "simple",
            Complexity::Medium => "medium",
            Complexity::Complex => "complex",
            Complexity::VeryComplex => "very_complex",
        }
    }

    pub fn score(&self) -> f64 {
        match self {
            Complexity::Simple => 0.25,
            Complexity::Medium => 0.5,
            Complexity::Complex => 0.75,
            Complexity::VeryComplex => 1.0,
        }
    }
}

/// Individual requirement within a specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: RequirementId,
    pub spec_id: SpecId,
    pub title: String,
    pub description: String,
    pub user_story: Option<String>,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub priority: Priority,
    pub status: RequirementStatus,
    pub linked_tasks: Vec<TaskId>,
    pub linked_context: Vec<ContextId>,
    pub dependencies: Vec<RequirementId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: RequirementMetadata,
}

impl Requirement {
    pub fn new(spec_id: SpecId, title: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            spec_id,
            title,
            description,
            user_story: None,
            acceptance_criteria: Vec::new(),
            priority: Priority::Medium,
            status: RequirementStatus::Draft,
            linked_tasks: Vec::new(),
            linked_context: Vec::new(),
            dependencies: Vec::new(),
            created_at: now,
            updated_at: now,
            metadata: RequirementMetadata::default(),
        }
    }

    pub fn with_user_story(mut self, user_story: String) -> Self {
        self.user_story = Some(user_story);
        self
    }

    pub fn add_acceptance_criterion(&mut self, criterion: AcceptanceCriterion) {
        self.acceptance_criteria.push(criterion);
        self.updated_at = Utc::now();
    }

    pub fn link_task(&mut self, task_id: TaskId) {
        if !self.linked_tasks.contains(&task_id) {
            self.linked_tasks.push(task_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn link_context(&mut self, context_id: ContextId) {
        if !self.linked_context.contains(&context_id) {
            self.linked_context.push(context_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn update_status(&mut self, status: RequirementStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }
}

/// Acceptance criterion for requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub id: String,
    pub description: String,
    pub criterion_type: CriterionType,
    pub status: CriterionStatus,
    pub test_cases: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl AcceptanceCriterion {
    pub fn new(description: String, criterion_type: CriterionType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            description,
            criterion_type,
            status: CriterionStatus::Pending,
            test_cases: Vec::new(),
            created_at: Utc::now(),
        }
    }
}

/// Types of acceptance criteria
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CriterionType {
    Functional,
    NonFunctional,
    Performance,
    Security,
    Usability,
    Accessibility,
    Custom(String),
}

/// Status of acceptance criteria
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CriterionStatus {
    Pending,
    InProgress,
    Satisfied,
    Failed,
    NotApplicable,
}

/// Status of requirements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RequirementStatus {
    Draft,
    Defined,
    InProgress,
    Implemented,
    Tested,
    Accepted,
    Rejected,
    Deferred,
}

impl RequirementStatus {
    pub fn as_str(&self) -> &str {
        match self {
            RequirementStatus::Draft => "draft",
            RequirementStatus::Defined => "defined",
            RequirementStatus::InProgress => "in_progress",
            RequirementStatus::Implemented => "implemented",
            RequirementStatus::Tested => "tested",
            RequirementStatus::Accepted => "accepted",
            RequirementStatus::Rejected => "rejected",
            RequirementStatus::Deferred => "deferred",
        }
    }
}

/// Metadata for requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementMetadata {
    pub source: String,
    pub rationale: Option<String>,
    pub assumptions: Vec<String>,
    pub constraints: Vec<String>,
    pub risks: Vec<String>,
    pub verification_method: Option<String>,
    pub custom_fields: HashMap<String, serde_json::Value>,
}

impl Default for RequirementMetadata {
    fn default() -> Self {
        Self {
            source: "manual".to_string(),
            rationale: None,
            assumptions: Vec::new(),
            constraints: Vec::new(),
            risks: Vec::new(),
            verification_method: None,
            custom_fields: HashMap::new(),
        }
    }
}

/// Individual task within a specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub spec_id: SpecId,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub task_type: TaskType,
    pub dependencies: Vec<TaskId>,
    pub subtasks: Vec<TaskId>,
    pub parent_task: Option<TaskId>,
    pub estimated_effort: Option<String>,
    pub actual_effort: Option<String>,
    pub assigned_to: Option<String>,
    pub linked_requirements: Vec<RequirementId>,
    pub linked_context: Vec<ContextId>,
    pub progress: f64, // 0.0 to 1.0
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: TaskMetadata,
}

impl Task {
    pub fn new(spec_id: SpecId, title: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            spec_id,
            title,
            description,
            status: TaskStatus::NotStarted,
            task_type: TaskType::Implementation,
            dependencies: Vec::new(),
            subtasks: Vec::new(),
            parent_task: None,
            estimated_effort: None,
            actual_effort: None,
            assigned_to: None,
            linked_requirements: Vec::new(),
            linked_context: Vec::new(),
            progress: 0.0,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
            metadata: TaskMetadata::default(),
        }
    }

    pub fn add_dependency(&mut self, task_id: TaskId) {
        if !self.dependencies.contains(&task_id) {
            self.dependencies.push(task_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn add_subtask(&mut self, task_id: TaskId) {
        if !self.subtasks.contains(&task_id) {
            self.subtasks.push(task_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn link_requirement(&mut self, requirement_id: RequirementId) {
        if !self.linked_requirements.contains(&requirement_id) {
            self.linked_requirements.push(requirement_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn link_context(&mut self, context_id: ContextId) {
        if !self.linked_context.contains(&context_id) {
            self.linked_context.push(context_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn update_status(&mut self, status: TaskStatus) {
        let now = Utc::now();
        
        match (&self.status, &status) {
            (TaskStatus::NotStarted, TaskStatus::InProgress) => {
                self.started_at = Some(now);
            }
            (_, TaskStatus::Completed) => {
                self.completed_at = Some(now);
                self.progress = 1.0;
            }
            _ => {}
        }
        
        self.status = status;
        self.updated_at = now;
    }

    pub fn update_progress(&mut self, progress: f64) {
        self.progress = progress.clamp(0.0, 1.0);
        self.updated_at = Utc::now();
        
        // Auto-update status based on progress
        if self.progress == 1.0 && self.status != TaskStatus::Completed {
            self.update_status(TaskStatus::Completed);
        } else if self.progress > 0.0 && self.status == TaskStatus::NotStarted {
            self.update_status(TaskStatus::InProgress);
        }
    }
}

/// Types of tasks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    Implementation,
    Testing,
    Documentation,
    Research,
    Design,
    Review,
    Deployment,
    Maintenance,
    Custom(String),
}

impl TaskType {
    pub fn as_str(&self) -> &str {
        match self {
            TaskType::Implementation => "implementation",
            TaskType::Testing => "testing",
            TaskType::Documentation => "documentation",
            TaskType::Research => "research",
            TaskType::Design => "design",
            TaskType::Review => "review",
            TaskType::Deployment => "deployment",
            TaskType::Maintenance => "maintenance",
            TaskType::Custom(name) => name,
        }
    }
}

/// Status of tasks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    NotStarted,
    InProgress,
    Completed,
    Blocked,
    OnHold,
    Cancelled,
    Deferred,
}

impl TaskStatus {
    pub fn as_str(&self) -> &str {
        match self {
            TaskStatus::NotStarted => "not_started",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Completed => "completed",
            TaskStatus::Blocked => "blocked",
            TaskStatus::OnHold => "on_hold",
            TaskStatus::Cancelled => "cancelled",
            TaskStatus::Deferred => "deferred",
        }
    }

    pub fn from_checkbox_status(status: &str) -> Self {
        match status.trim() {
            "[ ]" | "-" => TaskStatus::NotStarted,
            "[x]" | "x" => TaskStatus::Completed,
            "[-]" => TaskStatus::InProgress,
            _ => TaskStatus::NotStarted,
        }
    }
}

/// Metadata for tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetadata {
    pub tags: Vec<String>,
    pub complexity: Complexity,
    pub priority: Priority,
    pub category: Option<String>,
    pub notes: Vec<String>,
    pub attachments: Vec<String>,
    pub custom_fields: HashMap<String, serde_json::Value>,
}

impl Default for TaskMetadata {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            complexity: Complexity::Medium,
            priority: Priority::Medium,
            category: None,
            notes: Vec::new(),
            attachments: Vec::new(),
            custom_fields: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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