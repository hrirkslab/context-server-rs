/// Query caching module for performance optimization
/// Provides LRU and TTL-based caching for frequently accessed queries

use lru::LruCache;
use parking_lot::RwLock;
use serde_json::Value;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, trace};

/// Cache entry with optional TTL
#[derive(Clone)]
pub struct CacheEntry {
    data: Value,
    created_at: Instant,
    ttl: Option<Duration>,
}

impl CacheEntry {
    /// Check if this cache entry has expired
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.created_at.elapsed() > ttl
        } else {
            false
        }
    }
}

/// Query result cache with LRU eviction and TTL support
pub struct QueryCache {
    cache: Arc<RwLock<LruCache<String, CacheEntry>>>,
    max_size: usize,
}

impl QueryCache {
    /// Create a new query cache with specified maximum size
    pub fn new(max_size: usize) -> Self {
        let non_zero_size = NonZeroUsize::new(max_size).unwrap_or(NonZeroUsize::new(1000).unwrap());
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(non_zero_size))),
            max_size,
        }
    }

    /// Get a cached value if it exists and hasn't expired
    pub fn get(&self, key: &str) -> Option<Value> {
        let mut cache = self.cache.write();
        if let Some(entry) = cache.get_mut(key) {
            if entry.is_expired() {
                trace!("Cache entry expired for key: {}", key);
                cache.pop(key);
                return None;
            }
            debug!("Cache hit for key: {}", key);
            return Some(entry.data.clone());
        }
        trace!("Cache miss for key: {}", key);
        None
    }

    /// Store a value in cache with optional TTL
    pub fn set(&self, key: String, value: Value, ttl: Option<Duration>) {
        let entry = CacheEntry {
            data: value,
            created_at: Instant::now(),
            ttl,
        };
        let mut cache = self.cache.write();
        cache.put(key.clone(), entry);
        debug!("Cached value for key: {}", key);
    }

    /// Clear a specific cache entry
    pub fn invalidate(&self, key: &str) {
        let mut cache = self.cache.write();
        if cache.pop(key).is_some() {
            debug!("Invalidated cache entry for key: {}", key);
        }
    }

    /// Clear all cache entries matching a pattern
    pub fn invalidate_pattern(&self, pattern: &str) {
        let mut cache = self.cache.write();
        let keys_to_remove: Vec<String> = cache
            .iter()
            .filter(|(k, _)| k.contains(pattern))
            .map(|(k, _)| k.clone())
            .collect();

        for key in keys_to_remove {
            cache.pop(&key);
        }
        debug!("Invalidated {} cache entries matching pattern: {}", cache.len(), pattern);
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        let mut cache = self.cache.write();
        cache.clear();
        debug!("Cleared all cache entries");
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.read();
        CacheStats {
            size: cache.len(),
            max_size: self.max_size,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
}

impl CacheStats {
    /// Calculate cache hit ratio
    pub fn utilization_percent(&self) -> f64 {
        if self.max_size == 0 {
            0.0
        } else {
            (self.size as f64 / self.max_size as f64) * 100.0
        }
    }
}

/// Build cache keys for different query types
pub struct CacheKeyBuilder;

impl CacheKeyBuilder {
    /// Build cache key for project queries
    pub fn project(project_id: &str) -> String {
        format!("project:{}", project_id)
    }

    /// Build cache key for business rule queries
    pub fn business_rule(rule_id: &str) -> String {
        format!("rule:{}", rule_id)
    }

    /// Build cache key for business rules by project
    pub fn business_rules_by_project(project_id: &str) -> String {
        format!("rules:project:{}", project_id)
    }

    /// Build cache key for architectural decision queries
    pub fn architectural_decision(decision_id: &str) -> String {
        format!("decision:{}", decision_id)
    }

    /// Build cache key for architectural decisions by project
    pub fn architectural_decisions_by_project(project_id: &str) -> String {
        format!("decisions:project:{}", project_id)
    }

    /// Build cache key for performance requirement queries
    pub fn performance_requirement(req_id: &str) -> String {
        format!("perf_req:{}", req_id)
    }

    /// Build cache key for performance requirements by project
    pub fn performance_requirements_by_project(project_id: &str) -> String {
        format!("perf_reqs:project:{}", project_id)
    }

    /// Build cache key for security policy queries
    pub fn security_policy(policy_id: &str) -> String {
        format!("policy:{}", policy_id)
    }

    /// Build cache key for security policies by project
    pub fn security_policies_by_project(project_id: &str) -> String {
        format!("policies:project:{}", project_id)
    }

    /// Build cache key for feature context queries
    pub fn feature_context(context_id: &str) -> String {
        format!("feature:{}", context_id)
    }

    /// Build cache key for feature contexts by project
    pub fn feature_contexts_by_project(project_id: &str) -> String {
        format!("features:project:{}", project_id)
    }

    /// Build cache key for framework component queries
    pub fn framework_component(component_id: &str) -> String {
        format!("component:{}", component_id)
    }

    /// Build cache key for framework components by project
    pub fn framework_components_by_project(project_id: &str) -> String {
        format!("components:project:{}", project_id)
    }

    /// Build cache key for all projects
    pub fn all_projects() -> String {
        "projects:all".to_string()
    }

    /// Invalidation pattern for a project's data
    pub fn project_invalidation_pattern(project_id: &str) -> String {
        format!("*:project:{}", project_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let cache = QueryCache::new(100);
        let key = "test_key".to_string();
        let value = serde_json::json!({"name": "test"});

        // Test set and get
        cache.set(key.clone(), value.clone(), None);
        assert_eq!(cache.get(&key), Some(value));
    }

    #[test]
    fn test_cache_expiration() {
        let cache = QueryCache::new(100);
        let key = "test_key".to_string();
        let value = serde_json::json!({"name": "test"});

        // Set with 1ms TTL
        cache.set(key.clone(), value.clone(), Some(Duration::from_millis(1)));
        assert_eq!(cache.get(&key), Some(value));

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(cache.get(&key), None);
    }

    #[test]
    fn test_cache_invalidation() {
        let cache = QueryCache::new(100);
        let key = "test_key".to_string();
        let value = serde_json::json!({"name": "test"});

        cache.set(key.clone(), value, None);
        assert!(cache.get(&key).is_some());

        cache.invalidate(&key);
        assert_eq!(cache.get(&key), None);
    }

    #[test]
    fn test_cache_key_builder() {
        assert_eq!(CacheKeyBuilder::project("p1"), "project:p1");
        assert_eq!(CacheKeyBuilder::business_rule("r1"), "rule:r1");
        assert_eq!(CacheKeyBuilder::business_rules_by_project("p1"), "rules:project:p1");
    }
}
