# Quick Cleanup Reference

## Run Cleanup (30 seconds)
```bash
python3 scripts/cleanup.py
```

## Verify (5 minutes)
```bash
cargo build --release && cargo test --all
```

## Commit
```bash
git add -A && git commit -m "chore: cleanup for production"
```

---

## What Gets Removed (22 files)
- TASK_*.md (4), ENHANCEMENT_SUMMARY.md, TESTING_SUMMARY.md
- REPOSITORY_CLEANUP_SUMMARY.md, WARNINGS_CLEANUP.md
- MCP_STATUS.md, IDE_INTEGRATION_TESTING_GUIDE.md
- VSCODE_EXTENSION_SUMMARY.md
- DEPLOYMENT.md, PRODUCTION_READINESS.md, SHIPPING_GUIDE.md (duplicates)
- docs/IMPLEMENTATION_SUMMARY.md, docs/SOLID_*.md (3), docs/STATUS.md, docs/WARNINGS_CLEANUP.md
- demo_ide_integration.rs, test_ide_integration.rs, vscode-extension-integration-test.rs

## What Gets Moved (5 files)
- build-extension.sh, build-extension.ps1 → scripts/
- run_ide_tests.sh, run_ide_tests.ps1, test_mcp.sh → scripts/

---

## Backup Command (all-in-one)
```bash
python3 scripts/cleanup.py && \
cargo build --release && \
cargo test --all && \
echo "✅ Complete!"
```

## Troubleshooting
- No Python? Use `bash scripts/cleanup-project.sh`
- Build fails? Run `cargo clean && cargo build --release`
- Permission denied? Run `chmod +x scripts/cleanup.py`

**See [START_CLEANUP.md](START_CLEANUP.md) for full guide.**
