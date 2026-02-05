#!/usr/bin/env pwsh

Write-Host "IDE Integration Plugin Test Suite" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan
Write-Host

# Test 1: Run unit tests
Write-Host "Test 1: Running Unit Tests" -ForegroundColor Yellow
Write-Host "-----------------------------" -ForegroundColor Yellow
cargo test ide_integration_plugin --lib -- --nocapture
Write-Host

# Test 2: Check compilation
Write-Host "Test 2: Compilation Check" -ForegroundColor Yellow
Write-Host "----------------------------" -ForegroundColor Yellow
cargo check --lib
Write-Host

# Test 3: Run specific test with verbose output
Write-Host "Test 3: Detailed Language Detection Test" -ForegroundColor Yellow
Write-Host "-------------------------------------------" -ForegroundColor Yellow
cargo test ide_integration_plugin::tests::test_language_detection --lib -- --nocapture --exact
Write-Host

# Test 4: Run file analysis test
Write-Host "Test 4: File Analysis Test" -ForegroundColor Yellow
Write-Host "-----------------------------" -ForegroundColor Yellow
cargo test ide_integration_plugin::tests::test_file_analysis --lib -- --nocapture --exact
Write-Host

# Test 5: Run event handling test
Write-Host "Test 5: Event Handling Test" -ForegroundColor Yellow
Write-Host "-----------------------------" -ForegroundColor Yellow
cargo test ide_integration_plugin::tests::test_event_handling --lib -- --nocapture --exact
Write-Host

Write-Host "All tests completed!" -ForegroundColor Green
Write-Host
Write-Host "Next steps to test IDE integration:" -ForegroundColor Cyan
Write-Host "   1. Start the context server: cargo run" -ForegroundColor White
Write-Host "   2. Use the plugin API to send events" -ForegroundColor White
Write-Host "   3. Monitor context generation in real-time" -ForegroundColor White
Write-Host "   4. Test with your actual IDE integration" -ForegroundColor White