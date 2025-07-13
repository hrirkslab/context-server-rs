# Context Query Examples

These examples show how AI agents or tools can query the MCP Context Server for relevant context.

---

## 1. Querying for Feature Requirements
**Prompt:**
> What are the requirements for the login feature?

**Agent Action:**
- Query MCP server for requirements tagged "login".
- Respond with the stored requirements.

---

## 2. Querying for Architectural Decisions
**Prompt:**
> What database are we using and why?

**Agent Action:**
- Query MCP server for architectural decisions related to storage.
- Respond: "We use SQLite because it's lightweight and easy to set up."

---

## 3. Querying for Business Rules
**Prompt:**
> Are there any password policies?

**Agent Action:**
- Query MCP server for business rules related to passwords.
- Respond: "Users must reset their password every 90 days."

---

## 4. Querying for Missing Context
**Prompt:**
> Generate code for the registration form.

**Agent Action:**
- Query MCP server for requirements for registration form.
- If not found, prompt user to provide details, then save and use them.

---

These examples help ensure the right context is always available for AI-driven development.
