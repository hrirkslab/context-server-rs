# 🧠 MCP Context Server for AI Code Generation

## 📘 Project Overview

A **lightweight, focused Model Context Protocol (MCP) server** that captures and maintains only the essential project information that AI agents (like GitHub Copilot) cannot automatically discover or infer. This server provides **curated, high-value context** that enables AI agents to generate production-quality code without duplicating built-in AI capabilities. 

**Implementation:** This is a proper MCP server implementation using the official [Rust MCP SDK (rmcp)](https://github.com/modelcontextprotocol/rust-sdk), following the [Model Context Protocol specification](https://modelcontextprotocol.io/introduction).

## 🎯 Core Principle

**Only capture what AI agents can't automatically determine:**
- Business rules and domain logic
- Project-specific patterns and conventions
- Architectural decisions and constraints
- Performance requirements and bottlenecks
- Security policies and compliance rules

## 🚫 What We DON'T Build (AI Agents Handle This)

- ❌ File structure analysis
- ❌ Import/dependency mapping
- ❌ Method signature extraction
- ❌ Basic syntax parsing
- ❌ Class relationship inference
- ❌ Standard design pattern recognition
- ❌ Generic code completion
- ❌ Real-time file watching
- ❌ AST parsing and analysis

## ✅ What We DO Build (High-Value Context)

### 1. **Business Rules Repository**
```sql
-- Domain-specific business logic that AI can't infer
CREATE TABLE business_rules (
    id TEXT PRIMARY KEY,
    rule_name TEXT NOT NULL,
    description TEXT,
    domain_area TEXT, -- 'authentication', 'payments', 'user_management'
    implementation_pattern TEXT, -- How this rule is typically implemented
    constraints TEXT, -- JSON array of business constraints and validations
    examples TEXT, -- JSON array of code examples showing correct implementation
    created_at TEXT DEFAULT (datetime('now'))
);
```

### 2. **Architectural Decisions**
```sql
-- Project-specific architectural choices and patterns
CREATE TABLE architectural_decisions (
    id TEXT PRIMARY KEY,
    decision_title TEXT NOT NULL,
    context TEXT, -- Why this decision was made
    decision TEXT, -- What was decided
    consequences TEXT, -- Implications of this decision
    alternatives_considered TEXT,
    status TEXT, -- 'active', 'deprecated', 'proposed'
    created_at TEXT DEFAULT (datetime('now'))
);
```

### 3. **Performance Requirements**
```sql
-- Performance-critical areas and optimization patterns
CREATE TABLE performance_requirements (
    id TEXT PRIMARY KEY,
    component_area TEXT, -- 'image_loading', 'list_rendering', 'api_calls'
    requirement_type TEXT, -- 'response_time', 'memory_usage', 'battery_impact'
    target_value TEXT, -- '< 100ms', '< 50MB', 'minimal'
    optimization_patterns TEXT, -- JSON array
    avoid_patterns TEXT, -- JSON array
    created_at TEXT DEFAULT (datetime('now'))
);
```

### 4. **Security Policies**
```sql
-- Security requirements and implementation patterns
CREATE TABLE security_policies (
    id TEXT PRIMARY KEY,
    policy_name TEXT NOT NULL,
    policy_area TEXT, -- 'authentication', 'data_storage', 'api_security'
    requirements TEXT,
    implementation_pattern TEXT,
    forbidden_patterns TEXT, -- JSON array
    compliance_notes TEXT, -- GDPR, HIPAA, etc.
    created_at TEXT DEFAULT (datetime('now'))
);
```

### 5. **Project Conventions**
```sql
-- Team-specific coding standards and patterns
CREATE TABLE project_conventions (
    id TEXT PRIMARY KEY,
    convention_type TEXT, -- 'naming', 'error_handling', 'state_management'
    convention_rule TEXT,
    good_examples TEXT, -- JSON array
    bad_examples TEXT, -- JSON array
    rationale TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);
```

### 6. **Feature Context**
```sql
-- High-level feature context and requirements
CREATE TABLE feature_context (
    id TEXT PRIMARY KEY,
    feature_name TEXT NOT NULL,
    business_purpose TEXT,
    user_personas TEXT, -- JSON array
    key_workflows TEXT, -- JSON array
    integration_points TEXT, -- JSON array
    edge_cases TEXT, -- JSON array
    created_at TEXT DEFAULT (datetime('now'))
);
```

## 🏗️ MCP Architecture

```
┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐
│   Manual Input      │    │   MCP Context       │    │   MCP Client        │
│   (Developers)      │───▶│   Server            │───▶│   (Claude Desktop,  │
│                     │    │   (Model Context    │    │   Cursor IDE, etc.) │
│                     │    │   Protocol)         │    │                     │
└─────────────────────┘    │   (SQLite, embedded)│    └─────────────────────┘
                           └─────────────────────┘    
```

> **MCP Implementation:**
> - Uses the official Rust MCP SDK (rmcp) for protocol compliance
> - Communicates via standard input/output (stdio) transport
> - Provides MCP tools for querying and managing context
> - SQLite embedded database for efficient storage and querying

## 📝 Key Context Categories

### 1. **Business Domain Context**
- **What**: Domain-specific rules, terminology, and workflows
- **Why**: AI can't infer business logic from code structure
- **Example**: "User registration requires email verification AND phone number confirmation for premium accounts"

### 2. **Architectural Constraints**
- **What**: Specific architectural decisions and their rationale
- **Why**: AI needs to understand why certain patterns were chosen
- **Example**: "Use BLoC pattern for state management, avoid Provider in new features"

### 3. **Performance Context**
- **What**: Performance-sensitive areas and optimization requirements
- **Why**: AI needs to know where to prioritize performance
- **Example**: "Image loading must use progressive loading with max 2MB memory footprint"

### 4. **Security Context**
- **What**: Security policies and implementation requirements
- **Why**: AI needs explicit security guidance
- **Example**: "All API calls must include request signing, never log sensitive data"

### 5. **Integration Context**
- **What**: External system integrations and their quirks
- **Why**: AI can't know external API behaviors
- **Example**: "Payment API has 30-second timeout, implement retry with exponential backoff"

## 🔌 Simple HTTP REST API Design

```rust
// HTTP REST API endpoint: POST /context/query
// Request Body:
pub struct ContextQuery {
    pub project_id: String,
    pub feature_area: String,
    pub task_type: String, // 'implement', 'fix', 'optimize'
    pub components: Vec<String>,
}

// Response Body:
pub struct ContextResponse {
    pub business_rules: Vec<BusinessRule>,
    pub architectural_guidance: Vec<ArchitecturalDecision>,
    pub performance_requirements: Vec<PerformanceRequirement>,
    pub security_policies: Vec<SecurityPolicy>,
    pub conventions: Vec<ProjectConvention>,
}
```

The server exposes various HTTP endpoints for CRUD operations on context data:
- `GET /health` - Health check endpoint
- `GET /projects` - List all projects
- `POST /projects` - Create a new project
- `GET /projects/:project_id` - Get a specific project
- `DELETE /projects/:project_id` - Delete a project
- `GET /business_rules` - List all business rules
- `POST /business_rules` - Create a new business rule
- And many more resource-specific endpoints

## 📊 Context Input Methods

### 1. **Manual Entry Interface**
- Simple web UI for developers to input context
- Markdown support for documentation
- Template-based entry for common patterns

### 2. **Code Review Integration**
- Extract context from code review comments
- Capture architectural decisions from PR discussions
- Record performance issues and solutions

### 3. **Documentation Parsing**
- Extract business rules from existing documentation
- Parse architectural decision records (ADRs)
- Import security policies from compliance docs

## 🎯 Usage Examples

### HTTP Request 1: Implementing User Authentication
```
POST /context/query HTTP/1.1
Host: localhost:8080
Content-Type: application/json

{
  "project_id": "flutter-shop-app",
  "feature_area": "authentication",
  "task_type": "implement",
  "components": ["login", "signup", "password_reset"]
}
```

**HTTP Response**:
```json
{
  "business_rules": [
    {
      "id": "br-123",
      "rule_name": "Email Verification",
      "description": "Email verification required before account activation",
      "domain_area": "authentication"
    }
  ],
  "security_policies": [
    {
      "id": "sp-456",
      "policy_name": "Password Hashing",
      "requirements": "Use bcrypt with 12 rounds for password hashing"
    }
  ],
  "conventions": [
    {
      "id": "conv-789",
      "convention_type": "state_management",
      "convention_rule": "Use AuthenticationBloc for state management"
    }
  ],
  "performance_requirements": [
    {
      "id": "perf-101",
      "component_area": "login",
      "requirement_type": "response_time",
      "target_value": "< 2 seconds"
    }
  ],
  "architectural_guidance": []
}
```

### HTTP Request 2: Optimizing List Performance
```
POST /context/query HTTP/1.1
Host: localhost:8080
Content-Type: application/json

{
  "project_id": "flutter-shop-app",
  "feature_area": "user_interface",
  "task_type": "optimize",
  "components": ["product_list", "infinite_scroll"]
}
```

**HTTP Response**:
```json
{
  "performance_requirements": [
    {
      "id": "perf-202",
      "component_area": "product_list",
      "requirement_type": "performance",
      "target_value": "Handle 10,000+ items smoothly"
    }
  ],
  "architectural_guidance": [
    {
      "id": "arch-303",
      "decision_title": "List Implementation",
      "decision": "Use ListView.builder with pagination"
    }
  ],
  "conventions": [
    {
      "id": "conv-404",
      "convention_type": "user_experience",
      "convention_rule": "Implement pull-to-refresh pattern consistently"
    }
  ],
  "business_rules": [],
  "security_policies": []
}
```

## 🚀 Implementation Strategy

### Phase 1: Core Context HTTP Server (Completed)
1. Set up SQLite with context tables via Rusqlite
2. Implement REST API endpoints with Axum web framework
3. Build basic JSON API for context queries
4. Define core data models and schemas

### Phase 2: Integration (In Progress)
1. GitHub Copilot/AI agent HTTP integration
2. Simple web interface for manual context entry
3. Documentation import tools
4. Context validation and consistency checks

### Phase 3: Enhancement (Planned)
1. Context usage analytics
2. Automated context suggestions
3. Team collaboration features
4. Context versioning and history

## 📈 Success Metrics

- **Context Utilization**: How often AI agents query specific context
- **Code Quality**: Reduction in code review feedback on architectural issues
- **Development Speed**: Faster feature implementation with proper context
- **Consistency**: Improved adherence to project conventions and patterns

## 🔄 Maintenance

- **Regular Context Reviews**: Monthly review of context relevance
- **Context Updates**: Update context based on architectural changes
- **Team Feedback**: Gather feedback on context usefulness
- **Context Cleanup**: Remove outdated or unused context entries

