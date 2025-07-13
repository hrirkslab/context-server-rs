# Context Engineering Principles for AI Projects

Context engineering is the practice of systematically capturing, organizing, and serving all relevant information (context) needed for effective AI and LLM-powered workflows.

## Why Context Engineering?
- **LLMs are only as good as their context**: Without the right context, even the best models produce generic or incorrect results.
- **Repeatability**: Saving context (requirements, decisions, conventions) ensures future queries and code generations are consistent and relevant.
- **Collaboration**: Shared context enables teams and tools to work together seamlessly.

## Key Principles
1. **Centralize Context**: Store all project context (requirements, business rules, architecture, etc.) in a single, queryable source (like the MCP server with SQLite).
2. **Keep Context Up to Date**: Every new requirement, decision, or convention should be saved immediately.
3. **Make Context Discoverable**: Use structured schemas and protocols (like MCP) so tools and agents can find and use context automatically.
4. **Context-Driven Generation**: Always use the latest context for code generation, explanations, and decisions.
5. **Feedback Loop**: When context is missing or unclear, prompt users to provide it and save it for future use.

## How MCP Context Server Helps
- Provides a standard protocol for context exchange.
- Stores all context in a local, portable SQLite database.
- Enables AI agents and tools to read and write context programmatically.

## Example Workflow
1. User describes a new feature.
2. AI agent extracts requirements and saves them to the MCP server.
3. Future code generations use the saved requirements, ensuring consistency.
4. If a decision or rule is missing, the agent asks the user and updates the context.

---

For more, see `system_prompt_template.txt` and the main README.
