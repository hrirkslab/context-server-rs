use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Type aliases for better readability
pub type ContextId = String;
pub type ProjectId = String;
pub type RelationshipId = String;

/// Enhanced context item with intelligence features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedContextItem {
    pub id: ContextId,
    pub project_id: ProjectId,
    pub content: ContextContent,
    pub metadata: ContextMetadata,
    pub relationships: Vec<ContextRelationship>,
    pub quality_score: f64,
    pub usage_stats: UsageStatistics,
    pub semantic_tags: Vec<SemanticTag>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u32,
}

impl EnhancedContextItem {
    pub fn new(project_id: ProjectId, content: ContextContent) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            project_id,
            content,
            metadata: ContextMetadata::default(),
            relationships: Vec::new(),
            quality_score: 0.0,
            usage_stats: UsageStatistics::default(),
            semantic_tags: Vec::new(),
            created_at: now,
            updated_at: now,
            version: 1,
        }
    }

    pub fn update_content(&mut self, content: ContextContent) {
        self.content = content;
        self.updated_at = Utc::now();
        self.version += 1;
    }

    pub fn add_relationship(&mut self, relationship: ContextRelationship) {
        self.relationships.push(relationship);
        self.updated_at = Utc::now();
    }

    pub fn update_quality_score(&mut self, score: f64) {
        self.quality_score = score.clamp(0.0, 1.0);
        self.updated_at = Utc::now();
    }

    pub fn record_usage(&mut self) {
        self.usage_stats.record_access();
        self.metadata.last_accessed = Some(Utc::now());
        self.metadata.access_count += 1;
    }
}

/// Content of a context item with type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextContent {
    pub content_type: ContextType,
    pub title: String,
    pub description: String,
    pub data: serde_json::Value,
    pub source_file: Option<String>,
    pub source_line: Option<u32>,
}

/// Types of context that can be stored
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContextType {
    BusinessRule,
    ArchitecturalDecision,
    PerformanceRequirement,
    SecurityPolicy,
    ProjectConvention,
    FeatureContext,
    CodePattern,
    ApiSpecification,
    DatabaseSchema,
    TestCase,
    Documentation,
    Custom(String),
}

impl ContextType {
    pub fn as_str(&self) -> &str {
        match self {
            ContextType::BusinessRule => "business_rule",
            ContextType::ArchitecturalDecision => "architectural_decision",
            ContextType::PerformanceRequirement => "performance_requirement",
            ContextType::SecurityPolicy => "security_policy",
            ContextType::ProjectConvention => "project_convention",
            ContextType::FeatureContext => "feature_context",
            ContextType::CodePattern => "code_pattern",
            ContextType::ApiSpecification => "api_specification",
            ContextType::DatabaseSchema => "database_schema",
            ContextType::TestCase => "test_case",
            ContextType::Documentation => "documentation",
            ContextType::Custom(name) => name,
        }
    }
}

/// Relationship between context items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRelationship {
    pub id: RelationshipId,
    pub target_id: ContextId,
    pub relationship_type: RelationshipType,
    pub strength: f64,
    pub auto_detected: bool,
    pub confidence: f64,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl ContextRelationship {
    pub fn new(
        target_id: ContextId,
        relationship_type: RelationshipType,
        strength: f64,
        auto_detected: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            target_id,
            relationship_type,
            strength: strength.clamp(0.0, 1.0),
            auto_detected,
            confidence: if auto_detected { 0.7 } else { 1.0 },
            metadata: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Types of relationships between context items
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationshipType {
    DependsOn,
    Conflicts,
    Implements,
    Extends,
    References,
    Similar,
    Supersedes,
    Validates,
    Constrains,
    Enables,
    Custom(String),
}

impl RelationshipType {
    pub fn as_str(&self) -> &str {
        match self {
            RelationshipType::DependsOn => "depends_on",
            RelationshipType::Conflicts => "conflicts",
            RelationshipType::Implements => "implements",
            RelationshipType::Extends => "extends",
            RelationshipType::References => "references",
            RelationshipType::Similar => "similar",
            RelationshipType::Supersedes => "supersedes",
            RelationshipType::Validates => "validates",
            RelationshipType::Constrains => "constrains",
            RelationshipType::Enables => "enables",
            RelationshipType::Custom(name) => name,
        }
    }

    pub fn is_bidirectional(&self) -> bool {
        matches!(self, RelationshipType::Similar | RelationshipType::Conflicts)
    }
}

/// Metadata associated with context items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    pub tags: Vec<String>,
    pub priority: Priority,
    pub confidence: f64,
    pub source: ContextSource,
    pub validation_status: ValidationStatus,
    pub last_accessed: Option<DateTime<Utc>>,
    pub access_count: u64,
    pub custom_fields: HashMap<String, serde_json::Value>,
}

impl Default for ContextMetadata {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            priority: Priority::Medium,
            confidence: 1.0,
            source: ContextSource::Manual,
            validation_status: ValidationStatus::Pending,
            last_accessed: None,
            access_count: 0,
            custom_fields: HashMap::new(),
        }
    }
}

/// Priority levels for context items
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

/// Source of context information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContextSource {
    Manual,
    AutoDetected,
    CodeAnalysis,
    Documentation,
    Git,
    Plugin(String),
}

