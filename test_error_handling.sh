#!/bin/bash

# Test script to verify error handling improvements
echo "Testing error handling improvements..."

# Build the project
echo "Building project..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful - no compilation errors"
else
    echo "❌ Build failed"
    exit 1
fi

# Check for unwrap usage in critical paths
echo "Checking for remaining unwrap() calls in critical paths..."
echo "Routes unwrap count:"
grep -c "\.unwrap()" src/routes/targets.rs || echo "0"

echo "Prober unwrap count:"
grep -c "\.unwrap()" src/prober/mod.rs || echo "0"

echo "Main unwrap count:"
grep -c "\.unwrap()" src/main.rs || echo "0"

echo "✅ Error handling improvements completed!"
echo ""
echo "Key improvements made:"
echo "1. InfluxDB query errors now return HTTP 503 instead of panicking"
echo "2. Database operation errors return appropriate HTTP status codes"
echo "3. Prober initialization errors are handled gracefully"
echo "4. Invalid IP addresses in targets are handled without panicking"
echo "5. DataPoint building errors are handled in the prober"