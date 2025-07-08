# MCP Server Integration Guide for GitHub Copilot Agent and Flutter Projects

This guide explains how to use the Rust-based Model-Context-Provider (MCP) server as a context provider for AI agents (such as GitHub Copilot Agent) and for integration with developer tools or apps (including Flutter).

## 1. Run the MCP Server

1. Build and start the MCP server:
   ```sh
   cargo run --release
   ```
   The server will start at `http://127.0.0.1:8080/` by default.

2. Ensure the server is accessible from your Copilot agent or app (e.g., run both on the same machine or expose the server on your network).

## 2. Integrate with GitHub Copilot Agent

The MCP server exposes a REST API for querying project context (business rules, architectural decisions, conventions, etc.).

- Configure your Copilot agent to send HTTP requests to the MCP server endpoints (e.g., `/context/query`).
- Use the API to fetch relevant context for code generation, review, or suggestions.
- Example request payload:
  ```json
  {
    "feature_area": "authentication",
    "task_type": "implement",
    "components": ["login", "signup", "password_reset"]
  }
  ```
- The response will include business rules, security policies, conventions, and more, tailored to the query.

## 3. Connect from Flutter (Optional)

Use the `http` package in Flutter to make REST API calls to the MCP server.

### Example: Querying Context

Add `http` to your `pubspec.yaml`:
```yaml
dependencies:
  http: ^1.2.0
```

Sample Dart code to query the MCP server:
```dart
import 'dart:convert';
import 'package:http/http.dart' as http;

Future<void> fetchContext() async {
  final response = await http.get(Uri.parse('http://127.0.0.1:8080/health'));
  if (response.statusCode == 200) {
    print('MCP Server is healthy: ${response.body}');
  } else {
    print('Failed to reach MCP Server: ${response.statusCode}');
  }
}
```

For more advanced queries, use the context query API (see server docs for endpoint details).

## 4. Use Cases
- GitHub Copilot Agent: Fetch project context to improve code suggestions and ensure adherence to business rules and conventions.
- Flutter or other apps: Fetch business rules, architectural decisions, and conventions for code generation or review.
- Integrate with AI tools in your workflow.
- Automate context-aware code suggestions.

## 5. API Endpoints

### Health Check
- **GET /health**
  - Returns: `"OK"` if the server is running.

### Business Rules
- **GET /business_rules**
  - Returns: List of all business rules in the database.
- **POST /business_rules**
  - Body: JSON object matching the `BusinessRule` struct
  - Adds a new business rule to the database.

### Context Query
- **POST /context/query**
  - Body: JSON object matching the `ContextQuery` struct
  - Returns: `ContextResponse` with relevant business rules, architectural guidance, performance requirements, security policies, and conventions (currently returns empty arrays; implement your query logic as needed).

#### Example: Context Query Request
```json
{
  "feature_area": "authentication",
  "task_type": "implement",
  "components": ["login", "signup", "password_reset"]
}
```

#### Example: Context Query Response
```json
{
  "business_rules": [],
  "architectural_guidance": [],
  "performance_requirements": [],
  "security_policies": [],
  "conventions": []
}
```

---

For more details, see the MCP server API documentation or contact the maintainers.
