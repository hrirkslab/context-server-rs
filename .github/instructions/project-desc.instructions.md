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

## 🔌 MCP Tools and Resources Design

The server exposes MCP tools that clients can call to interact with the context data:

### Available MCP Tools

1. **query_context** - Query project context based on feature area and task type
   ```json
   {
     "project_id": "string",
     "feature_area": "string", 
     "task_type": "implement|fix|optimize",
     "components": ["string"]
   }
   ```

2. **list_projects** - List all available projects
   ```json
   {}
   ```

3. **create_project** - Create a new project
   ```json
   {
     "name": "string",
     "description": "string (optional)",
     "repository_url": "string (optional)"
   }
   ```

### MCP Protocol Benefits

- **Standardized Communication**: Uses the official MCP specification for LLM-context provider communication
- **Tool Discovery**: Clients can automatically discover available tools
- **Type Safety**: JSON schemas define tool parameters and validation
- **Error Handling**: Standard MCP error codes and messages
- **Transport Flexibility**: Supports stdio, SSE, HTTP streaming transports

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

### MCP Tool Call 1: Implementing User Authentication
**Tool**: `query_context`
**Arguments**:
```json
{
  "project_id": "flutter-shop-app",
  "feature_area": "authentication",
  "task_type": "implement",
  "components": ["login", "signup", "password_reset"]
}
```

**MCP Tool Response**:
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
  "project_conventions": [
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
  "architectural_decisions": []
}
```

### MCP Tool Call 2: Optimizing List Performance
**Tool**: `query_context`
**Arguments**:
```json
{
  "project_id": "flutter-shop-app",
  "feature_area": "user_interface",
  "task_type": "optimize",
  "components": ["product_list", "infinite_scroll"]
}
```

**MCP Tool Response**:
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
  "architectural_decisions": [
    {
      "id": "arch-303",
      "decision_title": "List Implementation",
      "decision": "Use ListView.builder with pagination"
    }
  ],
  "project_conventions": [
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

### Phase 1: Core MCP Server (Completed)
1. ✅ Set up SQLite with context tables via Rusqlite
2. ✅ Implement MCP ServerHandler with rmcp SDK
3. ✅ Build MCP tools for context queries
4. ✅ Define core data models and schemas
5. ✅ Implement stdio transport for MCP communication

### Phase 2: Enhanced MCP Integration (In Progress)
1. MCP client integration testing (Claude Desktop, Cursor)
2. Additional MCP tools for CRUD operations
3. MCP resources for browsing context
4. Context validation and consistency checks

### Phase 3: Advanced Features (Planned)
1. SSE transport support for web-based MCP clients
2. Context usage analytics via MCP tools
3. Automated context suggestions
4. Team collaboration features
5. Context versioning and history

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

