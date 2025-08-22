#!/bin/bash

# Professional Context Engine VS Code Extension Deployment Script
# This script builds, packages, and optionally installs the extension

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    if ! command_exists node; then
        print_error "Node.js is not installed. Please install Node.js 16.x or higher."
        exit 1
    fi
    
    if ! command_exists npm; then
        print_error "npm is not installed. Please install npm."
        exit 1
    fi
    
    if ! command_exists code; then
        print_warning "VS Code CLI is not available. Extension installation will be skipped."
        SKIP_INSTALL=true
    fi
    
    # Check Node.js version
    NODE_VERSION=$(node --version | cut -d'v' -f2 | cut -d'.' -f1)
    if [ "$NODE_VERSION" -lt 16 ]; then
        print_error "Node.js version 16 or higher is required. Current version: $(node --version)"
        exit 1
    fi
    
    print_success "Prerequisites check passed"
}

# Function to clean previous builds
clean_build() {
    print_status "Cleaning previous builds..."
    
    # Remove output directory
    if [ -d "out" ]; then
        rm -rf out
        print_status "Removed out directory"
    fi
    
    # Remove existing VSIX files
    if ls *.vsix 1> /dev/null 2>&1; then
        rm -f *.vsix
        print_status "Removed existing VSIX files"
    fi
    
    print_success "Build cleanup completed"
}

# Function to install dependencies
install_dependencies() {
    print_status "Installing dependencies..."
    
    if [ -f "package-lock.json" ]; then
        npm ci
    else
        npm install
    fi
    
    print_success "Dependencies installed"
}

# Function to run linting
run_lint() {
    print_status "Running linter..."
    
    if npm run lint; then
        print_success "Linting passed"
    else
        print_error "Linting failed"
        if [ "$IGNORE_LINT" != "true" ]; then
            exit 1
        else
            print_warning "Continuing despite linting errors (IGNORE_LINT=true)"
        fi
    fi
}

# Function to compile TypeScript
compile_typescript() {
    print_status "Compiling TypeScript..."
    
    if npm run compile; then
        print_success "TypeScript compilation completed"
    else
        print_error "TypeScript compilation failed"
        exit 1
    fi
}

# Function to run tests
run_tests() {
    print_status "Running tests..."
    
    if [ "$SKIP_TESTS" = "true" ]; then
        print_warning "Tests skipped (SKIP_TESTS=true)"
        return
    fi
    
    if npm test; then
        print_success "Tests passed"
    else
        print_error "Tests failed"
        if [ "$IGNORE_TEST_FAILURES" != "true" ]; then
            exit 1
        else
            print_warning "Continuing despite test failures (IGNORE_TEST_FAILURES=true)"
        fi
    fi
}

# Function to create VSIX package
create_package() {
    print_status "Creating VSIX package..."
    
    # Install vsce if not available
    if ! command_exists vsce; then
        print_status "Installing vsce..."
        npm install -g @vscode/vsce
    fi
    
    # Create package
    if vsce package --no-dependencies; then
        # Find the created VSIX file
        VSIX_FILE=$(ls *.vsix | head -1)
        if [ -n "$VSIX_FILE" ]; then
            FILE_SIZE=$(du -h "$VSIX_FILE" | cut -f1)
            print_success "VSIX package created: $VSIX_FILE ($FILE_SIZE)"
            echo "VSIX_FILE=$VSIX_FILE" > .build-info
        else
            print_error "No VSIX file was created"
            exit 1
        fi
    else
        print_error "VSIX packaging failed"
        exit 1
    fi
}

# Function to install extension locally
install_extension() {
    if [ "$SKIP_INSTALL" = "true" ] || [ "$NO_INSTALL" = "true" ]; then
        print_warning "Extension installation skipped"
        return
    fi
    
    print_status "Installing extension locally..."
    
    if [ -f ".build-info" ]; then
        source .build-info
        if [ -n "$VSIX_FILE" ] && [ -f "$VSIX_FILE" ]; then
            if code --install-extension "$VSIX_FILE"; then
                print_success "Extension installed successfully"
                print_status "Restart VS Code to activate the extension"
            else
                print_error "Extension installation failed"
                exit 1
            fi
        else
            print_error "VSIX file not found: $VSIX_FILE"
            exit 1
        fi
    else
        print_error "Build info not found. Package creation may have failed."
        exit 1
    fi
}

# Function to generate installation instructions
generate_instructions() {
    print_status "Generating installation instructions..."
    
    if [ -f ".build-info" ]; then
        source .build-info
        cat > DEPLOYMENT_INSTRUCTIONS.md << EOF
# Professional Context Engine VS Code Extension
## Deployment Instructions

### Package Information
- **File**: $VSIX_FILE
- **Version**: $(node -p "require('./package.json').version")
- **Build Date**: $(date)

### Installation Commands

\`\`\`bash
# Install via VS Code CLI
code --install-extension $VSIX_FILE

# Or install via VS Code UI
# 1. Open VS Code
# 2. Press Ctrl+Shift+P
# 3. Type "Extensions: Install from VSIX..."
# 4. Select $VSIX_FILE
\`\`\`

### Configuration
After installation, configure the extension:
1. Open VS Code Settings (Ctrl+,)
2. Search for "Context Engine"
3. Set server URL and other preferences

### Verification
Test the installation:
1. Open Command Palette (Ctrl+Shift+P)
2. Type "Context Engine: Test Connection"
3. Verify connection to your Context Engine server

For detailed instructions, see INSTALLATION.md
EOF
        print_success "Deployment instructions generated: DEPLOYMENT_INSTRUCTIONS.md"
    fi
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --no-install        Skip local installation"
    echo "  --skip-tests        Skip running tests"
    echo "  --ignore-lint       Continue despite linting errors"
    echo "  --ignore-tests      Continue despite test failures"
    echo "  --clean-only        Only clean previous builds"
    echo "  --help              Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  SKIP_TESTS=true     Skip running tests"
    echo "  IGNORE_LINT=true    Continue despite linting errors"
    echo "  IGNORE_TEST_FAILURES=true  Continue despite test failures"
    echo "  NO_INSTALL=true     Skip local installation"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-install)
            NO_INSTALL=true
            shift
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --ignore-lint)
            IGNORE_LINT=true
            shift
            ;;
        --ignore-tests)
            IGNORE_TEST_FAILURES=true
            shift
            ;;
        --clean-only)
            CLEAN_ONLY=true
            shift
            ;;
        --help)
            show_usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Main execution
main() {
    print_status "Starting Professional Context Engine VS Code Extension deployment..."
    echo ""
    
    # Check prerequisites
    check_prerequisites
    echo ""
    
    # Clean previous builds
    clean_build
    echo ""
    
    # If clean-only flag is set, exit here
    if [ "$CLEAN_ONLY" = "true" ]; then
        print_success "Clean completed. Exiting as requested."
        exit 0
    fi
    
    # Install dependencies
    install_dependencies
    echo ""
    
    # Run linting
    run_lint
    echo ""
    
    # Compile TypeScript
    compile_typescript
    echo ""
    
    # Run tests
    run_tests
    echo ""
    
    # Create VSIX package
    create_package
    echo ""
    
    # Install extension locally
    install_extension
    echo ""
    
    # Generate installation instructions
    generate_instructions
    echo ""
    
    print_success "Deployment completed successfully!"
    echo ""
    print_status "Next steps:"
    echo "1. Test the extension in VS Code"
    echo "2. Share the VSIX file with users"
    echo "3. See DEPLOYMENT_INSTRUCTIONS.md for distribution"
    echo "4. Consider publishing to VS Code Marketplace"
}

# Run main function
main "$@"