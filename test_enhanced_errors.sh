#!/bin/bash
# Test script to demonstrate enhanced error visualization

echo "🎨 Testing Enhanced Error Visualization"
echo "======================================"
echo

# Test 1: Missing API key error
echo "1️⃣ Testing API key error visualization:"
unset ANTHROPIC_API_KEY
cargo run --bin ccswarm -- start 2>&1 | head -30

echo
echo "Press Enter to continue..."
read

# Test 2: Doctor command with error diagnosis
echo "2️⃣ Testing error diagnosis:"
cargo run --bin ccswarm -- doctor --error ENV001

echo
echo "Press Enter to continue..."
read

# Test 3: List all error codes
echo "3️⃣ Testing error help:"
cargo run --bin ccswarm -- help errors

echo
echo "✅ Test complete!"