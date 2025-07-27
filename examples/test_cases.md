# Testing MCP Server API Endpoints

This file contains test examples to verify the MCP server functionality.

## Test Project Creation with Different Parameter Formats

### Test 1: Using `data` object (recommended format)
```json
{
  "action": "create",
  "data": {
    "name": "Test Project with Data Object",
    "description": "Testing project creation using nested data object",
    "repository_url": "https://github.com/test/project-data-format"
  }
}
```

### Test 2: Using direct parameters (alternative format)  
```json
{
  "action": "create",
  "name": "Test Project Direct Params",
  "description": "Testing project creation using direct parameters",
  "repository_url": "https://github.com/test/project-direct-params"
}
```

### Test 3: Creating project with minimal data
```json
{
  "action": "create",
  "data": {
    "name": "Minimal Test Project",
    "description": "Basic project for testing"
  }
}
```

## Test Entity Creation (after project exists)

### Test 4: Create business rule
```json
{
  "entity_type": "business_rule",
  "data": {
    "project_id": "YOUR_PROJECT_ID_HERE",
    "rule_name": "Test Authentication Rule",
    "description": "Users must authenticate before accessing protected resources",
    "priority": "high"
  }
}
```

### Test 5: Create framework component
```json
{
  "entity_type": "framework_component", 
  "data": {
    "project_id": "YOUR_PROJECT_ID_HERE",
    "component_name": "TestUserService",
    "component_type": "service",
    "architecture_layer": "domain",
    "file_path": "src/services/test_user_service.rs"
  }
}
```

## Expected Results

- Test 1 and Test 2 should both successfully create projects
- Test 3 should create a project with minimal required fields
- Test 4 and Test 5 should work after replacing YOUR_PROJECT_ID_HERE with actual project ID from Test 1/2/3
- All tests should return project/entity details with generated IDs

## Error Cases to Test

### Test 6: Missing action parameter
```json
{
  "name": "Project without action"
}
```
**Expected:** Error about missing required action parameter

### Test 7: Entity creation without project
```json
{
  "entity_type": "business_rule",
  "data": {
    "project_id": "nonexistent-project-id",
    "rule_name": "Test Rule"
  }
}
```
**Expected:** FOREIGN KEY constraint error

### Test 8: Invalid entity type
```json
{
  "entity_type": "invalid_entity",
  "data": {
    "project_id": "valid-project-id",
    "name": "Test"
  }
}
```
**Expected:** Unsupported entity type error
