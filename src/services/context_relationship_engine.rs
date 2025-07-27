use crate::models::enhanced_context::*;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use anyhow::Result;

/// Engine for detecting and managing relationships between context items
#[async_trait]
pub trait ContextRelationshipEngine: Send + Sync {
    /// Detect relationships for a new context item
    async fn detect_relationships(&self, context: &EnhancedContextItem, existing_contexts: &[EnhancedContextItem]) -> Result<Vec<ContextRelationship>>;
    
    /// Build a relationship graph for a set of context items
    async fn build_relationship_graph(&self, contexts: &[EnhancedContextItem]) -> Result<RelationshipGraph>;
    
    /// Calculate relationship strength between two context items
    async fn calculate_relationship_strength(&self, source: &EnhancedContextItem, target: &EnhancedContextItem) -> Result<f64>;
    
    /// Find related contexts based on relationship traversal
    async fn find_related_contexts(&self, context_id: &ContextId, contexts: &[EnhancedContextItem], max_depth: usize) -> Result<Vec<RelatedContext>>;
    
    /// Update relationships when context is modified
    async fn update_relationships_on_change(&self, changed_context: &EnhancedContextItem, all_contexts: &[EnhancedContextItem]) -> Result<Vec<RelationshipUpdate>>;
}

/// Default implementation of the Context Relationship Engine
pub struct DefaultContextRelationshipEngine {
    similarity_threshold: f64,
    keyword_extractors: Vec<Box<dyn KeywordExtractor>>,
    relationship_detectors: Vec<Box<dyn RelationshipDetector>>,
}

impl DefaultContextRelationshipEngine {
    pub fn new() -> Self {
        Self {
            similarity_threshold: 0.3,
            keyword_extractors: vec![
                Box::new(SimpleKeywordExtractor::new()),
                Box::new(TechnicalTermExtractor::new()),
            ],
            relationship_detectors: vec![
                Box::new(ContentSimilarityDetector::new()),
                Box::new(KeywordOverlapDetector::new()),
                Box::new(TypeBasedDetector::new()),
                Box::new(ProjectStructureDetector::new()),
            ],
        }
    }

