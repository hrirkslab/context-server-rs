# Troubleshooting Common MCP Server Errors

## Error: "Missing required parameter: data for create"

**Problem:** When using `manage_project` with `action: "create"`, the server expects either:
- A `data` object containing project details, OR
- Direct parameters like `name`, `description`, etc.

**Solutions:**

### Option 1: Using `data` object (recommended)
```json
{
  "action": "create",
  "data": {
    "name": "My Project",
    "description": "Project description",
    "repository_url": "https://github.com/user/repo"
  }
}
```

### Option 2: Using direct parameters
```json
{
  "action": "create",
  "name": "My Project",
  "description": "Project description", 
  "repository_url": "https://github.com/user/repo"
}
```

---

## Error: "FOREIGN KEY constraint failed"

**Problem:** This occurs when trying to create entities (components, requirements, etc.) before creating the parent project.

**Solution:**
1. **Always create the project first:**
   ```json
   {
     "action": "create",
     "data": {
       "name": "My Project",
       "description": "A sample project"
     }
   }
   ```

2. **Then create related entities using the project ID:**
   ```json
   {
     "entity_type": "business_rule",
     "data": {
       "project_id": "the-project-id-from-step-1",
       "rule_name": "User Registration Rule",
       "description": "Users must provide valid email"
     }
   }
   ```

---

## Best Practices

1. **Always create projects first** before adding any components or entities.
2. **Save the project ID** returned from project creation for use in subsequent calls.
3. **Use the `list_projects` action** to check existing projects if unsure.
4. **Check entity relationships** - some entities may depend on others being created first.

---

## Example Workflow

```json
// 1. Create project
{"action": "create", "data": {"name": "My App"}}
// Response: {"id": "proj-123", "name": "My App", ...}

// 2. Add business rule
{"entity_type": "business_rule", "data": {"project_id": "proj-123", "rule_name": "Login Rule"}}

// 3. Add component
{"entity_type": "framework_component", "data": {"project_id": "proj-123", "component_name": "LoginForm"}}
```

This ensures proper entity relationships and avoids constraint errors.
