use crate::services::conflict_resolution_engine::{ConflictInfo, ConflictType, ManualResolutionRequest, ConflictResolutionResult};
use crate::services::websocket_types::{ConflictStrategy, ClientId};
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

/// UI service for conflict resolution workflows
#[derive(Clone)]
pub struct ConflictResolutionUI {
    /// Active UI sessions for conflict resolution
    active_sessions: HashMap<String, ConflictResolutionSession>,
}

/// A conflict resolution session with UI state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolutionSession {
    pub session_id: String,
    pub conflict_id: String,
    pub user_id: String,
    pub client_id: ClientId,
    pub conflict_info: ConflictInfo,
    pub ui_state: ConflictUIState,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub timeout_at: Option<DateTime<Utc>>,
}

/// UI state for conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictUIState {
    pub current_step: ConflictResolutionStep,
    pub selected_strategy: Option<ConflictStrategy>,
    pub user_selections: HashMap<String, serde_json::Value>,
    pub preview_entity: Option<serde_json::Value>,
    pub validation_errors: Vec<ValidationError>,
    pub progress: ConflictResolutionProgress,
}

/// Steps in the conflict resolution workflow
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictResolutionStep {
    /// Initial conflict presentation
    ConflictPresentation,
    /// Strategy selection
    StrategySelection,
    /// Manual resolution (if manual strategy selected)
    ManualResolution,
    /// Preview and confirmation
    PreviewConfirmation,
    /// Resolution complete
    Complete,
    /// Resolution cancelled
    Cancelled,
}

/// Progress tracking for conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolutionProgress {
    pub total_steps: u32,
    pub current_step: u32,
    pub completed_steps: Vec<ConflictResolutionStep>,
    pub estimated_time_remaining: Option<u32>, // seconds
}

/// Validation error in conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub severity: ValidationSeverity,
}

/// Severity of validation errors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

/// Request to start a conflict resolution session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartResolutionRequest {
    pub conflict_id: String,
    pub user_id: String,
    pub client_id: ClientId,
    pub preferred_strategy: Option<ConflictStrategy>,
    pub timeout_seconds: Option<u64>,
}

/// Response when starting a conflict resolution session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartResolutionResponse {
    pub session_id: String,
    pub conflict_info: ConflictInfo,
    pub available_strategies: Vec<ConflictStrategyOption>,
    pub recommended_strategy: ConflictStrategy,
    pub ui_components: Vec<UIComponent>,
}

/// Available conflict resolution strategy with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictStrategyOption {
    pub strategy: ConflictStrategy,
    pub name: String,
    pub description: String,
    pub complexity: StrategyComplexity,
    pub estimated_time: u32, // seconds
    pub success_rate: f64,
    pub requires_manual_input: bool,
}

/// Complexity level of resolution strategies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StrategyComplexity {
    Simple,
    Moderate,
    Complex,
}

/// UI component for conflict resolution interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIComponent {
    pub component_type: UIComponentType,
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub data: serde_json::Value,
    pub validation_rules: Vec<ValidationRule>,
    pub is_required: bool,
}

/// Types of UI components
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UIComponentType {
    ConflictOverview,
    StrategySelector,
    EntityEditor,
    FieldMerger,
    DiffViewer,
    PreviewPanel,
    ActionButtons,
}

/// Validation rule for UI components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_type: ValidationType,
    pub message: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Types of validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationType {
    Required,
    MinLength,
    MaxLength,
    Pattern,
    Custom(String),
}

/// Request to update UI state during resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUIStateRequest {
    pub session_id: String,
    pub step: ConflictResolutionStep,
    pub user_selections: HashMap<String, serde_json::Value>,
    pub selected_strategy: Option<ConflictStrategy>,
}

/// Response to UI state update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUIStateResponse {
    pub success: bool,
    pub updated_ui_state: ConflictUIState,
    pub next_components: Vec<UIComponent>,
    pub validation_errors: Vec<ValidationError>,
    pub can_proceed: bool,
}

impl ConflictResolutionUI {
    /// Create a new conflict resolution UI service
    pub fn new() -> Self {
        Self {
            active_sessions: HashMap::new(),
        }
    }

