#!/usr/bin/env python3
"""
Remove redundant cleanup documentation
Keeps only: START_CLEANUP.md, QUICK_CLEANUP.md, scripts/README.md, PRODUCTION_READINESS_CHECKLIST.md
"""

import os
from pathlib import Path

# Redundant files to remove
REDUNDANT_FILES = [
    "CLEANUP_SUMMARY.md",
    "PRODUCTION_RELEASE.md",
    "PRODUCTION_CLEANUP.md",
    "CLEANUP_INSTRUCTIONS.md",
    "CLEANUP_INDEX.md",
    "CLEANUP_DELIVERY_SUMMARY.md",
    "CLEANUP_PACKAGE_SUMMARY.md",
]

def remove_redundant_docs():
    """Remove redundant cleanup documentation"""
    project_root = Path(__file__).parent.parent
    removed = 0
    
    print("🧹 Removing redundant cleanup documentation...\n")
    
    for file in REDUNDANT_FILES:
        file_path = project_root / file
        if file_path.exists():
            file_path.unlink()
            print(f"✓ Removed: {file}")
            removed += 1
        else:
            print(f"- Already removed: {file}")
    
    print(f"\n✅ Cleanup documentation consolidated!")
    print(f"   Removed: {removed} redundant files")
    print(f"\n📚 Lean documentation remaining:")
    print(f"   • START_CLEANUP.md - Main guide")
    print(f"   • QUICK_CLEANUP.md - Quick reference")
    print(f"   • scripts/README.md - Script documentation")
    print(f"   • docs/PRODUCTION_READINESS_CHECKLIST.md - Verification")

if __name__ == "__main__":
    remove_redundant_docs()
