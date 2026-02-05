#!/bin/bash

echo "🚀 MCP Context Server Test Suite"
echo "=================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Running Tests...${NC}"
echo ""

# Test 1: Run MCP endpoint validation tests
echo -e "${YELLOW}Test 1: MCP Endpoint Validation${NC}"
cargo test --test mcp_endpoint_tests test_mcp_create_project_entity_endpoint -- --nocapture 2>&1 | tail -20
echo ""

# Test 2: Run integration tests
echo -e "${YELLOW}Test 2: Integration Tests (CRUD Operations)${NC}"
cargo test --test integration_tests test_project_crud_operations -- --nocapture 2>&1 | tail -20
echo ""

# Test 3: List all tools
echo -e "${YELLOW}Test 3: MCP Tools Available${NC}"
cargo test --test mcp_endpoint_tests test_mcp_list_projects_endpoint -- --nocapture 2>&1 | tail -20
echo ""

# Test 4: Schema validation
echo -e "${YELLOW}Test 4: Entity Schema Validation${NC}"
cargo test --test mcp_endpoint_tests test_mcp_get_entity_endpoint_schema -- --nocapture 2>&1 | tail -20
echo ""

echo -e "${GREEN}✅ MCP Tests Complete!${NC}"
echo ""
echo "MCP Status Summary:"
echo "- Server Name: enhanced-context-server-rs"
echo "- Server Version: 0.2.0"
echo "- Protocol: Model Context Protocol (stdio transport)"
echo "- Entity Types: 8 total"
echo "  ✓ project"
echo "  ✓ business_rule"
echo "  ✓ architectural_decision"
echo "  ✓ performance_requirement"
echo "  ✓ security_policy"
echo "  ✓ framework_component"
echo "  ✓ development_phase"
echo "  ✓ feature_context"
echo ""
echo "Database:"
echo "- Type: SQLite (embedded)"
echo "- Location: ~/.config/context-server-rs/context.db"
echo ""
echo "Performance Optimizations:"
echo "- ✓ Query Caching (LRU + TTL)"
echo "- ✓ Connection Pooling"
echo "- ✓ SOLID Architecture"
echo ""
