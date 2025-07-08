# 🧠 Minimal Context Server for AI Code Generation

## 📘 Project Overview

A **lightweight, focused context server** that captures and maintains only the essential project information that AI agents (like GitHub Copilot) cannot automatically discover or infer. This server provides **curated, high-value context** that enables AI agents to generate production-quality code without duplicating built-in AI capabilities.

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

## 🏗️ Minimal Architecture

```
┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐
│   Manual Input      │    │   Context Server    │    │   AI Agent Query    │
│   (Developers)      │───▶│   (SQLite, embedded)│───▶│   (GitHub Copilot)  │
└─────────────────────┘    └─────────────────────┘    └─────────────────────┘
```

> **Note:**
> - The context server now uses an embedded SQLite database (via the `rusqlite` crate) for all data storage and querying. This removes the need for an external database server and simplifies deployment.
> - SQLite supports efficient search, indexing, and navigation of structured data, making it a better choice than file-based storage with `serde` for this use case.

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

## 🔌 Simple API Design

```rust
// Minimal API for AI agents to query context
pub struct ContextQuery {
    pub feature_area: String,
    pub task_type: String, // 'implement', 'fix', 'optimize'
    pub components: Vec<String>,
}

pub struct ContextResponse {
    pub business_rules: Vec<BusinessRule>,
    pub architectural_guidance: Vec<ArchitecturalDecision>,
    pub performance_requirements: Vec<PerformanceRequirement>,
    pub security_policies: Vec<SecurityPolicy>,
    pub conventions: Vec<ProjectConvention>,
}
```

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

### Query 1: Implementing User Authentication
```json
{
  "feature_area": "authentication",
  "task_type": "implement",
  "components": ["login", "signup", "password_reset"]
}
```

**Response Context**:
- Business rule: "Email verification required before account activation"
- Security policy: "Use bcrypt with 12 rounds for password hashing"
- Convention: "Use AuthenticationBloc for state management"
- Performance: "Login should complete within 2 seconds"

### Query 2: Optimizing List Performance
```json
{
  "feature_area": "user_interface",
  "task_type": "optimize",
  "components": ["product_list", "infinite_scroll"]
}
```

**Response Context**:
- Performance requirement: "List should handle 10,000+ items smoothly"
- Architectural decision: "Use ListView.builder with pagination"
- Convention: "Implement pull-to-refresh pattern consistently"

## 🚀 Implementation Strategy

### Phase 1: Core Context Database
1. Set up SQLite with context tables
2. Create simple web interface for manual entry
3. Build basic API for context queries
4. Implement context templates for common patterns

### Phase 2: Integration
1. GitHub Copilot/AI agent integration
2. Code review context extraction
3. Documentation import tools
4. Context validation and consistency checks

### Phase 3: Enhancement
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