    pub fn with_similarity_threshold(mut self, threshold: f64) -> Self {
        self.similarity_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Extract keywords from context content
    fn extract_keywords(&self, context: &EnhancedContextItem) -> Vec<String> {
        let mut keywords = HashSet::new();
        
        for extractor in &self.keyword_extractors {
            let extracted = extractor.extract_keywords(context);
            keywords.extend(extracted);
        }
        
        keywords.into_iter().collect()
    }

    /// Calculate content similarity between two contexts
    fn calculate_content_similarity(&self, source: &EnhancedContextItem, target: &EnhancedContextItem) -> f64 {
        let source_keywords = self.extract_keywords(source);
        let target_keywords = self.extract_keywords(target);
        
        if source_keywords.is_empty() || target_keywords.is_empty() {
            return 0.0;
        }
        
        let source_set: HashSet<_> = source_keywords.iter().collect();
        let target_set: HashSet<_> = target_keywords.iter().collect();
        
        let intersection = source_set.intersection(&target_set).count();
        let union = source_set.union(&target_set).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
}

#[async_trait]
impl ContextRelationshipEngine for DefaultContextRelationshipEngine {
    async fn detect_relationships(&self, context: &EnhancedContextItem, existing_contexts: &[EnhancedContextItem]) -> Result<Vec<ContextRelationship>> {
        let mut relationships = Vec::new();
        
        for existing in existing_contexts {
            if existing.id == context.id {
                continue;
            }
            
            // Run all relationship detectors
            for detector in &self.relationship_detectors {
                if let Some(relationship) = detector.detect_relationship(context, existing) {
                    if relationship.strength >= self.similarity_threshold {
                        relationships.push(relationship);
                    }
                }
            }
        }
        
        // Remove duplicates and keep the strongest relationships
        let mut relationship_map: HashMap<String, ContextRelationship> = HashMap::new();
        
        for rel in relationships {
            let key = format!("{}:{}", rel.target_id, rel.relationship_type.as_str());
            
            if let Some(existing) = relationship_map.get(&key) {
                if rel.strength > existing.strength {
                    relationship_map.insert(key, rel);
                }
            } else {
                relationship_map.insert(key, rel);
            }
        }
        
        Ok(relationship_map.into_values().collect())
    }

    async fn build_relationship_graph(&self, contexts: &[EnhancedContextItem]) -> Result<RelationshipGraph> {
        let mut graph = RelationshipGraph::new();
        
        // Add all contexts as nodes
        for context in contexts {
            graph.add_node(context.id.clone(), context.clone());
        }
        
        // Detect relationships between all pairs
        for (i, source) in contexts.iter().enumerate() {
            let remaining_contexts: Vec<_> = contexts.iter().skip(i + 1).collect();
            let relationships = self.detect_relationships(source, &remaining_contexts.into_iter().cloned().collect::<Vec<_>>()).await?;
            
            for relationship in relationships {
                graph.add_edge(source.id.clone(), relationship.clone());
                
                // Add bidirectional relationships if applicable
                if relationship.relationship_type.is_bidirectional() {
                    let reverse_relationship = ContextRelationship::new(
                        source.id.clone(),
                        relationship.relationship_type.clone(),
                        relationship.strength,
                        relationship.auto_detected,
                    ).with_confidence(relationship.confidence);
                    
                    graph.add_edge(relationship.target_id.clone(), reverse_relationship);
                }
            }
        }
        
        Ok(graph)
    }

    async fn calculate_relationship_strength(&self, source: &EnhancedContextItem, target: &EnhancedContextItem) -> Result<f64> {
        let mut total_strength = 0.0;
        let mut detector_count = 0;
        
        for detector in &self.relationship_detectors {
            if let Some(relationship) = detector.detect_relationship(source, target) {
                total_strength += relationship.strength;
                detector_count += 1;
            }
        }
        
        if detector_count == 0 {
            Ok(0.0)
        } else {
            Ok(total_strength / detector_count as f64)
        }
    }

    async fn find_related_contexts(&self, context_id: &ContextId, contexts: &[EnhancedContextItem], max_depth: usize) -> Result<Vec<RelatedContext>> {
        let graph = self.build_relationship_graph(contexts).await?;
        Ok(graph.find_related(context_id, max_depth))
    }

    async fn update_relationships_on_change(&self, changed_context: &EnhancedContextItem, all_contexts: &[EnhancedContextItem]) -> Result<Vec<RelationshipUpdate>> {
        let mut updates = Vec::new();
        
        // Find new relationships
        let new_relationships = self.detect_relationships(changed_context, all_contexts).await?;
        
        for new_rel in new_relationships {
            updates.push(RelationshipUpdate::Add {
                source_id: changed_context.id.clone(),
                relationship: new_rel,
            });
        }
        
        // Check if existing relationships need updates
        for existing_rel in &changed_context.relationships {
            if let Some(target_context) = all_contexts.iter().find(|c| c.id == existing_rel.target_id) {
                let new_strength = self.calculate_relationship_strength(changed_context, target_context).await?;
                
                if (new_strength - existing_rel.strength).abs() > 0.1 {
                    updates.push(RelationshipUpdate::Update {
                        source_id: changed_context.id.clone(),
                        relationship_id: existing_rel.id.clone(),
                        new_strength,
                    });
                }
            }
        }
        
        Ok(updates)
    }
}

/// Graph structure for managing context relationships
#[derive(Debug, Clone)]
pub struct RelationshipGraph {
    nodes: HashMap<ContextId, EnhancedContextItem>,
    edges: HashMap<ContextId, Vec<ContextRelationship>>,
}

impl RelationshipGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, id: ContextId, context: EnhancedContextItem) {
        self.nodes.insert(id.clone(), context);
        self.edges.entry(id).or_insert_with(Vec::new);
    }

    pub fn add_edge(&mut self, source_id: ContextId, relationship: ContextRelationship) {
        self.edges.entry(source_id).or_insert_with(Vec::new).push(relationship);
    }

    pub fn get_node(&self, id: &ContextId) -> Option<&EnhancedContextItem> {
        self.nodes.get(id)
    }

    pub fn get_relationships(&self, id: &ContextId) -> Option<&Vec<ContextRelationship>> {
        self.edges.get(id)
    }

