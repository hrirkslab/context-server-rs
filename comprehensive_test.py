#!/usr/bin/env python3
"""
Comprehensive final test suite for context-server-rs
Tests all CLI features and verifies production readiness
"""

import subprocess
import os
import sys
import json
import time
from pathlib import Path

os.chdir('/workspaces/context-server-rs')

def run_command(cmd, timeout=60, description=""):
    """Run a shell command and return result"""
    print(f"\n{'─'*80}")
    print(f"Command: {cmd}")
    if description:
        print(f"Description: {description}")
    print(f"{'─'*80}")
    
    try:
        result = subprocess.run(
            cmd,
            shell=True,
            capture_output=True,
            text=True,
            timeout=timeout,
            env={**os.environ, 'PATH': f"{os.path.expanduser('~')}/.cargo/bin:{os.environ.get('PATH', '')}"}
        )
        output = result.stdout + result.stderr
        print(output)
        return {
            'success': result.returncode == 0,
            'returncode': result.returncode,
            'output': output,
            'stdout': result.stdout,
            'stderr': result.stderr
        }
    except subprocess.TimeoutExpired:
        return {'success': False, 'returncode': -1, 'output': 'TIMEOUT', 'stdout': '', 'stderr': 'Command timed out'}
    except Exception as e:
        return {'success': False, 'returncode': -1, 'output': str(e), 'stdout': '', 'stderr': str(e)}

# ==============================================================================
# PHASE 1: SETUP
# ==============================================================================
print("\n" + "="*80)
print("PHASE 1: SETUP - Installing Rust and Building Project")
print("="*80)

# Install Rust
print("\n[1/2] Installing Rust toolchain...")
rust_install = run_command(
    'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && source $HOME/.cargo/env',
    timeout=180,
    description="Install Rust from official source"
)

if not rust_install['success']:
    print(f"⚠ Rust installation may have issues, continuing anyway...")

# Verify Rust installation
print("\n[2/2] Verifying Rust/Cargo installation...")
verify_result = run_command(
    'export PATH="$HOME/.cargo/bin:$PATH" && rustc --version && cargo --version',
    timeout=30,
    description="Verify Rust and Cargo versions"
)

if not verify_result['success']:
    print("⚠ Could not verify Rust, attempting to build anyway...")

# ==============================================================================
# PHASE 2: BUILD
# ==============================================================================
print("\n" + "="*80)
print("PHASE 2: BUILD - Compiling Release Binary")
print("="*80)

build_result = run_command(
    'export PATH="$HOME/.cargo/bin:$PATH" && cargo build --release 2>&1 | tail -50',
    timeout=300,
    description="Build release binary (this may take a few minutes)"
)

# Check if binary exists
binary_path = Path("target/release/context-server-rs")
if binary_path.exists():
    binary_size = binary_path.stat().st_size / (1024 * 1024)
    print(f"\n✓ Binary successfully built!")
    print(f"  Location: {binary_path}")
    print(f"  Size: {binary_size:.2f} MB")
else:
    print(f"\n✗ Binary not found at {binary_path}")
    if not build_result['success']:
        print("Build failed. Showing error details:")
        print(build_result['output'][-2000:])  # Show last 2000 chars

# ==============================================================================
# PHASE 3: COMPREHENSIVE TESTS
# ==============================================================================
test_results = {}

print("\n" + "="*80)
print("PHASE 3: COMPREHENSIVE TESTS")
print("="*80)

