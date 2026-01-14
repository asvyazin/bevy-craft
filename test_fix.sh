#!/bin/bash

cd /Volumes/KINGSTON/dev/rust/bevy-craft

echo "Building the application..."
cargo build 2>&1 | grep -E "(Finished|error)" | tail -5

echo ""
echo "Running the application for 5 seconds to check output..."
echo ""

# Run the application and capture output for 5 seconds
timeout 5s cargo run 2>&1 | grep -E "(Testing alkyd-enhanced texture generation|Alkyd-enhanced texture generation test completed)" | head -10

echo ""
echo "Checking if the test message appears only once..."
echo ""

# Count how many times the test message appears
COUNT=$(timeout 5s cargo run 2>&1 | grep -c "Testing alkyd-enhanced texture generation" || echo "0")

echo "Test message appeared $COUNT times"

if [ "$COUNT" -eq "1" ]; then
    echo "✅ SUCCESS: Test message appears only once!"
else
    echo "❌ FAILURE: Test message appears $COUNT times (should be 1)"
fi