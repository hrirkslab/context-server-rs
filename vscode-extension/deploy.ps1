# Professional Context Engine VS Code Extension Deployment Script (PowerShell)
# This script builds, packages, and optionally installs the extension

param(
    [switch]$NoInstall,
    [switch]$SkipTests,
    [switch]$IgnoreLint,
    [switch]$IgnoreTests,
    [switch]$CleanOnly,
    [switch]$Help
)

# Colors for output
$Red = "Red"
$Green = "Green"
$Yellow = "Yellow"
$Blue = "Blue"

function Write-Status {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor $Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor $Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor $Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor $Red
}

function Test-Command {
    param([string]$Command)
    try {
        Get-Command $Command -ErrorAction Stop | Out-Null
        return $true
    }
    catch {
        return $false
    }
}

function Test-Prerequisites {
    Write-Status "Checking prerequisites..."
    
    if (-not (Test-Command "node")) {
        Write-Error "Node.js is not installed. Please install Node.js 16.x or higher."
        exit 1
    }
    
    if (-not (Test-Command "npm")) {
        Write-Error "npm is not installed. Please install npm."
        exit 1
    }
    
    if (-not (Test-Command "code")) {
        Write-Warning "VS Code CLI is not available. Extension installation will be skipped."
        $script:SkipInstall = $true
    }
    
    # Check Node.js version
    $nodeVersion = (node --version) -replace 'v', '' -split '\.' | Select-Object -First 1
    if ([int]$nodeVersion -lt 16) {
        Write-Error "Node.js version 16 or higher is required. Current version: $(node --version)"
        exit 1
    }
    
    Write-Success "Prerequisites check passed"
}

function Clear-Build {
    Write-Status "Cleaning previous builds..."
    
    # Remove output directory
    if (Test-Path "out") {
        Remove-Item -Recurse -Force "out"
        Write-Status "Removed out directory"
    }
    
    # Remove existing VSIX files
    $vsixFiles = Get-ChildItem -Filter "*.vsix"
    if ($vsixFiles) {
        $vsixFiles | Remove-Item -Force
        Write-Status "Removed existing VSIX files"
    }
    
    Write-Success "Build cleanup completed"
}

function Install-Dependencies {
    Write-Status "Installing dependencies..."
    
    if (Test-Path "package-lock.json") {
        npm ci
    } else {
        npm install
    }
    
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to install dependencies"
        exit 1
    }
    
    Write-Success "Dependencies installed"
}

function Invoke-Lint {
    Write-Status "Running linter..."
    
    npm run lint
    
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Linting failed"
        if (-not $IgnoreLint) {
            exit 1
        } else {
            Write-Warning "Continuing despite linting errors (IgnoreLint flag set)"
        }
    } else {
        Write-Success "Linting passed"
    }
}

function Invoke-Compile {
    Write-Status "Compiling TypeScript..."
    
    npm run compile
    
    if ($LASTEXITCODE -ne 0) {
        Write-Error "TypeScript compilation failed"
        exit 1
    }
    
    Write-Success "TypeScript compilation completed"
}

function Invoke-Tests {
    Write-Status "Running tests..."
    
    if ($SkipTests) {
        Write-Warning "Tests skipped (SkipTests flag set)"
        return
    }
    
    npm test
    
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Tests failed"
        if (-not $IgnoreTests) {
            exit 1
        } else {
            Write-Warning "Continuing despite test failures (IgnoreTests flag set)"
        }
    } else {
        Write-Success "Tests passed"
    }
}

function New-Package {
    Write-Status "Creating VSIX package..."
    
    # Install vsce if not available
    if (-not (Test-Command "vsce")) {
        Write-Status "Installing vsce..."
        npm install -g @vscode/vsce
    }
    
    # Create package
    vsce package --no-dependencies
    
    if ($LASTEXITCODE -ne 0) {
        Write-Error "VSIX packaging failed"
        exit 1
    }
    
    # Find the created VSIX file
    $vsixFile = Get-ChildItem -Filter "*.vsix" | Select-Object -First 1
    if ($vsixFile) {
        $fileSize = [math]::Round($vsixFile.Length / 1MB, 2)
        Write-Success "VSIX package created: $($vsixFile.Name) ($fileSize MB)"
        
        # Save build info
        @{
            VsixFile = $vsixFile.Name
            BuildDate = Get-Date
        } | ConvertTo-Json | Out-File -FilePath ".build-info.json" -Encoding UTF8
    } else {
        Write-Error "No VSIX file was created"
        exit 1
    }
}

