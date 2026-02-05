use crate::models::context::*;
use crate::models::enhanced_context::*;
use chrono::{DateTime, Utc};
use serde_json::json;

/// Conversion utilities between legacy and enhanced context models
pub struct ContextConverter;

impl ContextConverter {
    /// Convert a BusinessRule to an EnhancedContextItem
    pub fn from_business_rule(rule: BusinessRule) -> EnhancedContextItem {
        let content = ContextContent {
            content_type: ContextType::BusinessRule,
            title: rule.rule_name.clone(),
            description: rule.description.unwrap_or_default(),
            data: json!({
                "domain_area": rule.domain_area,
                "implementation_pattern": rule.implementation_pattern,
                "constraints": rule.constraints,
                "examples": rule.examples
            }),
            source_file: None,
            source_line: None,
        };

        let mut item = EnhancedContextItem::new(rule.project_id, content);
        item.id = rule.id;
        
        // Parse created_at if available
        if let Some(created_at_str) = rule.created_at {
            if let Ok(created_at) = DateTime::parse_from_rfc3339(&created_at_str) {
                item.created_at = created_at.with_timezone(&Utc);
                item.updated_at = created_at.with_timezone(&Utc);
            }
        }

        // Add semantic tags based on domain area
        if let Some(domain) = &rule.domain_area {
            item.semantic_tags.push(SemanticTag::new(
                format!("domain:{}", domain),
                0.9,
                TagSource::Manual,
            ));
        }

        item.metadata.tags.push("business_rule".to_string());
        item
    }

    /// Convert an ArchitecturalDecision to an EnhancedContextItem
    pub fn from_architectural_decision(decision: ArchitecturalDecision) -> EnhancedContextItem {
        let content = ContextContent {
            content_type: ContextType::ArchitecturalDecision,
            title: decision.decision_title.clone(),
            description: decision.context.unwrap_or_default(),
            data: json!({
                "decision": decision.decision,
                "consequences": decision.consequences,
                "alternatives_considered": decision.alternatives_considered,
                "status": decision.status
            }),
            source_file: None,
            source_line: None,
        };

        let mut item = EnhancedContextItem::new(decision.project_id, content);
        item.id = decision.id;

        // Parse created_at if available
        if let Some(created_at_str) = decision.created_at {
            if let Ok(created_at) = DateTime::parse_from_rfc3339(&created_at_str) {
                item.created_at = created_at.with_timezone(&Utc);
                item.updated_at = created_at.with_timezone(&Utc);
            }
        }

        // Set priority based on status
        if let Some(status) = &decision.status {
            item.metadata.priority = match status.to_lowercase().as_str() {
                "accepted" => Priority::High,
                "proposed" => Priority::Medium,
                "deprecated" => Priority::Low,
                _ => Priority::Medium,
            };
        }

        item.metadata.tags.push("architectural_decision".to_string());
        item.semantic_tags.push(SemanticTag::new(
            "architecture".to_string(),
            0.95,
            TagSource::Manual,
        ));

        item
    }

    /// Convert a PerformanceRequirement to an EnhancedContextItem
    pub fn from_performance_requirement(req: PerformanceRequirement) -> EnhancedContextItem {
        let content = ContextContent {
            content_type: ContextType::PerformanceRequirement,
            title: format!("Performance: {}", req.component_area.as_deref().unwrap_or("General")),
            description: format!("Performance requirement for {}", req.component_area.as_deref().unwrap_or("system")),
            data: json!({
                "component_area": req.component_area,
                "requirement_type": req.requirement_type,
                "target_value": req.target_value,
                "optimization_patterns": req.optimization_patterns,
                "avoid_patterns": req.avoid_patterns
            }),
            source_file: None,
            source_line: None,
        };

        let mut item = EnhancedContextItem::new(req.project_id, content);
        item.id = req.id;

        // Parse created_at if available
        if let Some(created_at_str) = req.created_at {
            if let Ok(created_at) = DateTime::parse_from_rfc3339(&created_at_str) {
                item.created_at = created_at.with_timezone(&Utc);
                item.updated_at = created_at.with_timezone(&Utc);
            }
        }

        // Set high priority for performance requirements
        item.metadata.priority = Priority::High;
        item.metadata.tags.push("performance".to_string());
        
        if let Some(component) = &req.component_area {
            item.semantic_tags.push(SemanticTag::new(
                format!("component:{}", component),
                0.8,
                TagSource::Manual,
            ));
        }

        item
    }

