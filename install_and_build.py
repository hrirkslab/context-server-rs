#!/usr/bin/env python3
import subprocess
import sys
import os
import json

def run_command(cmd, description=None, shell=True):
    """Run a command and return output, error, and exit code"""
    if description:
        print(f"\n{'='*60}")
        print(f"Step: {description}")
        print(f"{'='*60}")
        print(f"Command: {cmd}\n")
    
    try:
        # Use bash explicitly for better compatibility
        process = subprocess.Popen(
            cmd,
            shell=shell,
            executable='/bin/bash',
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1
        )
        
        output_lines = []
        for line in process.stdout:
            print(line, end='')
            output_lines.append(line)
        
        process.wait()
        output = ''.join(output_lines)
        exit_code = process.returncode
        
        return output, exit_code
    except Exception as e:
        print(f"ERROR: {e}")
        return str(e), 1

def main():
    print("="*60)
    print("RUST INSTALLATION AND PROJECT BUILD")
    print("="*60)
    
    # Step 1: Install Rust
    output, exit_code = run_command(
        'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y',
        "Install Rust toolchain"
    )
    
    if exit_code != 0:
        print(f"\nERROR: Rust installation failed with exit code {exit_code}")
        return
    
    print("\n✓ Rust installation completed")
    
    # Step 2: Source Rust environment and check versions
    source_cmd = 'source $HOME/.cargo/env && which rustc && rustc --version && cargo --version'
    output, exit_code = run_command(
        source_cmd,
        "Verify Rust installation"
    )
    
    if exit_code != 0:
        print(f"\nERROR: Rust verification failed with exit code {exit_code}")
        return
    
    print("\n✓ Rust verified")
    
    # Step 3: Build the project
    build_cmd = 'cd /workspaces/context-server-rs && source $HOME/.cargo/env && cargo build 2>&1 | tee build.log'
    output, exit_code = run_command(
        build_cmd,
        "Build the Rust project"
    )
    
    # Save full output to a file
    with open('/workspaces/context-server-rs/build_output.txt', 'w') as f:
        f.write(output)
    
    # Print summary
    print("\n" + "="*60)
    print("BUILD SUMMARY")
    print("="*60)
    
    if exit_code == 0:
        print("✓ BUILD SUCCESSFUL")
    else:
        print(f"✗ BUILD FAILED (exit code: {exit_code})")
    
    print(f"Full output saved to: /workspaces/context-server-rs/build_output.txt")
    print(f"Exit code: {exit_code}")
    
    # Count errors and warnings
    errors = output.count("error")
    warnings = output.count("warning")
    print(f"Errors: {errors}")
    print(f"Warnings: {warnings}")
    
    sys.exit(exit_code)

if __name__ == "__main__":
    main()