function Install-Extension {
    if ($NoInstall -or $script:SkipInstall) {
        Write-Warning "Extension installation skipped"
        return
    }
    
    Write-Status "Installing extension locally..."
    
    if (Test-Path ".build-info.json") {
        $buildInfo = Get-Content ".build-info.json" | ConvertFrom-Json
        $vsixFile = $buildInfo.VsixFile
        
        if (Test-Path $vsixFile) {
            code --install-extension $vsixFile
            
            if ($LASTEXITCODE -eq 0) {
                Write-Success "Extension installed successfully"
                Write-Status "Restart VS Code to activate the extension"
            } else {
                Write-Error "Extension installation failed"
                exit 1
            }
        } else {
            Write-Error "VSIX file not found: $vsixFile"
            exit 1
        }
    } else {
        Write-Error "Build info not found. Package creation may have failed."
        exit 1
    }
}

function New-Instructions {
    Write-Status "Generating installation instructions..."
    
    if (Test-Path ".build-info.json") {
        $buildInfo = Get-Content ".build-info.json" | ConvertFrom-Json
        $packageJson = Get-Content "package.json" | ConvertFrom-Json
        
        $instructions = @"
# Professional Context Engine VS Code Extension
## Deployment Instructions

### Package Information
- **File**: $($buildInfo.VsixFile)
- **Version**: $($packageJson.version)
- **Build Date**: $($buildInfo.BuildDate)

### Installation Commands

``````powershell
# Install via VS Code CLI
code --install-extension $($buildInfo.VsixFile)

# Or install via VS Code UI
# 1. Open VS Code
# 2. Press Ctrl+Shift+P
# 3. Type "Extensions: Install from VSIX..."
# 4. Select $($buildInfo.VsixFile)
``````

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
"@
        
        $instructions | Out-File -FilePath "DEPLOYMENT_INSTRUCTIONS.md" -Encoding UTF8
        Write-Success "Deployment instructions generated: DEPLOYMENT_INSTRUCTIONS.md"
    }
}

function Show-Usage {
    Write-Host @"
Usage: .\deploy.ps1 [OPTIONS]

Options:
  -NoInstall        Skip local installation
  -SkipTests        Skip running tests
  -IgnoreLint       Continue despite linting errors
  -IgnoreTests      Continue despite test failures
  -CleanOnly        Only clean previous builds
  -Help             Show this help message

Examples:
  .\deploy.ps1                    # Full build and install
  .\deploy.ps1 -NoInstall         # Build without installing
  .\deploy.ps1 -SkipTests         # Build without running tests
  .\deploy.ps1 -CleanOnly         # Clean previous builds only
"@
}

function Main {
    if ($Help) {
        Show-Usage
        exit 0
    }
    
    Write-Status "Starting Professional Context Engine VS Code Extension deployment..."
    Write-Host ""
    
    # Check prerequisites
    Test-Prerequisites
    Write-Host ""
    
    # Clean previous builds
    Clear-Build
    Write-Host ""
    
    # If clean-only flag is set, exit here
    if ($CleanOnly) {
        Write-Success "Clean completed. Exiting as requested."
        exit 0
    }
    
    # Install dependencies
    Install-Dependencies
    Write-Host ""
    
    # Run linting
    Invoke-Lint
    Write-Host ""
    
    # Compile TypeScript
    Invoke-Compile
    Write-Host ""
    
    # Run tests
    Invoke-Tests
    Write-Host ""
    
    # Create VSIX package
    New-Package
    Write-Host ""
    
    # Install extension locally
    Install-Extension
    Write-Host ""
    
    # Generate installation instructions
    New-Instructions
    Write-Host ""
    
    Write-Success "Deployment completed successfully!"
    Write-Host ""
    Write-Status "Next steps:"
    Write-Host "1. Test the extension in VS Code"
    Write-Host "2. Share the VSIX file with users"
    Write-Host "3. See DEPLOYMENT_INSTRUCTIONS.md for distribution"
    Write-Host "4. Consider publishing to VS Code Marketplace"
}

# Run main function
Main