    /// Convert a SecurityPolicy to an EnhancedContextItem
    pub fn from_security_policy(policy: SecurityPolicy) -> EnhancedContextItem {
        let content = ContextContent {
            content_type: ContextType::SecurityPolicy,
            title: policy.policy_name.clone(),
            description: policy.requirements.clone().unwrap_or_default(),
            data: json!({
                "policy_area": policy.policy_area,
                "requirements": policy.requirements,
                "implementation_pattern": policy.implementation_pattern,
                "forbidden_patterns": policy.forbidden_patterns,
                "compliance_notes": policy.compliance_notes
            }),
            source_file: None,
            source_line: None,
        };

        let mut item = EnhancedContextItem::new(policy.project_id, content);
        item.id = policy.id;

        // Parse created_at if available
        if let Some(created_at_str) = policy.created_at {
            if let Ok(created_at) = DateTime::parse_from_rfc3339(&created_at_str) {
                item.created_at = created_at.with_timezone(&Utc);
                item.updated_at = created_at.with_timezone(&Utc);
            }
        }

        // Security policies are critical
        item.metadata.priority = Priority::Critical;
        item.metadata.tags.push("security".to_string());
        
        if let Some(area) = &policy.policy_area {
            item.semantic_tags.push(SemanticTag::new(
                format!("security:{}", area),
                0.95,
                TagSource::Manual,
            ));
        }

        item
    }

    /// Convert a ProjectConvention to an EnhancedContextItem
    pub fn from_project_convention(convention: ProjectConvention) -> EnhancedContextItem {
        let content = ContextContent {
            content_type: ContextType::ProjectConvention,
            title: format!("Convention: {}", convention.convention_type.as_deref().unwrap_or("General")),
            description: convention.convention_rule.clone().unwrap_or_default(),
            data: json!({
                "convention_type": convention.convention_type,
                "convention_rule": convention.convention_rule,
                "good_examples": convention.good_examples,
                "bad_examples": convention.bad_examples,
                "rationale": convention.rationale
            }),
            source_file: None,
            source_line: None,
        };

        let mut item = EnhancedContextItem::new(convention.project_id, content);
        item.id = convention.id;

        // Parse created_at if available
        if let Some(created_at_str) = convention.created_at {
            if let Ok(created_at) = DateTime::parse_from_rfc3339(&created_at_str) {
                item.created_at = created_at.with_timezone(&Utc);
                item.updated_at = created_at.with_timezone(&Utc);
            }
        }

        item.metadata.tags.push("convention".to_string());
        
        if let Some(conv_type) = &convention.convention_type {
            item.semantic_tags.push(SemanticTag::new(
                format!("convention:{}", conv_type),
                0.85,
                TagSource::Manual,
            ));
        }

        item
    }

    /// Convert a FeatureContext to an EnhancedContextItem
    pub fn from_feature_context(feature: FeatureContext) -> EnhancedContextItem {
        let content = ContextContent {
            content_type: ContextType::FeatureContext,
            title: feature.feature_name.clone(),
            description: feature.business_purpose.clone().unwrap_or_default(),
            data: json!({
                "business_purpose": feature.business_purpose,
                "user_personas": feature.user_personas,
                "key_workflows": feature.key_workflows,
                "integration_points": feature.integration_points,
                "edge_cases": feature.edge_cases
            }),
            source_file: None,
            source_line: None,
        };

        let mut item = EnhancedContextItem::new(feature.project_id, content);
        item.id = feature.id;

        // Parse created_at if available
        if let Some(created_at_str) = feature.created_at {
            if let Ok(created_at) = DateTime::parse_from_rfc3339(&created_at_str) {
                item.created_at = created_at.with_timezone(&Utc);
                item.updated_at = created_at.with_timezone(&Utc);
            }
        }

        item.metadata.tags.push("feature".to_string());
        item.semantic_tags.push(SemanticTag::new(
            format!("feature:{}", feature.feature_name),
            0.9,
            TagSource::Manual,
        ));

        item
    }

