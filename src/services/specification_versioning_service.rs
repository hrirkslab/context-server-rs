use crate::models::specification::ProjectSpecification;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rmcp::model::ErrorData as McpError;
use rusqlite::{params, Connection, Row, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Service for managing specification versions and change tracking
#[async_trait]
pub trait SpecificationVersioningService: Send + Sync {
    /// Create a new version of a specification
    async fn create_version(&self, spec: &ProjectSpecification, change_description: &str) -> Result<SpecificationVersion, McpError>;
    
    /// Get all versions of a specification
    async fn get_versions(&self, spec_id: &str) -> Result<Vec<SpecificationVersion>, McpError>;
    
    /// Get a specific version of a specification
    async fn get_version(&self, version_id: &str) -> Result<Option<SpecificationVersion>, McpError>;
    
    /// Compare two versions of a specification
    async fn compare_versions(&self, version1_id: &str, version2_id: &str) -> Result<VersionComparison, McpError>;
    
    /// Get the latest version of a specification
    async fn get_latest_version(&self, spec_id: &str) -> Result<Option<SpecificationVersion>, McpError>;
    
    /// Restore a specification to a specific version
    async fn restore_to_version(&self, spec_id: &str, version_id: &str) -> Result<ProjectSpecification, McpError>;
    
    /// Delete old versions (keep only the latest N versions)
    async fn cleanup_old_versions(&self, spec_id: &str, keep_count: usize) -> Result<usize, McpError>;
}

/// Represents a version of a specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificationVersion {
    pub id: String,
    pub spec_id: String,
    pub version_number: u32,
    pub content_hash: String,
    pub raw_content: String,
    pub parsed_sections: HashMap<String, String>,
    pub change_description: String,
    pub change_type: VersionChangeType,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub file_path: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of version changes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VersionChangeType {
    Created,
    Modified,
    Restored,
    Merged,
    AutoSync,
}

impl VersionChangeType {
    pub fn as_str(&self) -> &str {
        match self {
            VersionChangeType::Created => "created",
            VersionChangeType::Modified => "modified",
            VersionChangeType::Restored => "restored",
            VersionChangeType::Merged => "merged",
            VersionChangeType::AutoSync => "auto_sync",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "created" => VersionChangeType::Created,
            "modified" => VersionChangeType::Modified,
            "restored" => VersionChangeType::Restored,
            "merged" => VersionChangeType::Merged,
            "auto_sync" => VersionChangeType::AutoSync,
            _ => VersionChangeType::Modified,
        }
    }
}

/// Comparison between two specification versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionComparison {
    pub version1: SpecificationVersion,
    pub version2: SpecificationVersion,
    pub differences: Vec<VersionDifference>,
    pub similarity_score: f64,
}

/// Represents a difference between two versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDifference {
    pub section: String,
    pub change_type: DifferenceType,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub line_number: Option<usize>,
}

/// Types of differences between versions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DifferenceType {
    Added,
    Removed,
    Modified,
    Moved,
}

/// SQLite implementation of SpecificationVersioningService
pub struct SqliteSpecificationVersioningService {
    db: Arc<Mutex<Connection>>,
}