    /// Start a new conflict resolution session
    pub async fn start_resolution_session(
        &mut self,
        request: StartResolutionRequest,
        conflict_info: ConflictInfo,
    ) -> Result<StartResolutionResponse> {
        let session_id = Uuid::new_v4().to_string();
        
        debug!("Starting conflict resolution session {} for conflict {}", session_id, request.conflict_id);

        // Create initial UI state
        let ui_state = ConflictUIState {
            current_step: ConflictResolutionStep::ConflictPresentation,
            selected_strategy: request.preferred_strategy.clone(),
            user_selections: HashMap::new(),
            preview_entity: None,
            validation_errors: Vec::new(),
            progress: ConflictResolutionProgress {
                total_steps: 4,
                current_step: 1,
                completed_steps: Vec::new(),
                estimated_time_remaining: Some(300), // 5 minutes default
            },
        };

        // Calculate timeout
        let timeout_at = request.timeout_seconds.map(|seconds| {
            Utc::now() + chrono::Duration::seconds(seconds as i64)
        });

        // Create session
        let session = ConflictResolutionSession {
            session_id: session_id.clone(),
            conflict_id: request.conflict_id.clone(),
            user_id: request.user_id,
            client_id: request.client_id,
            conflict_info: conflict_info.clone(),
            ui_state,
            created_at: Utc::now(),
            last_activity: Utc::now(),
            timeout_at,
        };

        // Store session
        self.active_sessions.insert(session_id.clone(), session);

        // Generate response
        let available_strategies = self.get_available_strategies(&conflict_info);
        let recommended_strategy = self.recommend_strategy(&conflict_info);
        let ui_components = self.generate_initial_ui_components(&conflict_info);

        Ok(StartResolutionResponse {
            session_id,
            conflict_info,
            available_strategies,
            recommended_strategy,
            ui_components,
        })
    }