    /// Convert an EnhancedContextItem back to a BusinessRule (if applicable)
    pub fn to_business_rule(item: &EnhancedContextItem) -> Option<BusinessRule> {
        if item.content.content_type != ContextType::BusinessRule {
            return None;
        }

        let data = &item.content.data;
        Some(BusinessRule {
            id: item.id.clone(),
            project_id: item.project_id.clone(),
            rule_name: item.content.title.clone(),
            description: Some(item.content.description.clone()),
            domain_area: data.get("domain_area").and_then(|v| v.as_str()).map(String::from),
            implementation_pattern: data.get("implementation_pattern").and_then(|v| v.as_str()).map(String::from),
            constraints: data.get("constraints").and_then(|v| v.as_str()).map(String::from),
            examples: data.get("examples").and_then(|v| v.as_str()).map(String::from),
            created_at: Some(item.created_at.to_rfc3339()),
        })
    }

    /// Convert an EnhancedContextItem back to an ArchitecturalDecision (if applicable)
    pub fn to_architectural_decision(item: &EnhancedContextItem) -> Option<ArchitecturalDecision> {
        if item.content.content_type != ContextType::ArchitecturalDecision {
            return None;
        }

        let data = &item.content.data;
        Some(ArchitecturalDecision {
            id: item.id.clone(),
            project_id: item.project_id.clone(),
            decision_title: item.content.title.clone(),
            context: Some(item.content.description.clone()),
            decision: data.get("decision").and_then(|v| v.as_str()).map(String::from),
            consequences: data.get("consequences").and_then(|v| v.as_str()).map(String::from),
            alternatives_considered: data.get("alternatives_considered").and_then(|v| v.as_str()).map(String::from),
            status: data.get("status").and_then(|v| v.as_str()).map(String::from),
            created_at: Some(item.created_at.to_rfc3339()),
        })
    }

    /// Convert an EnhancedContextItem back to a PerformanceRequirement (if applicable)
    pub fn to_performance_requirement(item: &EnhancedContextItem) -> Option<PerformanceRequirement> {
        if item.content.content_type != ContextType::PerformanceRequirement {
            return None;
        }

        let data = &item.content.data;
        Some(PerformanceRequirement {
            id: item.id.clone(),
            project_id: item.project_id.clone(),
            component_area: data.get("component_area").and_then(|v| v.as_str()).map(String::from),
            requirement_type: data.get("requirement_type").and_then(|v| v.as_str()).map(String::from),
            target_value: data.get("target_value").and_then(|v| v.as_str()).map(String::from),
            optimization_patterns: data.get("optimization_patterns").and_then(|v| v.as_str()).map(String::from),
            avoid_patterns: data.get("avoid_patterns").and_then(|v| v.as_str()).map(String::from),
            created_at: Some(item.created_at.to_rfc3339()),
        })
    }

    /// Convert an EnhancedContextItem back to a SecurityPolicy (if applicable)
    pub fn to_security_policy(item: &EnhancedContextItem) -> Option<SecurityPolicy> {
        if item.content.content_type != ContextType::SecurityPolicy {
            return None;
        }

        let data = &item.content.data;
        Some(SecurityPolicy {
            id: item.id.clone(),
            project_id: item.project_id.clone(),
            policy_name: item.content.title.clone(),
            policy_area: data.get("policy_area").and_then(|v| v.as_str()).map(String::from),
            requirements: Some(item.content.description.clone()),
            implementation_pattern: data.get("implementation_pattern").and_then(|v| v.as_str()).map(String::from),
            forbidden_patterns: data.get("forbidden_patterns").and_then(|v| v.as_str()).map(String::from),
            compliance_notes: data.get("compliance_notes").and_then(|v| v.as_str()).map(String::from),
            created_at: Some(item.created_at.to_rfc3339()),
        })
    }

