#!/bin/bash
# Quick Production Cleanup - One-liner commands
# Run from project root: cd /workspaces/context-server-rs

# Remove intermediate summary files
git rm --cached --quiet TASK_2_2_IMPLEMENTATION_SUMMARY.md 2>/dev/null || true
git rm --cached --quiet TASK_2_3_IMPLEMENTATION_SUMMARY.md 2>/dev/null || true
git rm --cached --quiet TASK_3_2_IMPLEMENTATION_SUMMARY.md 2>/dev/null || true
git rm --cached --quiet TASK_3_3_IMPLEMENTATION_SUMMARY.md 2>/dev/null || true
git rm --cached --quiet ENHANCEMENT_SUMMARY.md 2>/dev/null || true
git rm --cached --quiet TESTING_SUMMARY.md 2>/dev/null || true
git rm --cached --quiet REPOSITORY_CLEANUP_SUMMARY.md 2>/dev/null || true
git rm --cached --quiet WARNINGS_CLEANUP.md 2>/dev/null || true
git rm --cached --quiet MCP_STATUS.md 2>/dev/null || true
git rm --cached --quiet IDE_INTEGRATION_TESTING_GUIDE.md 2>/dev/null || true
git rm --cached --quiet VSCODE_EXTENSION_SUMMARY.md 2>/dev/null || true

# Remove duplicate documentation
git rm --cached --quiet DEPLOYMENT.md 2>/dev/null || true
git rm --cached --quiet PRODUCTION_READINESS.md 2>/dev/null || true
git rm --cached --quiet SHIPPING_GUIDE.md 2>/dev/null || true

# Remove root-level test/demo files
git rm --cached --quiet demo_ide_integration.rs 2>/dev/null || true
git rm --cached --quiet test_ide_integration.rs 2>/dev/null || true
git rm --cached --quiet vscode-extension-integration-test.rs 2>/dev/null || true

# Remove doc folder intermediate files
git rm --cached --quiet docs/IMPLEMENTATION_SUMMARY.md 2>/dev/null || true
git rm --cached --quiet docs/SOLID_IMPLEMENTATION.md 2>/dev/null || true
git rm --cached --quiet docs/SOLID_SUCCESS.md 2>/dev/null || true
git rm --cached --quiet docs/STATUS.md 2>/dev/null || true
git rm --cached --quiet docs/WARNINGS_CLEANUP.md 2>/dev/null || true

# Commit cleanup
git commit -m "chore: cleanup project for production - remove intermediate files and organize structure"

echo "✅ Production cleanup complete!"