    /// Update UI state during conflict resolution
    pub async fn update_ui_state(
        &mut self,
        request: UpdateUIStateRequest,
    ) -> Result<UpdateUIStateResponse> {
        // First, get the session data we need for validation and component generation
        let conflict_info = {
            let session = self.active_sessions
                .get(&request.session_id)
                .ok_or_else(|| anyhow!("Session not found: {}", request.session_id))?;
            session.conflict_info.clone()
        };

        debug!("Updating UI state for session {} to step {:?}", request.session_id, request.step);

        // Now get mutable access to update the session
        let session = self.active_sessions
            .get_mut(&request.session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", request.session_id))?;

        // Update session activity
        session.last_activity = Utc::now();

        // Update UI state
        session.ui_state.current_step = request.step.clone();
        session.ui_state.user_selections.extend(request.user_selections);
        
        if let Some(strategy) = request.selected_strategy {
            session.ui_state.selected_strategy = Some(strategy);
        }

        // Update progress
        Self::update_progress_static(&mut session.ui_state, &request.step);

        // Validate current state
        let validation_errors = Self::validate_ui_state_static(&session.ui_state, &conflict_info)?;
        session.ui_state.validation_errors = validation_errors.clone();

        // Generate next UI components
        let next_components = Self::generate_ui_components_for_step_static(
            &request.step,
            &conflict_info,
            &session.ui_state,
        )?;

        // Check if can proceed
        let can_proceed = validation_errors.iter().all(|e| e.severity != ValidationSeverity::Error);

        // Generate preview if in preview step
        if request.step == ConflictResolutionStep::PreviewConfirmation {
            session.ui_state.preview_entity = Some(Self::generate_preview_entity_static(&session.ui_state, &conflict_info)?);
        }

        Ok(UpdateUIStateResponse {
            success: true,
            updated_ui_state: session.ui_state.clone(),
            next_components,
            validation_errors,
            can_proceed,
        })
    }

    /// Complete conflict resolution and generate manual resolution request
    pub async fn complete_resolution(
        &mut self,
        session_id: &str,
        resolution_notes: Option<String>,
    ) -> Result<ManualResolutionRequest> {
        let session = self.active_sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;

        info!("Completing conflict resolution session {}", session_id);

        // Validate final state
        let validation_errors = Self::validate_ui_state_static(&session.ui_state, &session.conflict_info)?;
        if validation_errors.iter().any(|e| e.severity == ValidationSeverity::Error) {
            return Err(anyhow!("Cannot complete resolution with validation errors"));
        }

        // Get selected strategy
        let strategy = session.ui_state.selected_strategy
            .clone()
            .unwrap_or(ConflictStrategy::LastWriterWins);

        // Generate resolved entity based on user selections and strategy
        let resolved_entity = match strategy {
            ConflictStrategy::ManualResolution => {
                session.ui_state.preview_entity.clone()
            }
            ConflictStrategy::LastWriterWins => {
                // For LastWriterWins, generate the resolved entity from the latest change
                Self::generate_preview_entity_static(&session.ui_state, &session.conflict_info).ok()
            }
            ConflictStrategy::AutoMerge => {
                // For AutoMerge, the engine will handle the merging
                None
            }
            ConflictStrategy::Reject => {
                // For Reject, no resolved entity needed
                None
            }
        };

        // Update session state
        session.ui_state.current_step = ConflictResolutionStep::Complete;

        // Create manual resolution request
        let manual_request = ManualResolutionRequest {
            conflict_id: session.conflict_id.clone(),
            resolution_strategy: strategy,
            resolved_entity,
            resolution_notes,
            resolved_by: session.user_id.clone(),
        };

        Ok(manual_request)
    }

    /// Cancel a conflict resolution session
    pub async fn cancel_resolution(&mut self, session_id: &str) -> Result<()> {
        if let Some(session) = self.active_sessions.get_mut(session_id) {
            session.ui_state.current_step = ConflictResolutionStep::Cancelled;
            info!("Cancelled conflict resolution session {}", session_id);
        }
        Ok(())
    }

    /// Get active session information
    pub fn get_session(&self, session_id: &str) -> Option<&ConflictResolutionSession> {
        self.active_sessions.get(session_id)
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&mut self) {
        let now = Utc::now();
        self.active_sessions.retain(|_, session| {
            if let Some(timeout_at) = session.timeout_at {
                timeout_at > now
            } else {
                true
            }
        });
    }

    /// Get available strategies for a conflict
    fn get_available_strategies(&self, conflict_info: &ConflictInfo) -> Vec<ConflictStrategyOption> {
        let mut strategies = vec![
            ConflictStrategyOption {
                strategy: ConflictStrategy::LastWriterWins,
                name: "Last Writer Wins".to_string(),
                description: "Accept the most recent change and discard others".to_string(),
                complexity: StrategyComplexity::Simple,
                estimated_time: 30,
                success_rate: 0.9,
                requires_manual_input: false,
            },
            ConflictStrategyOption {
                strategy: ConflictStrategy::AutoMerge,
                name: "Automatic Merge".to_string(),
                description: "Automatically merge non-conflicting changes".to_string(),
                complexity: StrategyComplexity::Moderate,
                estimated_time: 60,
                success_rate: 0.75,
                requires_manual_input: false,
            },
            ConflictStrategyOption {
                strategy: ConflictStrategy::Reject,
                name: "Reject All Changes".to_string(),
                description: "Reject all conflicting changes and keep original".to_string(),
                complexity: StrategyComplexity::Simple,
                estimated_time: 15,
                success_rate: 1.0,
                requires_manual_input: false,
            },
        ];

        // Add manual resolution for complex conflicts
        if conflict_info.conflicting_changes.len() > 1 || conflict_info.conflict_type == ConflictType::SemanticConflict {
            strategies.push(ConflictStrategyOption {
                strategy: ConflictStrategy::ManualResolution,
                name: "Manual Resolution".to_string(),
                description: "Manually review and resolve conflicts".to_string(),
                complexity: StrategyComplexity::Complex,
                estimated_time: 300,
                success_rate: 0.95,
                requires_manual_input: true,
            });
        }

        strategies
    }

    /// Recommend the best strategy for a conflict
    fn recommend_strategy(&self, conflict_info: &ConflictInfo) -> ConflictStrategy {
        match conflict_info.conflict_type {
            ConflictType::VersionConflict => ConflictStrategy::LastWriterWins,
            ConflictType::ContentConflict => {
                if conflict_info.conflicting_changes.len() <= 2 {
                    ConflictStrategy::AutoMerge
                } else {
                    ConflictStrategy::ManualResolution
                }
            }
            ConflictType::SemanticConflict | ConflictType::DependencyConflict => {
                ConflictStrategy::ManualResolution
            }
        }
    }

    /// Generate initial UI components for conflict presentation
    fn generate_initial_ui_components(&self, conflict_info: &ConflictInfo) -> Vec<UIComponent> {
        vec![
            UIComponent {
                component_type: UIComponentType::ConflictOverview,
                id: "conflict-overview".to_string(),
                title: "Conflict Overview".to_string(),
                description: Some("Review the conflicting changes".to_string()),
                data: serde_json::to_value(conflict_info).unwrap_or_default(),
                validation_rules: Vec::new(),
                is_required: false,
            },
            UIComponent {
                component_type: UIComponentType::DiffViewer,
                id: "diff-viewer".to_string(),
                title: "Change Differences".to_string(),
                description: Some("Compare the conflicting changes".to_string()),
                data: serde_json::json!({
                    "changes": conflict_info.conflicting_changes
                }),
                validation_rules: Vec::new(),
                is_required: false,
            },
            UIComponent {
                component_type: UIComponentType::ActionButtons,
                id: "action-buttons".to_string(),
                title: "Actions".to_string(),
                description: None,
                data: serde_json::json!({
                    "actions": ["proceed", "cancel"]
                }),
                validation_rules: Vec::new(),
                is_required: false,
            },
        ]
    }

    /// Generate UI components for a specific resolution step
    fn generate_ui_components_for_step(
        &self,
        step: &ConflictResolutionStep,
        conflict_info: &ConflictInfo,
        ui_state: &ConflictUIState,
    ) -> Result<Vec<UIComponent>> {
        match step {
            ConflictResolutionStep::StrategySelection => {
                Ok(vec![
                    UIComponent {
                        component_type: UIComponentType::StrategySelector,
                        id: "strategy-selector".to_string(),
                        title: "Resolution Strategy".to_string(),
                        description: Some("Choose how to resolve this conflict".to_string()),
                        data: serde_json::json!({
                            "strategies": self.get_available_strategies(conflict_info),
                            "recommended": self.recommend_strategy(conflict_info)
                        }),
                        validation_rules: vec![
                            ValidationRule {
                                rule_type: ValidationType::Required,
                                message: "Please select a resolution strategy".to_string(),
                                parameters: HashMap::new(),
                            }
                        ],
                        is_required: true,
                    }
                ])
            }
            ConflictResolutionStep::ManualResolution => {
                Ok(vec![
                    UIComponent {
                        component_type: UIComponentType::EntityEditor,
                        id: "entity-editor".to_string(),
                        title: "Manual Resolution".to_string(),
                        description: Some("Edit the entity to resolve conflicts".to_string()),
                        data: serde_json::json!({
                            "conflicting_changes": conflict_info.conflicting_changes,
                            "current_selections": ui_state.user_selections
                        }),
                        validation_rules: vec![
                            ValidationRule {
                                rule_type: ValidationType::Required,
                                message: "Please provide a resolved entity".to_string(),
                                parameters: HashMap::new(),
                            }
                        ],
                        is_required: true,
                    },
                    UIComponent {
                        component_type: UIComponentType::FieldMerger,
                        id: "field-merger".to_string(),
                        title: "Field-by-Field Merge".to_string(),
                        description: Some("Choose values for each conflicting field".to_string()),
                        data: serde_json::json!({
                            "conflicting_fields": self.extract_conflicting_fields(&conflict_info.conflicting_changes)
                        }),
                        validation_rules: Vec::new(),
                        is_required: false,
                    }
                ])
            }
            ConflictResolutionStep::PreviewConfirmation => {
                Ok(vec![
                    UIComponent {
                        component_type: UIComponentType::PreviewPanel,
                        id: "preview-panel".to_string(),
                        title: "Resolution Preview".to_string(),
                        description: Some("Review the resolved entity before applying".to_string()),
                        data: serde_json::json!({
                            "preview_entity": ui_state.preview_entity,
                            "strategy": ui_state.selected_strategy,
                            "discarded_changes": self.calculate_discarded_changes(ui_state, conflict_info)
                        }),
                        validation_rules: Vec::new(),
                        is_required: false,
                    }
                ])
            }
            _ => Ok(Vec::new()),
        }
    }

    /// Validate UI state for errors
    fn validate_ui_state(&self, ui_state: &ConflictUIState, _conflict_info: &ConflictInfo) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate strategy selection
        if ui_state.current_step == ConflictResolutionStep::StrategySelection && ui_state.selected_strategy.is_none() {
            errors.push(ValidationError {
                field: "selected_strategy".to_string(),
                message: "Please select a resolution strategy".to_string(),
                severity: ValidationSeverity::Error,
            });
        }

        // Validate manual resolution
        if ui_state.current_step == ConflictResolutionStep::ManualResolution {
            if ui_state.selected_strategy == Some(ConflictStrategy::ManualResolution) && ui_state.preview_entity.is_none() {
                errors.push(ValidationError {
                    field: "resolved_entity".to_string(),
                    message: "Please provide a resolved entity for manual resolution".to_string(),
                    severity: ValidationSeverity::Error,
                });
            }
        }

        Ok(errors)
    }

