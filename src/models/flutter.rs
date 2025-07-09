// Flutter-specific context models
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlutterComponent {
    pub id: String,
    pub project_id: String,
    pub component_name: String,
    pub component_type: ComponentType,
    pub architecture_layer: ArchitectureLayer,
    pub file_path: Option<String>,
    pub dependencies: Vec<String>,
    pub riverpod_scope: Option<RiverpodScope>,
    pub widget_type: Option<WidgetType>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentType {
    Widget,
    Provider,
    Service,
    Repository,
    Model,
    Utility,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchitectureLayer {
    Presentation,
    Domain,
    Data,
    Core,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiverpodScope {
    Global,
    Scoped,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    StatelessWidget,
    StatefulWidget,
    ConsumerWidget,
    ConsumerStatefulWidget,
    HookWidget,
    HookConsumerWidget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentPhase {
    pub id: String,
    pub project_id: String,
    pub phase_name: String,
    pub phase_order: i32,
    pub status: PhaseStatus,
    pub description: Option<String>,
    pub completion_criteria: Vec<String>,
    pub dependencies: Vec<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhaseStatus {
    Pending,
    InProgress,
    Completed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyRule {
    pub id: String,
    pub project_id: String,
    pub rule_name: String,
    pub rule_type: PrivacyRuleType,
    pub pattern: String,
    pub description: Option<String>,
    pub severity: Severity,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyRuleType {
    ForbiddenImport,
    RequiredLocalStorage,
    DataFlow,
    NetworkAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyViolation {
    pub id: String,
    pub project_id: String,
    pub rule_id: String,
    pub file_path: String,
    pub line_number: Option<i32>,
    pub violation_text: Option<String>,
    pub status: ViolationStatus,
    pub detected_at: Option<String>,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationStatus {
    Open,
    Resolved,
    Suppressed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureLayerConfig {
    pub id: String,
    pub project_id: String,
    pub layer_name: String,
    pub allowed_dependencies: Vec<String>,
    pub forbidden_imports: Vec<String>,
    pub description: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelContext {
    pub id: String,
    pub project_id: String,
    pub model_name: String,
    pub model_path: Option<String>,
    pub model_type: Option<String>,
    pub model_size: Option<String>,
    pub performance_metrics: Option<PerformanceMetrics>,
    pub configuration: Option<ModelConfiguration>,
    pub is_active: bool,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub inference_time_ms: Option<f64>,
    pub memory_usage_mb: Option<f64>,
    pub tokens_per_second: Option<f64>,
    pub cpu_usage_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfiguration {
    pub context_length: Option<i32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<i32>,
    pub threads: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeTemplate {
    pub id: String,
    pub project_id: String,
    pub template_name: String,
    pub template_type: TemplateType,
    pub template_content: String,
    pub variables: Vec<String>,
    pub description: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateType {
    Widget,
    Provider,
    Repository,
    Test,
    Service,
    Model,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilitiesInfo {
    pub server_info: ServerMetadata,
    pub features: Vec<FeatureInfo>,
    pub database_tables: Vec<TableInfo>,
    pub mcp_tools: Vec<ToolInfo>,
    pub usage_examples: Vec<UsageExample>,
    pub recommended_workflow: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub config_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureInfo {
    pub name: String,
    pub description: String,
    pub status: FeatureStatus,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureStatus {
    Implemented,
    Framework,    // Database structure exists, tools being added
    Planned,      // Not yet implemented
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub description: String,
    pub primary_fields: Vec<String>,
    pub example_use: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub category: String,
    pub required_params: Vec<String>,
    pub example_use: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageExample {
    pub scenario: String,
    pub steps: Vec<String>,
}

// Implement Display traits for enum serialization
impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentType::Widget => write!(f, "widget"),
            ComponentType::Provider => write!(f, "provider"),
            ComponentType::Service => write!(f, "service"),
            ComponentType::Repository => write!(f, "repository"),
            ComponentType::Model => write!(f, "model"),
            ComponentType::Utility => write!(f, "utility"),
        }
    }
}

impl std::fmt::Display for ArchitectureLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchitectureLayer::Presentation => write!(f, "presentation"),
            ArchitectureLayer::Domain => write!(f, "domain"),
            ArchitectureLayer::Data => write!(f, "data"),
            ArchitectureLayer::Core => write!(f, "core"),
        }
    }
}

impl std::fmt::Display for PhaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PhaseStatus::Pending => write!(f, "pending"),
            PhaseStatus::InProgress => write!(f, "in_progress"),
            PhaseStatus::Completed => write!(f, "completed"),
            PhaseStatus::Blocked => write!(f, "blocked"),
        }
    }
}

impl std::fmt::Display for FeatureStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeatureStatus::Implemented => write!(f, "implemented"),
            FeatureStatus::Framework => write!(f, "framework"),
            FeatureStatus::Planned => write!(f, "planned"),
        }
    }
}
