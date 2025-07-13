# API Usage Examples

This document provides practical examples for using the context-server-rs MCP API.

## Project Management

### Create a new project
```json
{
  "action": "create",
  "data": {
    "name": "E-commerce Platform",
    "description": "A modern e-commerce solution with microservices architecture",
    "repository_url": "https://github.com/company/ecommerce-platform",
    "project_type": "web_application",
    "architecture_style": "microservices"
  }
}
```

### List all projects
```json
{
  "action": "list"
}
```

### Get project details
```json
{
  "action": "get",
  "id": "proj-12345"
}
```

### Update project
```json
{
  "action": "update",
  "id": "proj-12345",
  "data": {
    "description": "Updated description with new features",
    "status": "in_development"
  }
}
```

## Entity Management

### Create Business Rules
```json
{
  "entity_type": "business_rule",
  "data": {
    "project_id": "proj-12345",
    "rule_name": "User Authentication",
    "description": "All users must authenticate before accessing protected resources",
    "priority": "high",
    "category": "security"
  }
}
```

### Create Framework Components
```json
{
  "entity_type": "framework_component",
  "data": {
    "project_id": "proj-12345",
    "component_name": "UserService",
    "component_type": "service",
    "architecture_layer": "domain",
    "file_path": "src/services/user_service.rs",
    "description": "Handles user registration, authentication, and profile management"
  }
}
```

### Create Architectural Decisions
```json
{
  "entity_type": "architectural_decision",
  "data": {
    "project_id": "proj-12345",
    "decision_title": "Database Selection",
    "description": "Use PostgreSQL for primary data storage",
    "rationale": "ACID compliance, strong consistency, and excellent performance for complex queries",
    "alternatives_considered": "MySQL, MongoDB, SQLite",
    "status": "approved"
  }
}
```

### Create Performance Requirements
```json
{
  "entity_type": "performance_requirement",
  "data": {
    "project_id": "proj-12345",
    "requirement_name": "API Response Time",
    "description": "All API endpoints must respond within 200ms under normal load",
    "metric_type": "response_time",
    "target_value": "200",
    "unit": "milliseconds",
    "priority": "high"
  }
}
```

## Bulk Operations

### Create multiple components at once
```json
{
  "project_id": "proj-12345",
  "components": [
    {
      "component_name": "UserController",
      "component_type": "widget",
      "architecture_layer": "presentation",
      "file_path": "src/controllers/user_controller.rs"
    },
    {
      "component_name": "UserRepository",
      "component_type": "repository", 
      "architecture_layer": "data",
      "file_path": "src/repositories/user_repository.rs"
    },
    {
      "component_name": "UserModel",
      "component_type": "model",
      "architecture_layer": "domain",
      "file_path": "src/models/user.rs"
    }
  ]
}
```

## Context Queries

### Query project context for a specific feature
```json
{
  "project_id": "proj-12345",
  "feature_area": "user_authentication",
  "task_type": "implement",
  "components": ["UserService", "AuthController", "TokenValidator"]
}
```

### Query for optimization tasks
```json
{
  "project_id": "proj-12345",
  "feature_area": "api_performance",
  "task_type": "optimize",
  "components": ["DatabaseConnection", "CacheLayer", "ResponseSerializer"]
}
```

## Architecture Validation

### Validate Clean Architecture compliance
```json
{
  "project_id": "proj-12345"
}
```

## Error Handling Examples

### Handling missing project errors
If you get a "FOREIGN KEY constraint failed" error, ensure the project exists:

```json
// First, list projects to check
{"action": "list"}

// If project doesn't exist, create it first
{
  "action": "create",
  "data": {
    "name": "My Project",
    "description": "Project description"
  }
}

// Then create your entities using the returned project ID
```

### Handling validation errors
The server validates entity data according to business rules. Common validation errors include:

- Missing required fields
- Invalid architecture layer combinations
- Duplicate component names within the same project
- Invalid enum values (e.g., component_type, architecture_layer)

Always check the error message for specific validation requirements.