    /// Validate UI state for errors (static version)
    fn validate_ui_state_static(ui_state: &ConflictUIState, _conflict_info: &ConflictInfo) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate strategy selection
        if ui_state.current_step == ConflictResolutionStep::StrategySelection && ui_state.selected_strategy.is_none() {
            errors.push(ValidationError {
                field: "selected_strategy".to_string(),
                message: "Please select a resolution strategy".to_string(),
                severity: ValidationSeverity::Error,
            });
        }

        // Validate manual resolution
        if ui_state.current_step == ConflictResolutionStep::ManualResolution {
            if ui_state.selected_strategy == Some(ConflictStrategy::ManualResolution) {
                // Check if we have either a resolved entity or field selections
                let has_resolved_entity = ui_state.user_selections.contains_key("resolved_entity");
                let has_field_selections = ui_state.user_selections.keys().any(|k| k.starts_with("field_"));
                
                if !has_resolved_entity && !has_field_selections {
                    errors.push(ValidationError {
                        field: "resolved_entity".to_string(),
                        message: "Please provide a resolved entity for manual resolution".to_string(),
                        severity: ValidationSeverity::Error,
                    });
                }
            }
        }

        Ok(errors)
    }

    /// Generate UI components for a specific resolution step (static version)
    fn generate_ui_components_for_step_static(
        step: &ConflictResolutionStep,
        conflict_info: &ConflictInfo,
        ui_state: &ConflictUIState,
    ) -> Result<Vec<UIComponent>> {
        match step {
            ConflictResolutionStep::StrategySelection => {
                let available_strategies = Self::get_available_strategies_static(conflict_info);
                let recommended_strategy = Self::recommend_strategy_static(conflict_info);
                
                Ok(vec![
                    UIComponent {
                        component_type: UIComponentType::StrategySelector,
                        id: "strategy-selector".to_string(),
                        title: "Resolution Strategy".to_string(),
                        description: Some("Choose how to resolve this conflict".to_string()),
                        data: serde_json::json!({
                            "strategies": available_strategies,
                            "recommended": recommended_strategy
                        }),
                        validation_rules: vec![
                            ValidationRule {
                                rule_type: ValidationType::Required,
                                message: "Please select a resolution strategy".to_string(),
                                parameters: HashMap::new(),
                            }
                        ],
                        is_required: true,
                    }
                ])
            }
            ConflictResolutionStep::ManualResolution => {
                Ok(vec![
                    UIComponent {
                        component_type: UIComponentType::EntityEditor,
                        id: "entity-editor".to_string(),
                        title: "Manual Resolution".to_string(),
                        description: Some("Edit the entity to resolve conflicts".to_string()),
                        data: serde_json::json!({
                            "conflicting_changes": conflict_info.conflicting_changes,
                            "current_selections": ui_state.user_selections
                        }),
                        validation_rules: vec![
                            ValidationRule {
                                rule_type: ValidationType::Required,
                                message: "Please provide a resolved entity".to_string(),
                                parameters: HashMap::new(),
                            }
                        ],
                        is_required: true,
                    },
                    UIComponent {
                        component_type: UIComponentType::FieldMerger,
                        id: "field-merger".to_string(),
                        title: "Field-by-Field Merge".to_string(),
                        description: Some("Choose values for each conflicting field".to_string()),
                        data: serde_json::json!({
                            "conflicting_fields": Self::extract_conflicting_fields_static(&conflict_info.conflicting_changes)
                        }),
                        validation_rules: Vec::new(),
                        is_required: false,
                    }
                ])
            }
            ConflictResolutionStep::PreviewConfirmation => {
                Ok(vec![
                    UIComponent {
                        component_type: UIComponentType::PreviewPanel,
                        id: "preview-panel".to_string(),
                        title: "Resolution Preview".to_string(),
                        description: Some("Review the resolved entity before applying".to_string()),
                        data: serde_json::json!({
                            "preview_entity": ui_state.preview_entity,
                            "strategy": ui_state.selected_strategy,
                            "discarded_changes": Self::calculate_discarded_changes_static(ui_state, conflict_info)
                        }),
                        validation_rules: Vec::new(),
                        is_required: false,
                    }
                ])
            }
            _ => Ok(Vec::new()),
        }
    }

    /// Generate preview entity based on user selections (static version)
    fn generate_preview_entity_static(ui_state: &ConflictUIState, conflict_info: &ConflictInfo) -> Result<serde_json::Value> {
        match ui_state.selected_strategy {
            Some(ConflictStrategy::LastWriterWins) => {
                // Find the most recent change
                let latest_change = conflict_info.conflicting_changes
                    .iter()
                    .max_by_key(|c| c.change.metadata.timestamp)
                    .ok_or_else(|| anyhow!("No conflicting changes found"))?;
                
                Ok(latest_change.change.full_entity.clone().unwrap_or_default())
            }
            Some(ConflictStrategy::ManualResolution) => {
                // Use user selections to build entity
                if let Some(resolved_entity) = ui_state.user_selections.get("resolved_entity") {
                    Ok(resolved_entity.clone())
                } else {
                    // Build from field selections
                    let mut merged_entity = serde_json::Map::new();
                    for (key, value) in &ui_state.user_selections {
                        if key.starts_with("field_") {
                            let field_name = key.strip_prefix("field_").unwrap();
                            merged_entity.insert(field_name.to_string(), value.clone());
                        }
                    }
                    Ok(serde_json::Value::Object(merged_entity))
                }
            }
            _ => {
                // For other strategies, return a placeholder
                Ok(serde_json::json!({
                    "preview": "Entity will be resolved automatically"
                }))
            }
        }
    }

    /// Get available strategies for a conflict (static version)
    fn get_available_strategies_static(conflict_info: &ConflictInfo) -> Vec<ConflictStrategyOption> {
        let mut strategies = vec![
            ConflictStrategyOption {
                strategy: ConflictStrategy::LastWriterWins,
                name: "Last Writer Wins".to_string(),
                description: "Accept the most recent change and discard others".to_string(),
                complexity: StrategyComplexity::Simple,
                estimated_time: 30,
                success_rate: 0.9,
                requires_manual_input: false,
            },
            ConflictStrategyOption {
                strategy: ConflictStrategy::AutoMerge,
                name: "Automatic Merge".to_string(),
                description: "Automatically merge non-conflicting changes".to_string(),
                complexity: StrategyComplexity::Moderate,
                estimated_time: 60,
                success_rate: 0.75,
                requires_manual_input: false,
            },
            ConflictStrategyOption {
                strategy: ConflictStrategy::Reject,
                name: "Reject All Changes".to_string(),
                description: "Reject all conflicting changes and keep original".to_string(),
                complexity: StrategyComplexity::Simple,
                estimated_time: 15,
                success_rate: 1.0,
                requires_manual_input: false,
            },
        ];

        // Add manual resolution for complex conflicts
        if conflict_info.conflicting_changes.len() > 1 || conflict_info.conflict_type == ConflictType::SemanticConflict {
            strategies.push(ConflictStrategyOption {
                strategy: ConflictStrategy::ManualResolution,
                name: "Manual Resolution".to_string(),
                description: "Manually review and resolve conflicts".to_string(),
                complexity: StrategyComplexity::Complex,
                estimated_time: 300,
                success_rate: 0.95,
                requires_manual_input: true,
            });
        }

        strategies
    }

    /// Recommend the best strategy for a conflict (static version)
    fn recommend_strategy_static(conflict_info: &ConflictInfo) -> ConflictStrategy {
        match conflict_info.conflict_type {
            ConflictType::VersionConflict => ConflictStrategy::LastWriterWins,
            ConflictType::ContentConflict => {
                if conflict_info.conflicting_changes.len() <= 2 {
                    ConflictStrategy::AutoMerge
                } else {
                    ConflictStrategy::ManualResolution
                }
            }
            ConflictType::SemanticConflict | ConflictType::DependencyConflict => {
                ConflictStrategy::ManualResolution
            }
        }
    }

    /// Extract conflicting fields from changes (static version)
    fn extract_conflicting_fields_static(changes: &[crate::services::conflict_resolution_engine::ConflictingChange]) -> serde_json::Value {
        let mut conflicting_fields = serde_json::Map::new();
        
        for change in changes {
            if let Some(entity) = &change.change.full_entity {
                if let serde_json::Value::Object(obj) = entity {
                    for (key, value) in obj {
                        conflicting_fields.entry(key.clone())
                            .or_insert_with(|| serde_json::json!([]))
                            .as_array_mut()
                            .unwrap()
                            .push(serde_json::json!({
                                "value": value,
                                "change_id": change.change_id,
                                "timestamp": change.change.metadata.timestamp,
                                "client_id": change.change.metadata.client_id
                            }));
                    }
                }
            }
        }
        
        serde_json::Value::Object(conflicting_fields)
    }

    /// Calculate which changes will be discarded (static version)
    fn calculate_discarded_changes_static(ui_state: &ConflictUIState, conflict_info: &ConflictInfo) -> Vec<Uuid> {
        match ui_state.selected_strategy {
            Some(ConflictStrategy::LastWriterWins) => {
                let latest_change = conflict_info.conflicting_changes
                    .iter()
                    .max_by_key(|c| c.change.metadata.timestamp);
                
                if let Some(latest) = latest_change {
                    conflict_info.conflicting_changes
                        .iter()
                        .filter(|c| c.change_id != latest.change_id)
                        .map(|c| c.change_id)
                        .collect()
                } else {
                    Vec::new()
                }
            }
            Some(ConflictStrategy::Reject) => {
                conflict_info.conflicting_changes
                    .iter()
                    .map(|c| c.change_id)
                    .collect()
            }
            _ => Vec::new(), // AutoMerge and ManualResolution don't discard changes
        }
    }

    /// Update progress tracking (static version)
    fn update_progress_static(ui_state: &mut ConflictUIState, current_step: &ConflictResolutionStep) {
        if !ui_state.progress.completed_steps.contains(current_step) {
            ui_state.progress.completed_steps.push(current_step.clone());
        }

        ui_state.progress.current_step = match current_step {
            ConflictResolutionStep::ConflictPresentation => 1,
            ConflictResolutionStep::StrategySelection => 2,
            ConflictResolutionStep::ManualResolution => 3,
            ConflictResolutionStep::PreviewConfirmation => 4,
            ConflictResolutionStep::Complete => 4,
            ConflictResolutionStep::Cancelled => 0,
        };

        // Update estimated time remaining
        ui_state.progress.estimated_time_remaining = match current_step {
            ConflictResolutionStep::ConflictPresentation => Some(240),
            ConflictResolutionStep::StrategySelection => Some(180),
            ConflictResolutionStep::ManualResolution => Some(120),
            ConflictResolutionStep::PreviewConfirmation => Some(30),
            _ => None,
        };
    }

    /// Generate preview entity based on user selections
    fn generate_preview_entity(&self, ui_state: &ConflictUIState, conflict_info: &ConflictInfo) -> Result<serde_json::Value> {
        match ui_state.selected_strategy {
            Some(ConflictStrategy::LastWriterWins) => {
                // Find the most recent change
                let latest_change = conflict_info.conflicting_changes
                    .iter()
                    .max_by_key(|c| c.change.metadata.timestamp)
                    .ok_or_else(|| anyhow!("No conflicting changes found"))?;
                
                Ok(latest_change.change.full_entity.clone().unwrap_or_default())
            }
            Some(ConflictStrategy::ManualResolution) => {
                // Use user selections to build entity
                if let Some(resolved_entity) = ui_state.user_selections.get("resolved_entity") {
                    Ok(resolved_entity.clone())
                } else {
                    // Build from field selections
                    let mut merged_entity = serde_json::Map::new();
                    for (key, value) in &ui_state.user_selections {
                        if key.starts_with("field_") {
                            let field_name = key.strip_prefix("field_").unwrap();
                            merged_entity.insert(field_name.to_string(), value.clone());
                        }
                    }
                    Ok(serde_json::Value::Object(merged_entity))
                }
            }
            _ => {
                // For other strategies, return a placeholder
                Ok(serde_json::json!({
                    "preview": "Entity will be resolved automatically"
                }))
            }
        }
    }

    /// Extract conflicting fields from changes
    fn extract_conflicting_fields(&self, changes: &[crate::services::conflict_resolution_engine::ConflictingChange]) -> serde_json::Value {
        let mut conflicting_fields = serde_json::Map::new();
        
        for change in changes {
            if let Some(entity) = &change.change.full_entity {
                if let serde_json::Value::Object(obj) = entity {
                    for (key, value) in obj {
                        conflicting_fields.entry(key.clone())
                            .or_insert_with(|| serde_json::json!([]))
                            .as_array_mut()
                            .unwrap()
                            .push(serde_json::json!({
                                "value": value,
                                "change_id": change.change_id,
                                "timestamp": change.change.metadata.timestamp,
                                "client_id": change.change.metadata.client_id
                            }));
                    }
                }
            }
        }
        
        serde_json::Value::Object(conflicting_fields)
    }

    /// Calculate which changes will be discarded
    fn calculate_discarded_changes(&self, ui_state: &ConflictUIState, conflict_info: &ConflictInfo) -> Vec<Uuid> {
        match ui_state.selected_strategy {
            Some(ConflictStrategy::LastWriterWins) => {
                let latest_change = conflict_info.conflicting_changes
                    .iter()
                    .max_by_key(|c| c.change.metadata.timestamp);
                
                if let Some(latest) = latest_change {
                    conflict_info.conflicting_changes
                        .iter()
                        .filter(|c| c.change_id != latest.change_id)
                        .map(|c| c.change_id)
                        .collect()
                } else {
                    Vec::new()
                }
            }
            Some(ConflictStrategy::Reject) => {
                conflict_info.conflicting_changes
                    .iter()
                    .map(|c| c.change_id)
                    .collect()
            }
            _ => Vec::new(), // AutoMerge and ManualResolution don't discard changes
        }
    }
}