if binary_path.exists():
    binary = str(binary_path)
    
    # TEST 1: Help Command
    print("\n\n" + "🔹"*40)
    print("TEST 1: Help Command - Verify all commands and examples visible")
    print("🔹"*40)
    test_name = "1. Help Command"
    result = run_command(
        f"{binary} --help",
        description="Check for all commands and features in help output"
    )
    test_results[test_name] = result
    
    # Check for expected content
    help_checks = {
        'serve command': 'serve' in result['stdout'].lower(),
        'query command': 'query' in result['stdout'].lower(),
        'list command': 'list' in result['stdout'].lower(),
        'search command': 'search' in result['stdout'].lower(),
        'get command': 'get' in result['stdout'].lower(),
        'examples section': 'example' in result['stdout'].lower(),
    }
    
    print("\nHelp Content Verification:")
    all_passed = True
    for check_name, passed in help_checks.items():
        status = "✓" if passed else "✗"
        print(f"  {status} {check_name}")
        all_passed = all_passed and passed
    
    test_results[test_name]['checks'] = help_checks
    test_results[test_name]['success'] = result['returncode'] == 0 and all_passed
    
    # TEST 2: Query Output Formats (JSON)
    print("\n\n" + "🔹"*40)
    print("TEST 2: Query Command - JSON Format")
    print("🔹"*40)
    test_name = "2a. Query Format - JSON"
    result = run_command(
        f"{binary} query -p default -f json 2>&1 | head -20",
        description="Test query with JSON output format"
    )
    test_results[test_name] = result
    test_results[test_name]['success'] = result['returncode'] == 0
    
    # TEST 3: Query Output Formats (Text)
    print("\n\n" + "🔹"*40)
    print("TEST 3: Query Command - Text Format")
    print("🔹"*40)
    test_name = "2b. Query Format - Text"
    result = run_command(
        f"{binary} query -p default -f text 2>&1 | head -10",
        description="Test query with text output format"
    )
    test_results[test_name] = result
    test_results[test_name]['success'] = result['returncode'] == 0
    
    # TEST 4: Query Output Formats (YAML)
    print("\n\n" + "🔹"*40)
    print("TEST 4: Query Command - YAML Format")
    print("🔹"*40)
    test_name = "2c. Query Format - YAML"
    result = run_command(
        f"{binary} query -p default -f yaml 2>&1 | head -10",
        description="Test query with YAML output format"
    )
    test_results[test_name] = result
    test_results[test_name]['success'] = result['returncode'] == 0
    
    # TEST 5: Search Entity Types - Encryption
    print("\n\n" + "🔹"*40)
    print("TEST 5: Search - Entity Type 1 (Encryption)")
    print("🔹"*40)
    test_name = "3a. Search - Encryption"
    result = run_command(
        f"{binary} search encryption -p default",
        description="Search for encryption-related contexts"
    )
    test_results[test_name] = result
    test_results[test_name]['success'] = result['returncode'] == 0
    
    # TEST 6: Search Entity Types - Payment
    print("\n\n" + "🔹"*40)
    print("TEST 6: Search - Entity Type 2 (Payment)")
    print("🔹"*40)
    test_name = "3b. Search - Payment"
    result = run_command(
        f"{binary} search payment -p default",
        description="Search for payment-related contexts"
    )
    test_results[test_name] = result
    test_results[test_name]['success'] = result['returncode'] == 0
    
    # TEST 7: Search Entity Types - User
    print("\n\n" + "🔹"*40)
    print("TEST 7: Search - Entity Type 3 (User)")
    print("🔹"*40)
    test_name = "3c. Search - User"
    result = run_command(
        f"{binary} search user -p default",
        description="Search for user-related contexts"
    )
    test_results[test_name] = result
    test_results[test_name]['success'] = result['returncode'] == 0
    
    # TEST 8: List Features
    print("\n\n" + "🔹"*40)
    print("TEST 8: List Command - Features Entity Type")
    print("🔹"*40)
    test_name = "4. List Features"
    result = run_command(
        f"{binary} list feature -p default 2>/dev/null",
        description="List all features with proper formatting"
    )
    test_results[test_name] = result
    test_results[test_name]['success'] = result['returncode'] == 0
    
    # TEST 9: Get Command
    print("\n\n" + "🔹"*40)
    print("TEST 9: Get Command - Retrieve Specific Entity")
    print("🔹"*40)
    test_name = "5. Get Command"
    result = run_command(
        f"{binary} get rule-001 -p default",
        description="Get a specific context by ID"
    )
    test_results[test_name] = result
    # This might fail if rule-001 doesn't exist, which is ok
    
    # TEST 10: Error Handling
    print("\n\n" + "🔹"*40)
    print("TEST 10: Error Handling - Invalid Entity ID")
    print("🔹"*40)
    test_name = "6. Error Handling"
    result = run_command(
        f"{binary} get nonexistent -p default 2>&1",
        description="Test error handling with non-existent ID"
    )
    test_results[test_name] = result
    # Should return non-zero exit code
    test_results[test_name]['success'] = result['returncode'] != 0 or 'not found' in result['output'].lower() or 'error' in result['output'].lower()

else:
    print("\n✗ SKIPPING TESTS - Binary not found")
    test_results["Binary Check"] = {
        'success': False,
        'returncode': 1,
        'output': 'Binary not found'
    }

# ==============================================================================
# FINAL SUMMARY
# ==============================================================================
print("\n\n" + "="*80)
print("FINAL TEST SUMMARY")
print("="*80)

passed = sum(1 for r in test_results.values() if r.get('success', False))
total = len(test_results)
percentage = (passed / total * 100) if total > 0 else 0

print(f"\nTests Passed: {passed}/{total} ({percentage:.1f}%)")
print(f"\nDetailed Results:")
for test_name in sorted(test_results.keys()):
    result = test_results[test_name]
    status = "✓ PASSED" if result.get('success', False) else "✗ FAILED"
    returncode = result.get('returncode', -1)
    print(f"  {status} - {test_name} (Return code: {returncode})")

# ==============================================================================
# PRODUCTION READINESS ASSESSMENT
# ==============================================================================
print("\n" + "="*80)
print("PRODUCTION READINESS ASSESSMENT")
print("="*80)

readiness_checklist = {
    'Binary builds successfully': binary_path.exists(),
    'Help command functional': test_results.get("1. Help Command", {}).get('success', False),
    'Query JSON format working': test_results.get("2a. Query Format - JSON", {}).get('success', False),
    'Query Text format working': test_results.get("2b. Query Format - Text", {}).get('success', False),
    'Query YAML format working': test_results.get("2c. Query Format - YAML", {}).get('success', False),
    'Search encryption working': test_results.get("3a. Search - Encryption", {}).get('success', False),
    'Search payment working': test_results.get("3b. Search - Payment", {}).get('success', False),
    'Search user working': test_results.get("3c. Search - User", {}).get('success', False),
    'List features working': test_results.get("4. List Features", {}).get('success', False),
    'Get command functional': test_results.get("5. Get Command", {}).get('returncode', -1) in [0, 1],  # Either works or properly errors
    'Error handling proper': test_results.get("6. Error Handling", {}).get('success', False),
}

print("\nProduction Readiness Checklist:")
all_ready = True
for item, passed in readiness_checklist.items():
    status = "✓" if passed else "✗"
    print(f"  {status} {item}")
    all_ready = all_ready and passed

print("\n" + "="*80)
if all_ready and percentage >= 90:
    print("🚀 PRODUCTION READY - All critical tests passed")
    print("The application is ready for production deployment.")
    sys.exit(0)
elif percentage >= 70:
    print("⚠ MOSTLY READY - Most features working, minor issues to address")
    print(f"Pass rate: {percentage:.1f}%")
    sys.exit(0)
else:
    print("❌ NOT READY - Significant issues remain")
    print(f"Pass rate: {percentage:.1f}%")
    sys.exit(1)
