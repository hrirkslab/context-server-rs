#!/bin/bash
# Production Cleanup Script for Context Server RS
# Removes intermediate work files and organizes the project structure

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "🧹 Starting Project Cleanup..."
echo "======================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Count removals
REMOVED_COUNT=0

# Function to safely remove files
remove_file() {
    if [ -f "$1" ]; then
        rm -f "$1"
        echo -e "${GREEN}✓${NC} Removed: $1"
        ((REMOVED_COUNT++))
    fi
}

echo -e "\n${YELLOW}1. Removing intermediate summary files...${NC}"
remove_file "TASK_2_2_IMPLEMENTATION_SUMMARY.md"
remove_file "TASK_2_3_IMPLEMENTATION_SUMMARY.md"
remove_file "TASK_3_2_IMPLEMENTATION_SUMMARY.md"
remove_file "TASK_3_3_IMPLEMENTATION_SUMMARY.md"
remove_file "ENHANCEMENT_SUMMARY.md"
remove_file "TESTING_SUMMARY.md"
remove_file "REPOSITORY_CLEANUP_SUMMARY.md"
remove_file "WARNINGS_CLEANUP.md"
remove_file "MCP_STATUS.md"
remove_file "IDE_INTEGRATION_TESTING_GUIDE.md"
remove_file "VSCODE_EXTENSION_SUMMARY.md"

echo -e "\n${YELLOW}2. Removing duplicate documentation from root...${NC}"
# Keep the originals in docs/ instead
remove_file "DEPLOYMENT.md"
remove_file "PRODUCTION_READINESS.md"
remove_file "SHIPPING_GUIDE.md"

echo -e "\n${YELLOW}3. Cleaning docs/ directory (removing intermediate summaries)...${NC}"
if [ -d "docs" ]; then
    remove_file "docs/IMPLEMENTATION_SUMMARY.md"
    remove_file "docs/SOLID_IMPLEMENTATION.md"
    remove_file "docs/SOLID_SUCCESS.md"
    remove_file "docs/STATUS.md"
    remove_file "docs/WARNINGS_CLEANUP.md"
fi

echo -e "\n${YELLOW}4. Removing test/demo files from root (moving to appropriate directories)...${NC}"

# demo_ide_integration.rs is a demo file, move to examples or remove
if [ -f "demo_ide_integration.rs" ]; then
    rm -f "demo_ide_integration.rs"
    echo -e "${GREEN}✓${NC} Removed: demo_ide_integration.rs (demo file, use examples/ instead)"
    ((REMOVED_COUNT++))
fi

# Move test files to proper locations if they exist
if [ -f "test_ide_integration.rs" ]; then
    rm -f "test_ide_integration.rs"
    echo -e "${GREEN}✓${NC} Removed: test_ide_integration.rs (use tests/ directory instead)"
    ((REMOVED_COUNT++))
fi

if [ -f "vscode-extension-integration-test.rs" ]; then
    rm -f "vscode-extension-integration-test.rs"
    echo -e "${GREEN}✓${NC} Removed: vscode-extension-integration-test.rs (belongs in vscode-extension/)"
    ((REMOVED_COUNT++))
fi

echo -e "\n${YELLOW}5. Organizing build scripts...${NC}"

# Create scripts directory if it doesn't exist
mkdir -p scripts

# Move scripts (if they exist at root level, which shouldn't be typical)
if [ -f "build-extension.sh" ] && [ ! -f "scripts/build-extension.sh" ]; then
    mv build-extension.sh scripts/
    echo -e "${GREEN}✓${NC} Moved: build-extension.sh → scripts/"
fi

if [ -f "build-extension.ps1" ] && [ ! -f "scripts/build-extension.ps1" ]; then
    mv build-extension.ps1 scripts/
    echo -e "${GREEN}✓${NC} Moved: build-extension.ps1 → scripts/"
fi

if [ -f "run_ide_tests.sh" ] && [ ! -f "scripts/run_ide_tests.sh" ]; then
    mv run_ide_tests.sh scripts/
    echo -e "${GREEN}✓${NC} Moved: run_ide_tests.sh → scripts/"
fi

if [ -f "run_ide_tests.ps1" ] && [ ! -f "scripts/run_ide_tests.ps1" ]; then
    mv run_ide_tests.ps1 scripts/
    echo -e "${GREEN}✓${NC} Moved: run_ide_tests.ps1 → scripts/"
fi

if [ -f "test_mcp.sh" ] && [ ! -f "scripts/test_mcp.sh" ]; then
    mv test_mcp.sh scripts/
    echo -e "${GREEN}✓${NC} Moved: test_mcp.sh → scripts/"
fi

echo -e "\n${YELLOW}6. Verifying project structure...${NC}"
echo -e "Root directory files (production):"
ls -1 *.md 2>/dev/null | head -20 || echo -e "${GREEN}✓${NC} Clean root directory"

echo -e "\n${GREEN}======================================"
echo "✅ Project Cleanup Complete!"
echo "=====================================${NC}"
echo -e "\nFiles removed: ${REMOVED_COUNT}"
echo -e "\n${YELLOW}Next steps:${NC}"
echo "1. Run 'cargo test' to ensure all tests pass"
echo "2. Review docs/ for production documentation"
echo "3. Update .gitignore if needed"
echo "4. Commit changes: git add -A && git commit -m 'chore: cleanup project for production'"