impl Default for ConflictResolutionUI {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::conflict_resolution_engine::{ConflictingChange, ClientInfo};
    use crate::services::websocket_types::{ContextChange, ChangeType, ChangeMetadata};
    use serde_json::json;

    fn create_test_conflict_info() -> ConflictInfo {
        let client1 = Uuid::new_v4();
        let client2 = Uuid::new_v4();
        let now = Utc::now();

        ConflictInfo {
            conflict_id: Uuid::new_v4().to_string(),
            entity_type: "business_rule".to_string(),
            entity_id: "rule-1".to_string(),
            project_id: "test-project".to_string(),
            conflicting_changes: vec![
                ConflictingChange {
                    change_id: Uuid::new_v4(),
                    change: ContextChange {
                        change_id: Uuid::new_v4(),
                        change_type: ChangeType::Update,
                        entity_type: "business_rule".to_string(),
                        entity_id: "rule-1".to_string(),
                        project_id: "test-project".to_string(),
                        feature_area: Some("auth".to_string()),
                        delta: None,
                        full_entity: Some(json!({
                            "id": "rule-1",
                            "name": "Rule from Client 1",
                            "description": "Original description"
                        })),
                        metadata: ChangeMetadata {
                            user_id: Some("user1".to_string()),
                            client_id: client1,
                            timestamp: now,
                            version: 1,
                            conflict_resolution: None,
                        },
                    },
                    base_version: 1,
                    client_info: ClientInfo {
                        client_id: client1,
                        user_id: Some("user1".to_string()),
                        client_type: "test".to_string(),
                        timestamp: now,
                    },
                },
                ConflictingChange {
                    change_id: Uuid::new_v4(),
                    change: ContextChange {
                        change_id: Uuid::new_v4(),
                        change_type: ChangeType::Update,
                        entity_type: "business_rule".to_string(),
                        entity_id: "rule-1".to_string(),
                        project_id: "test-project".to_string(),
                        feature_area: Some("auth".to_string()),
                        delta: None,
                        full_entity: Some(json!({
                            "id": "rule-1",
                            "name": "Rule from Client 2",
                            "priority": "high"
                        })),
                        metadata: ChangeMetadata {
                            user_id: Some("user2".to_string()),
                            client_id: client2,
                            timestamp: now + chrono::Duration::seconds(10),
                            version: 1,
                            conflict_resolution: None,
                        },
                    },
                    base_version: 1,
                    client_info: ClientInfo {
                        client_id: client2,
                        user_id: Some("user2".to_string()),
                        client_type: "test".to_string(),
                        timestamp: now + chrono::Duration::seconds(10),
                    },
                },
            ],
            conflict_type: ConflictType::ContentConflict,
            detected_at: now,
            resolution_strategy: None,
            resolved_at: None,
            resolved_by: None,
            resolution_result: None,
        }
    }

