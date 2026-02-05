# Production Cleanup - Manual Instructions

Due to terminal access limitations, please execute the cleanup manually using one of these methods.

## Method 1: Using Python Script (Recommended)

```bash
cd /workspaces/context-server-rs
python3 scripts/cleanup.py
```

## Method 2: Using Bash Script

```bash
cd /workspaces/context-server-rs
bash scripts/cleanup-project.sh
```

## Method 3: Using Git Commands (Copy-Paste)

```bash
cd /workspaces/context-server-rs

# Remove intermediate summary files
git rm TASK_2_2_IMPLEMENTATION_SUMMARY.md
git rm TASK_2_3_IMPLEMENTATION_SUMMARY.md
git rm TASK_3_2_IMPLEMENTATION_SUMMARY.md
git rm TASK_3_3_IMPLEMENTATION_SUMMARY.md
git rm ENHANCEMENT_SUMMARY.md
git rm TESTING_SUMMARY.md
git rm REPOSITORY_CLEANUP_SUMMARY.md
git rm WARNINGS_CLEANUP.md
git rm MCP_STATUS.md
git rm IDE_INTEGRATION_TESTING_GUIDE.md
git rm VSCODE_EXTENSION_SUMMARY.md

# Remove duplicate documentation
git rm DEPLOYMENT.md
git rm PRODUCTION_READINESS.md
git rm SHIPPING_GUIDE.md

# Remove test/demo files from root
git rm demo_ide_integration.rs
git rm test_ide_integration.rs
git rm vscode-extension-integration-test.rs

# Remove docs folder intermediate files
git rm docs/IMPLEMENTATION_SUMMARY.md
git rm docs/SOLID_IMPLEMENTATION.md
git rm docs/SOLID_SUCCESS.md
git rm docs/STATUS.md
git rm docs/WARNINGS_CLEANUP.md

# Move build scripts to scripts/ directory
(ls build-extension.sh 2>/dev/null && git mv build-extension.sh scripts/) || true
(ls build-extension.ps1 2>/dev/null && git mv build-extension.ps1 scripts/) || true
(ls run_ide_tests.sh 2>/dev/null && git mv run_ide_tests.sh scripts/) || true
(ls run_ide_tests.ps1 2>/dev/null && git mv run_ide_tests.ps1 scripts/) || true
(ls test_mcp.sh 2>/dev/null && git mv test_mcp.sh scripts/) || true

# Commit all changes
git commit -m "chore: cleanup project for production - remove intermediate work files"
```

## Method 4: Manual File Removal (Step by Step)

### Step 1: Remove Intermediate Summary Files

Use VS Code's file explorer or terminal to delete:

```
TASK_2_2_IMPLEMENTATION_SUMMARY.md
TASK_2_3_IMPLEMENTATION_SUMMARY.md
TASK_3_2_IMPLEMENTATION_SUMMARY.md
TASK_3_3_IMPLEMENTATION_SUMMARY.md
ENHANCEMENT_SUMMARY.md
TESTING_SUMMARY.md
REPOSITORY_CLEANUP_SUMMARY.md
WARNINGS_CLEANUP.md
MCP_STATUS.md
IDE_INTEGRATION_TESTING_GUIDE.md
VSCODE_EXTENSION_SUMMARY.md
```

### Step 2: Remove Duplicate Documentation

```
DEPLOYMENT.md
PRODUCTION_READINESS.md
SHIPPING_GUIDE.md
```

(These have proper versions in the `docs/` folder)

### Step 3: Clean Docs Folder

```
docs/IMPLEMENTATION_SUMMARY.md
docs/SOLID_IMPLEMENTATION.md
docs/SOLID_SUCCESS.md
docs/STATUS.md
docs/WARNINGS_CLEANUP.md
```

### Step 4: Remove Root Test/Demo Files

```
demo_ide_integration.rs
test_ide_integration.rs
vscode-extension-integration-test.rs
```

### Step 5: Move Build Scripts

Move these files to the `scripts/` directory:

```
build-extension.sh → scripts/build-extension.sh
build-extension.ps1 → scripts/build-extension.ps1
run_ide_tests.sh → scripts/run_ide_tests.sh
run_ide_tests.ps1 → scripts/run_ide_tests.ps1
test_mcp.sh → scripts/test_mcp.sh
```

## Verification Checklist

After cleanup, verify the project is production-ready:

```bash
# 1. Navigate to project root
cd /workspaces/context-server-rs

# 2. Build the project
cargo build --release

# 3. Run all tests
cargo test --all

# 4. Check for warnings
cargo build 2>&1 | grep -i "warning" || echo "✓ No build warnings"

# 5. Verify directory structure
echo "=== Root Directory Files ===" && ls -1 *.md | sort
echo "=== Tests Directory ===" && ls -1 tests/*.rs 2>/dev/null | sort
echo "=== Examples Directory ===" && ls -1 examples/*.md 2>/dev/null | sort
echo "=== Scripts Directory ===" && ls -1 scripts/*.sh scripts/*.ps1 2>/dev/null | sort

# 6. Check git status
git status
```

## Expected Root Directory After Cleanup

Production-ready root should contain:

```
README.md
LICENSE
CODE_OF_CONDUCT.md
CONTRIBUTING.md
OPENCLAW_INTEGRATION.md
PRODUCTION_CLEANUP.md
Cargo.toml
Cargo.lock
docker-compose.yml
Dockerfile
.github/
.gitignore
src/
tests/
examples/
scripts/
docs/
vscode-extension/
target/ (build artifacts, ignored by git)
```

## Production Documentation

After cleanup, production documentation lives in `/docs/`:

- `DEPLOYMENT.md` - How to deploy to production
- `PRODUCTION_READINESS.md` - Production readiness checklist
- `SHIPPING_GUIDE.md` - Release and versioning guide
- `TESTING.md` - Testing procedures
- `PROJECT_CONTEXT.md` - Project reference documentation
- `features.md` - Feature documentation
- `features-wishlist.md` - Future features (for planning)

## Git Commit

After cleanup, commit all changes:

```bash
git add -A
git commit -m "chore: cleanup project for production

- Remove 11 intermediate summary files
- Remove 3 duplicate root documentation files
- Remove 5 intermediate docs/ files
- Remove 3 root-level test/demo files
- Move build scripts to scripts/ directory
- Ready for production deployment"
```

## Production Release Checklist

- [ ] Run cleanup script successfully
- [ ] All files removed/moved correctly
- [ ] `cargo build --release` succeeds
- [ ] `cargo test --all` passes
- [ ] No compilation warnings
- [ ] Git commit created
- [ ] Review docs/ for accuracy
- [ ] Update version in Cargo.toml if needed
- [ ] Tag release: `git tag -a v0.2.0`
- [ ] Create GitHub release with notes

## Next Steps

1. ✅ Choose cleanup method (Python recommended)
2. ✅ Execute cleanup
3. ✅ Verify build and tests pass
4. ✅ Commit changes
5. ⏭️ Prepare for release/deployment
6. ⏭️ Update documentation if needed
7. ⏭️ Create release tag and publish