impl SqliteSpecificationVersioningService {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }

    /// Initialize database tables for specification versioning
    pub fn initialize_tables(&self) -> Result<(), McpError> {
        let db = self.db.lock().unwrap();

        // Create specification_versions table
        db.execute(
            r#"
            CREATE TABLE IF NOT EXISTS specification_versions (
                id TEXT PRIMARY KEY,
                spec_id TEXT NOT NULL,
                version_number INTEGER NOT NULL,
                content_hash TEXT NOT NULL,
                raw_content TEXT NOT NULL,
                parsed_sections TEXT, -- JSON
                change_description TEXT NOT NULL,
                change_type TEXT NOT NULL,
                created_at TEXT NOT NULL,
                created_by TEXT,
                file_path TEXT,
                metadata TEXT, -- JSON
                FOREIGN KEY (spec_id) REFERENCES specifications (id) ON DELETE CASCADE
            )
            "#,
            [],
        ).map_err(|e| McpError::internal_error(format!("Failed to create specification_versions table: {}", e), None))?;

        // Create indexes for better performance
        db.execute("CREATE INDEX IF NOT EXISTS idx_spec_versions_spec_id ON specification_versions (spec_id)", [])
            .map_err(|e| McpError::internal_error(format!("Failed to create index: {}", e), None))?;
        
        db.execute("CREATE INDEX IF NOT EXISTS idx_spec_versions_version_number ON specification_versions (spec_id, version_number)", [])
            .map_err(|e| McpError::internal_error(format!("Failed to create index: {}", e), None))?;

        Ok(())
    }

    /// Calculate content hash for change detection
    fn calculate_content_hash(content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Convert database row to SpecificationVersion
    fn row_to_version(row: &Row) -> Result<SpecificationVersion, rusqlite::Error> {
        let parsed_sections: HashMap<String, String> = row.get::<_, Option<String>>(5)?
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let metadata: HashMap<String, serde_json::Value> = row.get::<_, Option<String>>(11)?
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        Ok(SpecificationVersion {
            id: row.get(0)?,
            spec_id: row.get(1)?,
            version_number: row.get::<_, i64>(2)? as u32,
            content_hash: row.get(3)?,
            raw_content: row.get(4)?,
            parsed_sections,
            change_description: row.get(6)?,
            change_type: VersionChangeType::from_str(&row.get::<_, String>(7)?),
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(8, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            created_by: row.get(9)?,
            file_path: row.get(10)?,
            metadata,
        })
    }

    /// Calculate differences between two content strings
    fn calculate_differences(old_content: &str, new_content: &str) -> Vec<VersionDifference> {
        let mut differences = Vec::new();
        
        let old_lines: Vec<&str> = old_content.lines().collect();
        let new_lines: Vec<&str> = new_content.lines().collect();
        
        // Simple line-by-line comparison
        let max_lines = old_lines.len().max(new_lines.len());
        
        for i in 0..max_lines {
            let old_line = old_lines.get(i).copied();
            let new_line = new_lines.get(i).copied();
            
            match (old_line, new_line) {
                (Some(old), Some(new)) if old != new => {
                    differences.push(VersionDifference {
                        section: format!("line_{}", i + 1),
                        change_type: DifferenceType::Modified,
                        old_content: Some(old.to_string()),
                        new_content: Some(new.to_string()),
                        line_number: Some(i + 1),
                    });
                }
                (Some(old), None) => {
                    differences.push(VersionDifference {
                        section: format!("line_{}", i + 1),
                        change_type: DifferenceType::Removed,
                        old_content: Some(old.to_string()),
                        new_content: None,
                        line_number: Some(i + 1),
                    });
                }
                (None, Some(new)) => {
                    differences.push(VersionDifference {
                        section: format!("line_{}", i + 1),
                        change_type: DifferenceType::Added,
                        old_content: None,
                        new_content: Some(new.to_string()),
                        line_number: Some(i + 1),
                    });
                }
                _ => {} // Lines are the same or both None
            }
        }
        
        differences
    }

    /// Calculate similarity score between two content strings
    fn calculate_similarity_score(old_content: &str, new_content: &str) -> f64 {
        let old_lines: Vec<&str> = old_content.lines().collect();
        let new_lines: Vec<&str> = new_content.lines().collect();
        
        if old_lines.is_empty() && new_lines.is_empty() {
            return 1.0;
        }
        
        let max_lines = old_lines.len().max(new_lines.len());
        if max_lines == 0 {
            return 1.0;
        }
        
        let mut matching_lines = 0;
        let min_lines = old_lines.len().min(new_lines.len());
        
        for i in 0..min_lines {
            if old_lines[i] == new_lines[i] {
                matching_lines += 1;
            }
        }
        
        matching_lines as f64 / max_lines as f64
    }
}

#[async_trait]
impl SpecificationVersioningService for SqliteSpecificationVersioningService {
    async fn create_version(&self, spec: &ProjectSpecification, change_description: &str) -> Result<SpecificationVersion, McpError> {
        let db = self.db.lock().unwrap();

        let content_hash = Self::calculate_content_hash(&spec.content.raw_content);
        
        // Check if this content already exists as a version
        let mut existing_stmt = db.prepare(
            "SELECT id FROM specification_versions WHERE spec_id = ? AND content_hash = ?"
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        let existing_version = existing_stmt.query_row([&spec.id, &content_hash], |row| {
            Ok(row.get::<_, String>(0)?)
        }).optional().map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        if existing_version.is_some() {
            return Err(McpError::invalid_params("Version with identical content already exists".to_string(), None));
        }

        let version = SpecificationVersion {
            id: Uuid::new_v4().to_string(),
            spec_id: spec.id.clone(),
            version_number: spec.version,
            content_hash,
            raw_content: spec.content.raw_content.clone(),
            parsed_sections: spec.content.parsed_sections.clone(),
            change_description: change_description.to_string(),
            change_type: VersionChangeType::Modified,
            created_at: Utc::now(),
            created_by: None, // Could be enhanced to track user
            file_path: spec.file_path.clone(),
            metadata: HashMap::new(),
        };

        let parsed_sections_json = serde_json::to_string(&version.parsed_sections)
            .map_err(|e| McpError::internal_error(format!("Failed to serialize parsed sections: {}", e), None))?;

        let metadata_json = serde_json::to_string(&version.metadata)
            .map_err(|e| McpError::internal_error(format!("Failed to serialize metadata: {}", e), None))?;

        db.execute(
            r#"
            INSERT INTO specification_versions (
                id, spec_id, version_number, content_hash, raw_content, parsed_sections,
                change_description, change_type, created_at, created_by, file_path, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                &version.id,
                &version.spec_id,
                version.version_number,
                &version.content_hash,
                &version.raw_content,
                parsed_sections_json,
                &version.change_description,
                version.change_type.as_str(),
                version.created_at.to_rfc3339(),
                &version.created_by,
                &version.file_path,
                metadata_json,
            ],
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(version)
    }

    async fn get_versions(&self, spec_id: &str) -> Result<Vec<SpecificationVersion>, McpError> {
        let db = self.db.lock().unwrap();
        let mut versions = Vec::new();

        let mut stmt = db.prepare(
            r#"
            SELECT id, spec_id, version_number, content_hash, raw_content, parsed_sections,
                   change_description, change_type, created_at, created_by, file_path, metadata
            FROM specification_versions WHERE spec_id = ? ORDER BY version_number DESC
            "#
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        let version_rows = stmt.query_map([spec_id], Self::row_to_version)
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        for version in version_rows {
            match version {
                Ok(version) => versions.push(version),
                Err(e) => tracing::warn!("Failed to parse specification version: {}", e),
            }
        }

        Ok(versions)
    }

    async fn get_version(&self, version_id: &str) -> Result<Option<SpecificationVersion>, McpError> {
        let db = self.db.lock().unwrap();

        let mut stmt = db.prepare(
            r#"
            SELECT id, spec_id, version_number, content_hash, raw_content, parsed_sections,
                   change_description, change_type, created_at, created_by, file_path, metadata
            FROM specification_versions WHERE id = ?
            "#
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        let mut version_iter = stmt.query_map([version_id], Self::row_to_version)
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        match version_iter.next() {
            Some(Ok(version)) => Ok(Some(version)),
            Some(Err(e)) => Err(McpError::internal_error(format!("Database error: {}", e), None)),
            None => Ok(None),
        }
    }

    async fn compare_versions(&self, version1_id: &str, version2_id: &str) -> Result<VersionComparison, McpError> {
        let version1 = self.get_version(version1_id).await?
            .ok_or_else(|| McpError::resource_not_found(format!("Version not found: {}", version1_id), None))?;

        let version2 = self.get_version(version2_id).await?
            .ok_or_else(|| McpError::resource_not_found(format!("Version not found: {}", version2_id), None))?;

        let differences = Self::calculate_differences(&version1.raw_content, &version2.raw_content);
        let similarity_score = Self::calculate_similarity_score(&version1.raw_content, &version2.raw_content);

        Ok(VersionComparison {
            version1,
            version2,
            differences,
            similarity_score,
        })
    }

    async fn get_latest_version(&self, spec_id: &str) -> Result<Option<SpecificationVersion>, McpError> {
        let db = self.db.lock().unwrap();

        let mut stmt = db.prepare(
            r#"
            SELECT id, spec_id, version_number, content_hash, raw_content, parsed_sections,
                   change_description, change_type, created_at, created_by, file_path, metadata
            FROM specification_versions WHERE spec_id = ? ORDER BY version_number DESC LIMIT 1
            "#
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        let mut version_iter = stmt.query_map([spec_id], Self::row_to_version)
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        match version_iter.next() {
            Some(Ok(version)) => Ok(Some(version)),
            Some(Err(e)) => Err(McpError::internal_error(format!("Database error: {}", e), None)),
            None => Ok(None),
        }
    }

    async fn restore_to_version(&self, spec_id: &str, version_id: &str) -> Result<ProjectSpecification, McpError> {
        let version = self.get_version(version_id).await?
            .ok_or_else(|| McpError::resource_not_found(format!("Version not found: {}", version_id), None))?;

        // This would typically involve updating the main specification record
        // For now, we'll return a reconstructed ProjectSpecification
        // In a real implementation, you'd update the specification in the repository
        
        Err(McpError::internal_error("restore_to_version not fully implemented".to_string(), None))
    }

    async fn cleanup_old_versions(&self, spec_id: &str, keep_count: usize) -> Result<usize, McpError> {
        let db = self.db.lock().unwrap();

        // Get versions to delete (keep only the latest N)
        let mut stmt = db.prepare(
            r#"
            SELECT id FROM specification_versions 
            WHERE spec_id = ? 
            ORDER BY version_number DESC 
            LIMIT -1 OFFSET ?
            "#
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        let version_ids: Result<Vec<String>, rusqlite::Error> = stmt.query_map([spec_id, &keep_count.to_string()], |row| {
            Ok(row.get::<_, String>(0)?)
        }).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?
        .collect();

        let version_ids = version_ids.map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        let mut deleted_count = 0;
        for version_id in version_ids {
            let rows_affected = db.execute("DELETE FROM specification_versions WHERE id = ?", [&version_id])
                .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;
            
            if rows_affected > 0 {
                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::specification::{SpecContent, SpecFormat, SpecType};
    use rusqlite::Connection;
    use tempfile::NamedTempFile;

    fn create_test_db() -> Arc<Mutex<Connection>> {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Connection::open(temp_file.path()).unwrap();
        Arc::new(Mutex::new(db))
    }

    fn create_test_spec() -> ProjectSpecification {
        let content = SpecContent::new(
            SpecFormat::Markdown,
            "# Test Specification\n\nThis is a test.".to_string(),
        );
        ProjectSpecification::new(
            "test-project".to_string(),
            SpecType::Requirements,
            "Test Spec".to_string(),
            content,
        )
    }

    #[tokio::test]
    async fn test_create_version() {
        let db = create_test_db();
        let service = SqliteSpecificationVersioningService::new(db);
        service.initialize_tables().unwrap();

        let spec = create_test_spec();
        let result = service.create_version(&spec, "Initial version").await;
        
        assert!(result.is_ok());
        let version = result.unwrap();
        assert_eq!(version.spec_id, spec.id);
        assert_eq!(version.version_number, spec.version);
        assert_eq!(version.change_description, "Initial version");
    }

    #[tokio::test]
    async fn test_get_versions() {
        let db = create_test_db();
        let service = SqliteSpecificationVersioningService::new(db);
        service.initialize_tables().unwrap();

        let spec = create_test_spec();
        let _version1 = service.create_version(&spec, "Version 1").await.unwrap();
        
        let mut spec2 = spec.clone();
        spec2.version = 2;
        spec2.content.raw_content = "# Updated Specification\n\nThis is updated.".to_string();
        let _version2 = service.create_version(&spec2, "Version 2").await.unwrap();

        let versions = service.get_versions(&spec.id).await.unwrap();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version_number, 2); // Should be ordered by version DESC
        assert_eq!(versions[1].version_number, 1);
    }

    #[tokio::test]
    async fn test_compare_versions() {
        let db = create_test_db();
        let service = SqliteSpecificationVersioningService::new(db);
        service.initialize_tables().unwrap();

        let spec1 = create_test_spec();
        let version1 = service.create_version(&spec1, "Version 1").await.unwrap();
        
        let mut spec2 = spec1.clone();
        spec2.version = 2;
        spec2.content.raw_content = "# Updated Specification\n\nThis is updated content.".to_string();
        let version2 = service.create_version(&spec2, "Version 2").await.unwrap();

        let comparison = service.compare_versions(&version1.id, &version2.id).await.unwrap();
        assert!(!comparison.differences.is_empty());
        assert!(comparison.similarity_score < 1.0);
    }

    #[test]
    fn test_calculate_content_hash() {
        let content1 = "Hello, world!";
        let content2 = "Hello, world!";
        let content3 = "Hello, universe!";

        let hash1 = SqliteSpecificationVersioningService::calculate_content_hash(content1);
        let hash2 = SqliteSpecificationVersioningService::calculate_content_hash(content2);
        let hash3 = SqliteSpecificationVersioningService::calculate_content_hash(content3);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_calculate_differences() {
        let old_content = "Line 1\nLine 2\nLine 3";
        let new_content = "Line 1\nModified Line 2\nLine 3\nLine 4";

        let differences = SqliteSpecificationVersioningService::calculate_differences(old_content, new_content);
        
        assert_eq!(differences.len(), 2);
        assert_eq!(differences[0].change_type, DifferenceType::Modified);
        assert_eq!(differences[1].change_type, DifferenceType::Added);
    }

    #[test]
    fn test_calculate_similarity_score() {
        let content1 = "Line 1\nLine 2\nLine 3";
        let content2 = "Line 1\nLine 2\nLine 3";
        let content3 = "Line 1\nModified Line 2\nLine 3";
        let content4 = "Completely different content";

        let score1 = SqliteSpecificationVersioningService::calculate_similarity_score(content1, content2);
        let score2 = SqliteSpecificationVersioningService::calculate_similarity_score(content1, content3);
        let score3 = SqliteSpecificationVersioningService::calculate_similarity_score(content1, content4);

        assert_eq!(score1, 1.0);
        assert!(score2 > 0.5 && score2 < 1.0);
        assert!(score3 < 0.5);
    }
}