    #[tokio::test]
    async fn test_start_resolution_session() {
        let mut ui = ConflictResolutionUI::new();
        let conflict_info = create_test_conflict_info();
        
        let request = StartResolutionRequest {
            conflict_id: conflict_info.conflict_id.clone(),
            user_id: "test-user".to_string(),
            client_id: Uuid::new_v4(),
            preferred_strategy: Some(ConflictStrategy::AutoMerge),
            timeout_seconds: Some(600),
        };

        let response = ui.start_resolution_session(request, conflict_info).await.unwrap();
        
        assert!(!response.session_id.is_empty());
        assert_eq!(response.recommended_strategy, ConflictStrategy::AutoMerge);
        assert!(!response.available_strategies.is_empty());
        assert!(!response.ui_components.is_empty());
    }

    #[tokio::test]
    async fn test_update_ui_state() {
        let mut ui = ConflictResolutionUI::new();
        let conflict_info = create_test_conflict_info();
        
        let start_request = StartResolutionRequest {
            conflict_id: conflict_info.conflict_id.clone(),
            user_id: "test-user".to_string(),
            client_id: Uuid::new_v4(),
            preferred_strategy: None,
            timeout_seconds: Some(600),
        };

        let start_response = ui.start_resolution_session(start_request, conflict_info).await.unwrap();
        
        let mut user_selections = HashMap::new();
        user_selections.insert("strategy".to_string(), json!("LastWriterWins"));
        
        let update_request = UpdateUIStateRequest {
            session_id: start_response.session_id.clone(),
            step: ConflictResolutionStep::StrategySelection,
            user_selections,
            selected_strategy: Some(ConflictStrategy::LastWriterWins),
        };

        let update_response = ui.update_ui_state(update_request).await.unwrap();
        
        assert!(update_response.success);
        assert_eq!(update_response.updated_ui_state.current_step, ConflictResolutionStep::StrategySelection);
        assert_eq!(update_response.updated_ui_state.selected_strategy, Some(ConflictStrategy::LastWriterWins));
    }