    pub fn find_related(&self, context_id: &ContextId, max_depth: usize) -> Vec<RelatedContext> {
        let mut visited = HashSet::new();
        let mut related = Vec::new();
        
        self.find_related_recursive(context_id, 0, max_depth, &mut visited, &mut related);
        
        // Sort by strength and depth
        related.sort_by(|a, b| {
            b.strength.partial_cmp(&a.strength)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.depth.cmp(&b.depth))
        });
        
        related
    }

    fn find_related_recursive(
        &self,
        context_id: &ContextId,
        current_depth: usize,
        max_depth: usize,
        visited: &mut HashSet<ContextId>,
        related: &mut Vec<RelatedContext>,
    ) {
        if current_depth >= max_depth || visited.contains(context_id) {
            return;
        }
        
        visited.insert(context_id.clone());
        
        if let Some(relationships) = self.get_relationships(context_id) {
            for relationship in relationships {
                if let Some(target_context) = self.get_node(&relationship.target_id) {
                    related.push(RelatedContext {
                        context: target_context.clone(),
                        relationship_type: relationship.relationship_type.clone(),
                        strength: relationship.strength,
                        depth: current_depth + 1,
                        path: vec![context_id.clone(), relationship.target_id.clone()],
                    });
                    
                    // Recursively find related contexts
                    self.find_related_recursive(
                        &relationship.target_id,
                        current_depth + 1,
                        max_depth,
                        visited,
                        related,
                    );
                }
            }
        }
    }
}

/// Related context with relationship information
#[derive(Debug, Clone)]
pub struct RelatedContext {
    pub context: EnhancedContextItem,
    pub relationship_type: RelationshipType,
    pub strength: f64,
    pub depth: usize,
    pub path: Vec<ContextId>,
}

/// Updates to relationships
#[derive(Debug, Clone)]
pub enum RelationshipUpdate {
    Add {
        source_id: ContextId,
        relationship: ContextRelationship,
    },
    Update {
        source_id: ContextId,
        relationship_id: RelationshipId,
        new_strength: f64,
    },
    Remove {
        source_id: ContextId,
        relationship_id: RelationshipId,
    },
}

/// Trait for extracting keywords from context
trait KeywordExtractor: Send + Sync {
    fn extract_keywords(&self, context: &EnhancedContextItem) -> Vec<String>;
}

/// Simple keyword extractor based on common words
struct SimpleKeywordExtractor;

impl SimpleKeywordExtractor {
    fn new() -> Self {
        Self
    }
}

impl KeywordExtractor for SimpleKeywordExtractor {
    fn extract_keywords(&self, context: &EnhancedContextItem) -> Vec<String> {
        let text = format!("{} {}", context.content.title, context.content.description);
        let words: Vec<String> = text
            .to_lowercase()
            .split_whitespace()
            .filter(|word| word.len() > 3)
            .filter(|word| !is_stop_word(word))
            .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|word| !word.is_empty())
            .collect();
        
        words
    }
}

/// Technical term extractor for programming-related keywords
struct TechnicalTermExtractor;

impl TechnicalTermExtractor {
    fn new() -> Self {
        Self
    }
}

impl KeywordExtractor for TechnicalTermExtractor {
    fn extract_keywords(&self, context: &EnhancedContextItem) -> Vec<String> {
        let text = format!("{} {}", context.content.title, context.content.description);
        let mut keywords = Vec::new();
        
        // Extract technical terms (camelCase, snake_case, etc.)
        let technical_pattern = regex::Regex::new(r"[A-Z][a-z]+(?:[A-Z][a-z]+)*|[a-z]+_[a-z_]+").unwrap();
        for cap in technical_pattern.find_iter(&text) {
            keywords.push(cap.as_str().to_lowercase());
        }
        
        // Extract semantic tags
        for tag in &context.semantic_tags {
            keywords.push(tag.tag.clone());
        }
        
        // Extract from metadata tags
        keywords.extend(context.metadata.tags.clone());
        
        keywords
    }
}

/// Trait for detecting relationships between contexts
trait RelationshipDetector: Send + Sync {
    fn detect_relationship(&self, source: &EnhancedContextItem, target: &EnhancedContextItem) -> Option<ContextRelationship>;
}

