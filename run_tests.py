#!/usr/bin/env python3
import subprocess
import sys
import os

os.chdir('/workspaces/context-server-rs')

tests = [
    ("Build Release Binary", "cargo build --release 2>&1 | tail -30"),
    ("Test Help", "./target/release/context-server-rs --help"),
    ("Test Query Command", "./target/release/context-server-rs query -p default"),
    ("Test Search Performance", "./target/release/context-server-rs search performance -p default"),
    ("Test Search Feature", "./target/release/context-server-rs search feature -p default"),
    ("Test List Feature", "./target/release/context-server-rs list feature -p default"),
]

results = {}

for test_name, command in tests:
    print(f"\n{'='*80}")
    print(f"TEST: {test_name}")
    print(f"Command: {command}")
    print(f"{'='*80}")
    
    try:
        result = subprocess.run(command, shell=True, capture_output=True, text=True, timeout=60)
        output = result.stdout + result.stderr
        print(output)
        results[test_name] = {
            'success': result.returncode == 0,
            'returncode': result.returncode,
            'output': output[:1000]  # Store first 1000 chars for summary
        }
        print(f"Return Code: {result.returncode}")
    except subprocess.TimeoutExpired:
        print(f"Test timed out!")
        results[test_name] = {'success': False, 'returncode': -1, 'output': 'TIMEOUT'}
    except Exception as e:
        print(f"Error running test: {e}")
        results[test_name] = {'success': False, 'returncode': -1, 'output': str(e)}

print(f"\n\n{'='*80}")
print("SUMMARY OF RESULTS")
print(f"{'='*80}")
for test_name, result in results.items():
    status = "✓ PASSED" if result['success'] else "✗ FAILED"
    print(f"{test_name}: {status} (Return code: {result['returncode']})")