    #[tokio::test]
    async fn test_complete_resolution() {
        let mut ui = ConflictResolutionUI::new();
        let conflict_info = create_test_conflict_info();
        
        let start_request = StartResolutionRequest {
            conflict_id: conflict_info.conflict_id.clone(),
            user_id: "test-user".to_string(),
            client_id: Uuid::new_v4(),
            preferred_strategy: Some(ConflictStrategy::LastWriterWins),
            timeout_seconds: Some(600),
        };

        let start_response = ui.start_resolution_session(start_request, conflict_info).await.unwrap();
        
        // Update to strategy selection
        let update_request = UpdateUIStateRequest {
            session_id: start_response.session_id.clone(),
            step: ConflictResolutionStep::StrategySelection,
            user_selections: HashMap::new(),
            selected_strategy: Some(ConflictStrategy::LastWriterWins),
        };
        ui.update_ui_state(update_request).await.unwrap();

        let manual_request = ui.complete_resolution(
            &start_response.session_id,
            Some("Resolved using last writer wins".to_string()),
        ).await.unwrap();
        
        assert_eq!(manual_request.resolution_strategy, ConflictStrategy::LastWriterWins);
        assert_eq!(manual_request.resolved_by, "test-user");
        assert!(manual_request.resolution_notes.is_some());
    }