/// Content similarity-based relationship detector
struct ContentSimilarityDetector;

impl ContentSimilarityDetector {
    fn new() -> Self {
        Self
    }
}

impl RelationshipDetector for ContentSimilarityDetector {
    fn detect_relationship(&self, source: &EnhancedContextItem, target: &EnhancedContextItem) -> Option<ContextRelationship> {
        let similarity = calculate_text_similarity(&source.content.description, &target.content.description);
        
        if similarity > 0.3 {
            Some(ContextRelationship::new(
                target.id.clone(),
                RelationshipType::Similar,
                similarity,
                true,
            ).with_confidence(similarity))
        } else {
            None
        }
    }
}

/// Keyword overlap-based relationship detector
struct KeywordOverlapDetector;

impl KeywordOverlapDetector {
    fn new() -> Self {
        Self
    }
}

impl RelationshipDetector for KeywordOverlapDetector {
    fn detect_relationship(&self, source: &EnhancedContextItem, target: &EnhancedContextItem) -> Option<ContextRelationship> {
        let source_tags: HashSet<_> = source.metadata.tags.iter().collect();
        let target_tags: HashSet<_> = target.metadata.tags.iter().collect();
        
        let intersection = source_tags.intersection(&target_tags).count();
        let union = source_tags.union(&target_tags).count();
        
        if union == 0 {
            return None;
        }
        
        let overlap = intersection as f64 / union as f64;
        
        if overlap > 0.2 {
            Some(ContextRelationship::new(
                target.id.clone(),
                RelationshipType::References,
                overlap,
                true,
            ).with_confidence(overlap))
        } else {
            None
        }
    }
}

/// Type-based relationship detector
struct TypeBasedDetector;

impl TypeBasedDetector {
    fn new() -> Self {
        Self
    }
}

impl RelationshipDetector for TypeBasedDetector {
    fn detect_relationship(&self, source: &EnhancedContextItem, target: &EnhancedContextItem) -> Option<ContextRelationship> {
        let relationship_type = match (&source.content.content_type, &target.content.content_type) {
            (ContextType::BusinessRule, ContextType::ArchitecturalDecision) => Some(RelationshipType::Constrains),
            (ContextType::ArchitecturalDecision, ContextType::BusinessRule) => Some(RelationshipType::Enables),
            (ContextType::SecurityPolicy, _) => Some(RelationshipType::Constrains),
            (_, ContextType::SecurityPolicy) => Some(RelationshipType::DependsOn),
            (ContextType::FeatureContext, ContextType::BusinessRule) => Some(RelationshipType::Implements),
            (ContextType::BusinessRule, ContextType::FeatureContext) => Some(RelationshipType::Validates),
            _ => None,
        };
        
        if let Some(rel_type) = relationship_type {
            Some(ContextRelationship::new(
                target.id.clone(),
                rel_type,
                0.7,
                true,
            ).with_confidence(0.8))
        } else {
            None
        }
    }
}

/// Project structure-based relationship detector
struct ProjectStructureDetector;

impl ProjectStructureDetector {
    fn new() -> Self {
        Self
    }
}

impl RelationshipDetector for ProjectStructureDetector {
    fn detect_relationship(&self, source: &EnhancedContextItem, target: &EnhancedContextItem) -> Option<ContextRelationship> {
        // Same project contexts are related
        if source.project_id == target.project_id {
            Some(ContextRelationship::new(
                target.id.clone(),
                RelationshipType::References,
                0.4,
                true,
            ).with_confidence(0.6))
        } else {
            None
        }
    }
}