    /// Convert an EnhancedContextItem back to a ProjectConvention (if applicable)
    pub fn to_project_convention(item: &EnhancedContextItem) -> Option<ProjectConvention> {
        if item.content.content_type != ContextType::ProjectConvention {
            return None;
        }

        let data = &item.content.data;
        Some(ProjectConvention {
            id: item.id.clone(),
            project_id: item.project_id.clone(),
            convention_type: data.get("convention_type").and_then(|v| v.as_str()).map(String::from),
            convention_rule: Some(item.content.description.clone()),
            good_examples: data.get("good_examples").and_then(|v| v.as_str()).map(String::from),
            bad_examples: data.get("bad_examples").and_then(|v| v.as_str()).map(String::from),
            rationale: data.get("rationale").and_then(|v| v.as_str()).map(String::from),
            created_at: Some(item.created_at.to_rfc3339()),
        })
    }

    /// Convert an EnhancedContextItem back to a FeatureContext (if applicable)
    pub fn to_feature_context(item: &EnhancedContextItem) -> Option<FeatureContext> {
        if item.content.content_type != ContextType::FeatureContext {
            return None;
        }

        let data = &item.content.data;
        Some(FeatureContext {
            id: item.id.clone(),
            project_id: item.project_id.clone(),
            feature_name: item.content.title.clone(),
            business_purpose: Some(item.content.description.clone()),
            user_personas: data.get("user_personas").and_then(|v| v.as_str()).map(String::from),
            key_workflows: data.get("key_workflows").and_then(|v| v.as_str()).map(String::from),
            integration_points: data.get("integration_points").and_then(|v| v.as_str()).map(String::from),
            edge_cases: data.get("edge_cases").and_then(|v| v.as_str()).map(String::from),
            created_at: Some(item.created_at.to_rfc3339()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_business_rule_conversion() {
        let rule = BusinessRule {
            id: "test-id".to_string(),
            project_id: "project-1".to_string(),
            rule_name: "Test Rule".to_string(),
            description: Some("Test description".to_string()),
            domain_area: Some("auth".to_string()),
            implementation_pattern: Some("pattern".to_string()),
            constraints: Some("constraints".to_string()),
            examples: Some("examples".to_string()),
            created_at: Some(Utc::now().to_rfc3339()),
        };

        let enhanced = ContextConverter::from_business_rule(rule.clone());
        assert_eq!(enhanced.id, rule.id);
        assert_eq!(enhanced.project_id, rule.project_id);
        assert_eq!(enhanced.content.title, rule.rule_name);
        assert_eq!(enhanced.content.content_type, ContextType::BusinessRule);

        let converted_back = ContextConverter::to_business_rule(&enhanced).unwrap();
        assert_eq!(converted_back.id, rule.id);
        assert_eq!(converted_back.rule_name, rule.rule_name);
    }

    #[test]
    fn test_architectural_decision_conversion() {
        let decision = ArchitecturalDecision {
            id: "test-id".to_string(),
            project_id: "project-1".to_string(),
            decision_title: "Test Decision".to_string(),
            context: Some("Test context".to_string()),
            decision: Some("Use microservices".to_string()),
            consequences: Some("Better scalability".to_string()),
            alternatives_considered: Some("Monolith".to_string()),
            status: Some("accepted".to_string()),
            created_at: Some(Utc::now().to_rfc3339()),
        };

        let enhanced = ContextConverter::from_architectural_decision(decision.clone());
        assert_eq!(enhanced.content.content_type, ContextType::ArchitecturalDecision);
        assert_eq!(enhanced.metadata.priority, Priority::High);

        let converted_back = ContextConverter::to_architectural_decision(&enhanced).unwrap();
        assert_eq!(converted_back.decision_title, decision.decision_title);
    }
}