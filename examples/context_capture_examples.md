# Context Capture Examples

These examples show how to capture and store different types of context using the MCP Context Server.

---

## 1. Capturing a New Feature Requirement
**User:**
> I want to add a login page with email and password authentication.

**AI Agent:**
- Extracts requirement:
  - Feature: Login page
  - Auth method: Email & password
- Saves to MCP server as a new requirement.

---

## 2. Capturing an Architectural Decision
**User:**
> Let's use SQLite for local storage because it's lightweight and easy to set up.

**AI Agent:**
- Extracts decision:
  - Use SQLite for storage
  - Reason: Lightweight, easy setup
- Saves to MCP server as an architectural decision.

---

## 3. Capturing a Business Rule
**User:**
> Users must reset their password every 90 days.

**AI Agent:**
- Extracts business rule:
  - Password reset interval: 90 days
- Saves to MCP server as a business rule.

---

## 4. Prompting for Missing Context
**User:**
> Generate the registration form code.

**AI Agent:**
- Notices no requirements for registration form in context.
- Prompts user: "Can you describe the fields and validation rules for registration?"
- Saves new context when provided.

---

Use these patterns to ensure all relevant context is captured and available for future development!
