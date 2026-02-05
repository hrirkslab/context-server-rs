#!/usr/bin/env python3
"""
Production Cleanup Script for Context Server RS
Removes intermediate work files and organizes project structure
"""

import os
import sys
from pathlib import Path
from typing import List, Set

# Colors for terminal output
class Colors:
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    RESET = '\033[0m'
    BOLD = '\033[1m'

def print_status(message: str, status: str = "info"):
    """Print colored status messages"""
    if status == "success":
        print(f"{Colors.GREEN}✓{Colors.RESET} {message}")
    elif status == "warning":
        print(f"{Colors.YELLOW}⚠{Colors.RESET} {message}")
    elif status == "error":
        print(f"{Colors.RED}✗{Colors.RESET} {message}")
    elif status == "section":
        print(f"\n{Colors.YELLOW}{message}{Colors.RESET}")
    else:
        print(f"  {message}")

def remove_files(project_root: Path, files: List[str]) -> int:
    """Remove specified files and return count"""
    removed = 0
    for file in files:
        file_path = project_root / file
        if file_path.exists():
            try:
                file_path.unlink()
                print_status(f"Removed: {file}", "success")
                removed += 1
            except Exception as e:
                print_status(f"Failed to remove {file}: {e}", "error")
        else:
            print_status(f"File not found (already removed): {file}", "warning")
    return removed

def move_files(project_root: Path, files: dict) -> int:
    """Move files from source to destination"""
    moved = 0
    for src, dst in files.items():
        src_path = project_root / src
        dst_path = project_root / dst
        
        if src_path.exists():
            try:
                # Create destination directory if needed
                dst_path.parent.mkdir(parents=True, exist_ok=True)
                
                # Only move if destination doesn't exist
                if not dst_path.exists():
                    src_path.rename(dst_path)
                    print_status(f"Moved: {src} → {dst}", "success")
                    moved += 1
                else:
                    print_status(f"Destination exists, skipping: {dst}", "warning")
                    src_path.unlink()  # Remove source since destination exists
            except Exception as e:
                print_status(f"Failed to move {src}: {e}", "error")
    return moved

def main():
    """Main cleanup function"""
    project_root = Path(__file__).parent.parent
    os.chdir(project_root)
    
    print(f"\n{Colors.BOLD}🧹 Production Cleanup - Context Server RS{Colors.RESET}")
    print("=" * 60)
    
    total_removed = 0
    
    # 1. Remove intermediate summary files
    print_status("1. Removing intermediate summary files...", "section")
    intermediate_files = [
        "TASK_2_2_IMPLEMENTATION_SUMMARY.md",
        "TASK_2_3_IMPLEMENTATION_SUMMARY.md",
        "TASK_3_2_IMPLEMENTATION_SUMMARY.md",
        "TASK_3_3_IMPLEMENTATION_SUMMARY.md",
        "ENHANCEMENT_SUMMARY.md",
        "TESTING_SUMMARY.md",
        "REPOSITORY_CLEANUP_SUMMARY.md",
        "WARNINGS_CLEANUP.md",
        "MCP_STATUS.md",
        "IDE_INTEGRATION_TESTING_GUIDE.md",
        "VSCODE_EXTENSION_SUMMARY.md",
    ]
    total_removed += remove_files(project_root, intermediate_files)
    
    # 2. Remove duplicate documentation
    print_status("2. Removing duplicate documentation from root...", "section")
    duplicate_docs = [
        "DEPLOYMENT.md",
        "PRODUCTION_READINESS.md",
        "SHIPPING_GUIDE.md",
    ]
    total_removed += remove_files(project_root, duplicate_docs)
    
    # 3. Clean docs directory
    print_status("3. Cleaning docs/ directory...", "section")
    docs_cleanup = [
        "docs/IMPLEMENTATION_SUMMARY.md",
        "docs/SOLID_IMPLEMENTATION.md",
        "docs/SOLID_SUCCESS.md",
        "docs/STATUS.md",
        "docs/WARNINGS_CLEANUP.md",
    ]
    total_removed += remove_files(project_root, docs_cleanup)
    
    # 4. Remove test/demo files from root
    print_status("4. Removing test/demo files from root...", "section")
    test_files = [
        "demo_ide_integration.rs",
        "test_ide_integration.rs",
        "vscode-extension-integration-test.rs",
    ]
    total_removed += remove_files(project_root, test_files)
    
    # 5. Organize build scripts
    print_status("5. Organizing build scripts...", "section")
    script_moves = {
        "build-extension.sh": "scripts/build-extension.sh",
        "build-extension.ps1": "scripts/build-extension.ps1",
        "run_ide_tests.sh": "scripts/run_ide_tests.sh",
        "run_ide_tests.ps1": "scripts/run_ide_tests.ps1",
        "test_mcp.sh": "scripts/test_mcp.sh",
    }
    
    # Ensure scripts directory exists
    scripts_dir = project_root / "scripts"
    scripts_dir.mkdir(parents=True, exist_ok=True)
    
    total_removed += move_files(project_root, script_moves)
    
    # 6. Verify structure
    print_status("6. Project structure after cleanup...", "section")
    
    root_files = sorted([f.name for f in project_root.glob("*.md")])
    if root_files:
        print("Root-level markdown files (production documentation):")
        for f in root_files:
            print(f"  • {f}")
    
    # Summary
    print("\n" + "=" * 60)
    print_status(f"Cleanup complete! Removed {total_removed} files", "success")
    print("\n" + Colors.BOLD + "Next steps:" + Colors.RESET)
    print("1. Verify build: cargo build")
    print("2. Run tests: cargo test")
    print("3. Commit changes: git add -A && git commit -m 'chore: cleanup project for production'")

if __name__ == "__main__":
    main()