impl ContextSource {
    pub fn as_str(&self) -> &str {
        match self {
            ContextSource::Manual => "manual",
            ContextSource::AutoDetected => "auto_detected",
            ContextSource::CodeAnalysis => "code_analysis",
            ContextSource::Documentation => "documentation",
            ContextSource::Git => "git",
            ContextSource::Plugin(name) => name,
        }
    }
}

/// Validation status of context items
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationStatus {
    Pending,
    Valid,
    Invalid,
    NeedsReview,
    Outdated,
}

impl ValidationStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ValidationStatus::Pending => "pending",
            ValidationStatus::Valid => "valid",
            ValidationStatus::Invalid => "invalid",
            ValidationStatus::NeedsReview => "needs_review",
            ValidationStatus::Outdated => "outdated",
        }
    }
}

/// Usage statistics for context items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStatistics {
    pub total_queries: u64,
    pub successful_queries: u64,
    pub last_query: Option<DateTime<Utc>>,
    pub average_relevance_score: f64,
    pub query_patterns: Vec<QueryPattern>,
    pub usage_frequency: UsageFrequency,
}

impl Default for UsageStatistics {
    fn default() -> Self {
        Self {
            total_queries: 0,
            successful_queries: 0,
            last_query: None,
            average_relevance_score: 0.0,
            query_patterns: Vec::new(),
            usage_frequency: UsageFrequency::Never,
        }
    }
}

impl UsageStatistics {
    pub fn record_access(&mut self) {
        self.total_queries += 1;
        self.last_query = Some(Utc::now());
        self.update_frequency();
    }

    pub fn record_successful_query(&mut self, relevance_score: f64) {
        self.successful_queries += 1;
        self.update_average_relevance(relevance_score);
    }

    fn update_average_relevance(&mut self, new_score: f64) {
        if self.successful_queries == 1 {
            self.average_relevance_score = new_score;
        } else {
            let total_score = self.average_relevance_score * (self.successful_queries - 1) as f64;
            self.average_relevance_score = (total_score + new_score) / self.successful_queries as f64;
        }
    }

    fn update_frequency(&mut self) {
        // Simple frequency calculation based on total queries
        self.usage_frequency = match self.total_queries {
            0 => UsageFrequency::Never,
            1..=5 => UsageFrequency::Rare,
            6..=20 => UsageFrequency::Occasional,
            21..=50 => UsageFrequency::Regular,
            _ => UsageFrequency::Frequent,
        };
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_queries == 0 {
            0.0
        } else {
            self.successful_queries as f64 / self.total_queries as f64
        }
    }
}

/// Usage frequency categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UsageFrequency {
    Never,
    Rare,
    Occasional,
    Regular,
    Frequent,
}

/// Query patterns for usage analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPattern {
    pub pattern_type: String,
    pub frequency: u32,
    pub last_seen: DateTime<Utc>,
    pub context_keywords: Vec<String>,
}

/// Semantic tags for context categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticTag {
    pub tag: String,
    pub confidence: f64,
    pub source: TagSource,
    pub created_at: DateTime<Utc>,
}

impl SemanticTag {
    pub fn new(tag: String, confidence: f64, source: TagSource) -> Self {
        Self {
            tag,
            confidence: confidence.clamp(0.0, 1.0),
            source,
            created_at: Utc::now(),
        }
    }
}

/// Source of semantic tags
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TagSource {
    Manual,
    NLP,
    CodeAnalysis,
    PatternMatching,
    MachineLearning,
}

/// Quality metrics for context items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub completeness_score: f64,
    pub consistency_score: f64,
    pub accuracy_score: f64,
    pub relevance_score: f64,
    pub freshness_score: f64,
    pub overall_score: f64,
    pub calculated_at: DateTime<Utc>,
    pub issues: Vec<QualityIssue>,
    pub suggestions: Vec<QualityImprovement>,
}

impl QualityMetrics {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            completeness_score: 0.0,
            consistency_score: 0.0,
            accuracy_score: 0.0,
            relevance_score: 0.0,
            freshness_score: 0.0,
            overall_score: 0.0,
            calculated_at: now,
            issues: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn calculate_overall_score(&mut self) {
        let scores = [
            self.completeness_score,
            self.consistency_score,
            self.accuracy_score,
            self.relevance_score,
            self.freshness_score,
        ];
        
        self.overall_score = scores.iter().sum::<f64>() / scores.len() as f64;
        self.calculated_at = Utc::now();
    }
}

/// Quality issues identified in context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    pub issue_type: QualityIssueType,
    pub severity: IssueSeverity,
    pub description: String,
    pub field: Option<String>,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QualityIssueType {
    MissingInformation,
    InconsistentData,
    OutdatedContent,
    InvalidFormat,
    ConflictingRules,
    LowUsage,
    DuplicateContent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Quality improvement suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityImprovement {
    pub suggestion_type: ImprovementType,
    pub description: String,
    pub priority: Priority,
    pub estimated_impact: f64,
    pub action_required: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImprovementType {
    AddMissingInfo,
    UpdateContent,
    ResolveConflict,
    AddExamples,
    ImproveDescription,
    AddRelationships,
    UpdateTags,
    ArchiveUnused,
}