/// Simple text similarity calculation
fn calculate_text_similarity(text1: &str, text2: &str) -> f64 {
    let text1_lower = text1.to_lowercase();
    let text2_lower = text2.to_lowercase();
    let words1: HashSet<_> = text1_lower.split_whitespace().collect();
    let words2: HashSet<_> = text2_lower.split_whitespace().collect();
    
    if words1.is_empty() || words2.is_empty() {
        return 0.0;
    }
    
    let intersection = words1.intersection(&words2).count();
    let union = words1.union(&words2).count();
    
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

/// Check if a word is a stop word
fn is_stop_word(word: &str) -> bool {
    matches!(word, "the" | "and" | "or" | "but" | "in" | "on" | "at" | "to" | "for" | "of" | "with" | "by" | "from" | "up" | "about" | "into" | "through" | "during" | "before" | "after" | "above" | "below" | "between" | "among" | "within" | "without" | "under" | "over" | "this" | "that" | "these" | "those" | "a" | "an" | "is" | "are" | "was" | "were" | "be" | "been" | "being" | "have" | "has" | "had" | "do" | "does" | "did" | "will" | "would" | "could" | "should" | "may" | "might" | "must" | "can" | "shall")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_context(id: &str, project_id: &str, content_type: ContextType, title: &str, description: &str) -> EnhancedContextItem {
        let content = ContextContent {
            content_type,
            title: title.to_string(),
            description: description.to_string(),
            data: json!({}),
            source_file: None,
            source_line: None,
        };
        
        let mut context = EnhancedContextItem::new(project_id.to_string(), content);
        context.id = id.to_string();
        context
    }

    #[tokio::test]
    async fn test_relationship_detection() {
        let engine = DefaultContextRelationshipEngine::new();
        
        let context1 = create_test_context(
            "1",
            "project1",
            ContextType::BusinessRule,
            "User Authentication",
            "Users must authenticate using OAuth2 with JWT tokens"
        );
        
        let context2 = create_test_context(
            "2",
            "project1",
            ContextType::SecurityPolicy,
            "JWT Security",
            "JWT tokens must be validated and have proper expiration"
        );
        
        let existing_contexts = vec![context2];
        let relationships = engine.detect_relationships(&context1, &existing_contexts).await.unwrap();
        
        assert!(!relationships.is_empty());
        assert!(relationships.iter().any(|r| r.relationship_type == RelationshipType::DependsOn));
    }

    #[tokio::test]
    async fn test_relationship_graph() {
        let engine = DefaultContextRelationshipEngine::new();
        
        let contexts = vec![
            create_test_context("1", "project1", ContextType::BusinessRule, "Auth Rule", "Authentication required"),
            create_test_context("2", "project1", ContextType::SecurityPolicy, "Auth Policy", "Use OAuth2 authentication"),
            create_test_context("3", "project1", ContextType::FeatureContext, "Login Feature", "User login functionality"),
        ];
        
        let graph = engine.build_relationship_graph(&contexts).await.unwrap();
        
        assert_eq!(graph.nodes.len(), 3);
        assert!(!graph.edges.is_empty());
    }

    #[tokio::test]
    async fn test_find_related_contexts() {
        let engine = DefaultContextRelationshipEngine::new();
        
        let contexts = vec![
            create_test_context("1", "project1", ContextType::BusinessRule, "Auth Rule", "Authentication required"),
            create_test_context("2", "project1", ContextType::SecurityPolicy, "Auth Policy", "Use OAuth2 authentication"),
            create_test_context("3", "project1", ContextType::FeatureContext, "Login Feature", "User login functionality"),
        ];
        
        let related = engine.find_related_contexts(&"1".to_string(), &contexts, 2).await.unwrap();
        
        assert!(!related.is_empty());
        assert!(related.iter().any(|r| r.context.id == "2" || r.context.id == "3"));
    }

    #[test]
    fn test_text_similarity() {
        let similarity = calculate_text_similarity(
            "User authentication with OAuth2",
            "OAuth2 authentication for users"
        );
        
        // Should have some similarity due to common words
        assert!(similarity > 0.3);
        
        // Test identical texts
        let identical_similarity = calculate_text_similarity(
            "test text",
            "test text"
        );
        assert_eq!(identical_similarity, 1.0);
        
        // Test completely different texts
        let no_similarity = calculate_text_similarity(
            "completely different",
            "nothing matches here"
        );
        assert_eq!(no_similarity, 0.0);
    }

    #[test]
    fn test_keyword_extraction() {
        let context = create_test_context(
            "1",
            "project1",
            ContextType::BusinessRule,
            "User Authentication",
            "Users must authenticate using OAuth2 with JWT tokens"
        );
        
        let extractor = SimpleKeywordExtractor::new();
        let keywords = extractor.extract_keywords(&context);
        
        assert!(keywords.contains(&"authentication".to_string()));
        assert!(keywords.contains(&"oauth2".to_string()));
    }
}