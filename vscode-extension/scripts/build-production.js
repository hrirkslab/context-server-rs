#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

console.log('🚀 Building Professional Context Engine VS Code Extension for Production...\n');

// Step 1: Clean previous builds
console.log('🧹 Cleaning previous builds...');
try {
    if (fs.existsSync('out')) {
        fs.rmSync('out', { recursive: true, force: true });
    }
    if (fs.existsSync('*.vsix')) {
        const vsixFiles = fs.readdirSync('.').filter(file => file.endsWith('.vsix'));
        vsixFiles.forEach(file => fs.unlinkSync(file));
    }
    console.log('✅ Cleanup completed\n');
} catch (error) {
    console.error('❌ Cleanup failed:', error.message);
    process.exit(1);
}

// Step 2: Install dependencies
console.log('📦 Installing dependencies...');
try {
    execSync('npm install', { stdio: 'inherit' });
    console.log('✅ Dependencies installed\n');
} catch (error) {
    console.error('❌ Failed to install dependencies:', error.message);
    process.exit(1);
}

// Step 3: Run linting
console.log('🔍 Running linter...');
try {
    execSync('npm run lint', { stdio: 'inherit' });
    console.log('✅ Linting passed\n');
} catch (error) {
    console.error('❌ Linting failed:', error.message);
    process.exit(1);
}

// Step 4: Compile TypeScript
console.log('🔨 Compiling TypeScript...');
try {
    execSync('npm run compile', { stdio: 'inherit' });
    console.log('✅ TypeScript compilation completed\n');
} catch (error) {
    console.error('❌ TypeScript compilation failed:', error.message);
    process.exit(1);
}

// Step 5: Run tests
console.log('🧪 Running tests...');
try {
    // Skip tests if no test environment is available
    if (process.env.SKIP_TESTS !== 'true') {
        execSync('npm test', { stdio: 'inherit' });
        console.log('✅ Tests passed\n');
    } else {
        console.log('⚠️ Tests skipped (SKIP_TESTS=true)\n');
    }
} catch (error) {
    console.error('❌ Tests failed:', error.message);
    if (process.env.IGNORE_TEST_FAILURES !== 'true') {
        process.exit(1);
    } else {
        console.log('⚠️ Continuing despite test failures (IGNORE_TEST_FAILURES=true)\n');
    }
}

// Step 6: Validate package.json
console.log('📋 Validating package.json...');
try {
    const packageJson = JSON.parse(fs.readFileSync('package.json', 'utf8'));
    
    const requiredFields = ['name', 'displayName', 'description', 'version', 'publisher', 'engines'];
    const missingFields = requiredFields.filter(field => !packageJson[field]);
    
    if (missingFields.length > 0) {
        throw new Error(`Missing required fields: ${missingFields.join(', ')}`);
    }
    
    if (!packageJson.engines.vscode) {
        throw new Error('Missing vscode engine requirement');
    }
    
    console.log('✅ package.json validation passed\n');
} catch (error) {
    console.error('❌ package.json validation failed:', error.message);
    process.exit(1);
}

// Step 7: Create VSIX package
console.log('📦 Creating VSIX package...');
try {
    execSync('npx vsce package --no-dependencies', { stdio: 'inherit' });
    
    // Find the created VSIX file
    const vsixFiles = fs.readdirSync('.').filter(file => file.endsWith('.vsix'));
    if (vsixFiles.length === 0) {
        throw new Error('No VSIX file was created');
    }
    
    const vsixFile = vsixFiles[0];
    const stats = fs.statSync(vsixFile);
    const fileSizeMB = (stats.size / (1024 * 1024)).toFixed(2);
    
    console.log(`✅ VSIX package created: ${vsixFile} (${fileSizeMB} MB)\n`);
} catch (error) {
    console.error('❌ VSIX packaging failed:', error.message);
    process.exit(1);
}

// Step 8: Generate installation instructions
console.log('📝 Generating installation instructions...');
try {
    const packageJson = JSON.parse(fs.readFileSync('package.json', 'utf8'));
    const vsixFile = fs.readdirSync('.').find(file => file.endsWith('.vsix'));
    
    const instructions = `
# Professional Context Engine VS Code Extension
## Installation Instructions

### Method 1: Install from VSIX file
1. Download the extension package: \`${vsixFile}\`
2. Open VS Code
3. Press \`Ctrl+Shift+P\` (or \`Cmd+Shift+P\` on Mac) to open Command Palette
4. Type "Extensions: Install from VSIX..." and select it
5. Browse and select the \`${vsixFile}\` file
6. Click "Install"
7. Reload VS Code when prompted

### Method 2: Command Line Installation
\`\`\`bash
code --install-extension ${vsixFile}
\`\`\`

### Method 3: Manual Installation
1. Copy \`${vsixFile}\` to your VS Code extensions directory:
   - Windows: \`%USERPROFILE%\\.vscode\\extensions\`
   - macOS: \`~/.vscode/extensions\`
   - Linux: \`~/.vscode/extensions\`
2. Extract the VSIX file (it's a ZIP archive)
3. Restart VS Code

## Configuration
After installation, configure the extension:
1. Open VS Code Settings (\`Ctrl+,\`)
2. Search for "Context Engine"
3. Set the server URL: \`http://localhost:3000\` (or your server URL)
4. Configure other settings as needed

## Verification
1. Open Command Palette (\`Ctrl+Shift+P\`)
2. Type "Context Engine: Test Connection"
3. Verify successful connection to your Context Engine server

## Requirements
- VS Code ${packageJson.engines.vscode}
- Professional Context Engine server running
- Network access to the Context Engine server

For more information, see README.md
`;
    
    fs.writeFileSync('INSTALLATION.md', instructions.trim());
    console.log('✅ Installation instructions generated: INSTALLATION.md\n');
} catch (error) {
    console.error('❌ Failed to generate installation instructions:', error.message);
}

console.log('🎉 Production build completed successfully!');
console.log('\nNext steps:');
console.log('1. Test the extension by installing the VSIX file');
console.log('2. Share the VSIX file with users');
console.log('3. Consider publishing to VS Code Marketplace');
console.log('4. See INSTALLATION.md for detailed installation instructions');