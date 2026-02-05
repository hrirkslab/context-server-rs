# Build & Utility Scripts

## Cleanup Scripts

### cleanup.py (⭐ Recommended)
```bash
python3 cleanup.py
```
- **Use:** Primary cleanup tool, fully automated
- **Time:** 30 seconds
- **Output:** Colored, clear feedback
- **Cross-platform:** Yes

### cleanup-project.sh
```bash
bash cleanup-project.sh
```
- **Use:** Alternative with detailed logging
- **Time:** 1 minute
- **Output:** Detailed progress
- **Platform:** Unix/Linux/Mac

### git-cleanup.sh
```bash
bash git-cleanup.sh
```
- **Use:** Git-based cleanup (tracked removals)
- **Time:** 3-5 minutes
- **Output:** Git commits changes
- **Platform:** Unix/Linux/Mac

---

## Other Build Scripts

- **build-extension.sh** - Build VS Code extension (Linux/Mac)
- **build-extension.ps1** - Build VS Code extension (Windows)
- **run_ide_tests.sh** - Run IDE integration tests
- **run_ide_tests.ps1** - Run IDE tests (Windows)
- **test_mcp.sh** - Test MCP server functionality

---

## Quick Start

```bash
# 1. Clean up project (required)
python3 cleanup.py

# 2. Verify build
cargo build --release && cargo test --all

# 3. Done! ✅
```

See [START_CLEANUP.md](../START_CLEANUP.md) for full details.