    #[tokio::test]
    async fn test_strategy_recommendation() {
        let ui = ConflictResolutionUI::new();
        
        // Test version conflict recommendation
        let mut version_conflict = create_test_conflict_info();
        version_conflict.conflict_type = ConflictType::VersionConflict;
        assert_eq!(ui.recommend_strategy(&version_conflict), ConflictStrategy::LastWriterWins);
        
        // Test semantic conflict recommendation
        let mut semantic_conflict = create_test_conflict_info();
        semantic_conflict.conflict_type = ConflictType::SemanticConflict;
        assert_eq!(ui.recommend_strategy(&semantic_conflict), ConflictStrategy::ManualResolution);
        
        // Test simple content conflict recommendation
        let mut simple_content_conflict = create_test_conflict_info();
        simple_content_conflict.conflict_type = ConflictType::ContentConflict;
        simple_content_conflict.conflicting_changes.truncate(2);
        assert_eq!(ui.recommend_strategy(&simple_content_conflict), ConflictStrategy::AutoMerge);
    }

    #[tokio::test]
    async fn test_validation() {
        let ui = ConflictResolutionUI::new();
        let conflict_info = create_test_conflict_info();
        
        // Test validation with missing strategy
        let ui_state = ConflictUIState {
            current_step: ConflictResolutionStep::StrategySelection,
            selected_strategy: None,
            user_selections: HashMap::new(),
            preview_entity: None,
            validation_errors: Vec::new(),
            progress: ConflictResolutionProgress {
                total_steps: 4,
                current_step: 2,
                completed_steps: Vec::new(),
                estimated_time_remaining: Some(180),
            },
        };
        
        let errors = ui.validate_ui_state(&ui_state, &conflict_info).unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].severity, ValidationSeverity::Error);
        assert_eq!(errors[0].field, "selected_strategy");
    }
}