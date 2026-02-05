# Build and Development Scripts

This directory contains utility scripts for building, testing, and managing the Context Server RS project.

## Available Scripts

### Cleanup Scripts

#### `cleanup.py` (Recommended)
Python-based cleanup script for production preparation.

```bash
python3 cleanup.py
```

**What it does:**
- Removes 11 intermediate summary files
- Removes 3 duplicate documentation files
- Cleans up 5 docs/ folder intermediate files
- Removes 3 root-level test/demo files
- Organizes build scripts into scripts/ directory
- Provides colored output for easy verification

**Requirements:** Python 3.6+

#### `cleanup-project.sh`
Bash script for cleanup with detailed output.

```bash
bash cleanup-project.sh
```

**Requirements:** Bash shell

#### `git-cleanup.sh`
Git-based cleanup using `git rm` for tracking deletions.

```bash
bash git-cleanup.sh
```

**Benefits:** All deletions are tracked by git, easier for code review.

**Requirements:** Git, bash shell

### Build Scripts

#### `build-extension.sh`
Build the VS Code extension.

```bash
bash build-extension.sh
```

**What it does:**
1. Navigates to vscode-extension/
2. Installs npm dependencies
3. Compiles TypeScript
4. Packages the extension

**Requirements:** Node.js, npm, TypeScript

#### `build-extension.ps1`
PowerShell version of build-extension.sh for Windows.

```powershell
./build-extension.ps1
```

**Requirements:** PowerShell, Node.js, npm

### Test Scripts

#### `run_ide_tests.sh`
Run IDE integration tests on Unix-like systems.

```bash
bash run_ide_tests.sh
```

**What it does:**
- Runs all integration tests
- Generates test reports
- Verifies IDE plugin functionality

**Requirements:** Rust toolchain

#### `run_ide_tests.ps1`
PowerShell version for Windows systems.

```powershell
./run_ide_tests.ps1
```

**Requirements:** PowerShell, Rust toolchain

#### `test_mcp.sh`
Test MCP server functionality.

```bash
bash test_mcp.sh
```

**What it does:**
- Starts the MCP server
- Runs protocol validation tests
- Verifies server responses

**Requirements:** Rust toolchain, Tokio

## Production Preparation Workflow

```bash
# 1. Clean up the project
python3 scripts/cleanup.py

# 2. Build release artifacts
cargo build --release

# 3. Run full test suite
cargo test --all

# 4. Verify no warnings
cargo build --release 2>&1 | grep -i warning || echo "✓ All clear"

# 5. Test MCP functionality
bash scripts/test_mcp.sh

# 6. Build and test VS Code extension  
bash scripts/build-extension.sh

# 7. Commit and tag
git add -A
git commit -m "chore: production release cleanup"
git tag -a v0.2.0 -m "Release version 0.2.0"
```

## Troubleshooting

### Python script errors
- Ensure Python 3.6+ is installed: `python3 --version`
- Set execute permissions: `chmod +x cleanup.py`

### Bash script errors
- Set execute permissions: `chmod +x *.sh`
- Use `bash script.sh` if shebang issues occur

### Build issues
- Ensure Rust toolchain is installed: `rustc --version`
- Update Rust: `rustup update`
- Clean build: `cargo clean && cargo build --release`

### Test failures
- Check dependencies: `cargo build --all`
- Run with verbose output: `RUST_LOG=debug cargo test`
- Review test output in `target/`

## Script Maintenance

When adding new scripts:
1. Add shebang line: `#!/bin/bash` or `#!/usr/bin/env python3`
2. Add documentation at top
3. Make executable: `chmod +x scriptname.sh`
4. Update this README
5. Test script before committing

## Environment Variables

```bash
# Rust compilation
RUST_LOG=debug          # Enable debug logging
RUST_BACKTRACE=full     # Full backtrace on panic

# Build options
PROFILE=release         # Build profile (debug/release)
JOBS=4                  # Parallel build jobs
```

## Related Documentation

- See [CLEANUP_INSTRUCTIONS.md](../CLEANUP_INSTRUCTIONS.md) for production cleanup
- See [PRODUCTION_CLEANUP.md](../PRODUCTION_CLEANUP.md) for detailed cleanup information
- See [docs/DEPLOYMENT.md](../docs/DEPLOYMENT.md) for deployment guide
