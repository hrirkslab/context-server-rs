#!/bin/bash

echo "🚀 IDE Integration Plugin Test Suite"
echo "===================================="
echo

# Test 1: Run unit tests
echo "📋 Test 1: Running Unit Tests"
echo "-----------------------------"
cargo test ide_integration_plugin --lib -- --nocapture
echo

# Test 2: Check compilation
echo "🔧 Test 2: Compilation Check"
echo "----------------------------"
cargo check --lib
echo

# Test 3: Run specific test with verbose output
echo "🔍 Test 3: Detailed Language Detection Test"
echo "-------------------------------------------"
cargo test ide_integration_plugin::tests::test_language_detection --lib -- --nocapture --exact
echo

# Test 4: Run file analysis test
echo "📄 Test 4: File Analysis Test"
echo "-----------------------------"
cargo test ide_integration_plugin::tests::test_file_analysis --lib -- --nocapture --exact
echo

# Test 5: Run event handling test
echo "⚡ Test 5: Event Handling Test"
echo "-----------------------------"
cargo test ide_integration_plugin::tests::test_event_handling --lib -- --nocapture --exact
echo

echo "✅ All tests completed!"
echo
echo "💡 Next steps to test IDE integration:"
echo "   1. Start the context server: cargo run"
echo "   2. Use the plugin API to send events"
echo "   3. Monitor context generation in real-time"
echo "   4. Test with your actual IDE integration"