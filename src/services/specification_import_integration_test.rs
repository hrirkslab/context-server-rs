#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::AppContainer;
    use std::path::Path;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_specification_import_integration() {
        // Create a temporary directory structure
        let temp_dir = TempDir::new().unwrap();
        let kiro_specs_dir = temp_dir.path().join(".kiro").join("specs").join("test-project");
        fs::create_dir_all(&kiro_specs_dir).await.unwrap();

        // Create test specification files
        let requirements_content = r#"
# Requirements Document

## Introduction

This is a test requirements document for the professional context engine.

## Requirements

### Requirement 1

**User Story:** As a developer, I want to import specifications automatically, so that I can keep my context up to date.

#### Acceptance Criteria

1. WHEN a specification file is created THEN the system SHALL automatically import it
2. WHEN a specification file is modified THEN the system SHALL update the existing specification
3. WHEN a specification file is deleted THEN the system SHALL remove it from the context
"#;

        let design_content = r#"
# Design Document

## Overview

This design document outlines the specification import system architecture.

## Architecture

The system consists of:
- File system monitoring
- Specification parsing
- Version tracking
- Change detection

## Components and Interfaces

### SpecificationImportService
- Handles file scanning and import
- Monitors file system changes
- Validates specification content

### SpecificationVersioningService  
- Tracks specification versions
- Compares versions
- Manages version history
"#;

        let tasks_content = r#"
# Implementation Plan

- [x] 1. Create specification import service
  - Implement file scanning functionality
  - Add specification validation
  - _Requirements: 1.1, 1.2_

- [ ] 2. Add file system monitoring
  - [ ] 2.1 Implement file watcher
  - [ ] 2.2 Handle file change events
  - _Requirements: 1.3_

- [ ] 3. Create version tracking
  - Implement version comparison
  - Add change history
  - _Requirements: 1.1_
"#;

        // Write the test files
        fs::write(kiro_specs_dir.join("requirements.md"), requirements_content).await.unwrap();
        fs::write(kiro_specs_dir.join("design.md"), design_content).await.unwrap();
        fs::write(kiro_specs_dir.join("tasks.md"), tasks_content).await.unwrap();

        // Create the application container with a test database
        let db_path = temp_dir.path().join("test.db");
        let container = AppContainer::new(db_path.to_str().unwrap()).unwrap();

        // Test scanning and importing specifications
        let specs_path = temp_dir.path().join(".kiro").join("specs");
        let imported_specs = container
            .specification_import_service
            .scan_and_import_specifications(&specs_path)
            .await
            .unwrap();

        // Verify that all three specifications were imported
        assert_eq!(imported_specs.len(), 3);

        // Check that we have the expected specification types
        let spec_types: Vec<_> = imported_specs.iter().map(|s| &s.spec_type).collect();
        assert!(spec_types.iter().any(|t| matches!(t, crate::models::specification::SpecType::Requirements)));
        assert!(spec_types.iter().any(|t| matches!(t, crate::models::specification::SpecType::Design)));
        assert!(spec_types.iter().any(|t| matches!(t, crate::models::specification::SpecType::Tasks)));

        // Test individual file import
        let requirements_file = kiro_specs_dir.join("requirements.md");
        let imported_spec = container
            .specification_import_service
            .import_specification_file(&requirements_file)
            .await
            .unwrap();

        assert_eq!(imported_spec.spec_type, crate::models::specification::SpecType::Requirements);
        assert_eq!(imported_spec.project_id, "test-project");
        assert!(imported_spec.content.raw_content.contains("professional context engine"));

        // Test specification validation
        let validation_issues = container
            .specification_import_service
            .validate_specification_file(&requirements_file)
            .await
            .unwrap();

        // Should have no validation issues for a properly structured file
        assert!(validation_issues.is_empty());

        // Test versioning service
        let version = container
            .specification_versioning_service
            .create_version(&imported_spec, "Initial import")
            .await
            .unwrap();

        assert_eq!(version.spec_id, imported_spec.id);
        assert_eq!(version.version_number, imported_spec.version);
        assert_eq!(version.change_description, "Initial import");

        // Test getting versions
        let versions = container
            .specification_versioning_service
            .get_versions(&imported_spec.id)
            .await
            .unwrap();

        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].id, version.id);

        println!("✅ Specification import integration test passed!");
    }

    #[tokio::test]
    async fn test_specification_validation_errors() {
        let temp_dir = TempDir::new().unwrap();
        let kiro_specs_dir = temp_dir.path().join(".kiro").join("specs").join("invalid-project");
        fs::create_dir_all(&kiro_specs_dir).await.unwrap();

        // Create an invalid specification file (wrong name for content type)
        let invalid_content = r#"
# Tasks Document

This should be a requirements document but has the wrong content.
"#;

        fs::write(kiro_specs_dir.join("requirements.md"), invalid_content).await.unwrap();

        let db_path = temp_dir.path().join("test.db");
        let container = AppContainer::new(db_path.to_str().unwrap()).unwrap();

        // Test validation of invalid file
        let requirements_file = kiro_specs_dir.join("requirements.md");
        let validation_issues = container
            .specification_import_service
            .validate_specification_file(&requirements_file)
            .await
            .unwrap();

        // Should have validation issues
        assert!(!validation_issues.is_empty());
        println!("Validation issues: {:?}", validation_issues);
        // The validation should detect that this is not a proper requirements specification
        assert!(validation_issues.iter().any(|issue| issue.contains("requirements") || issue.contains("specification")));

        println!("✅ Specification validation error test passed!");
    }
}