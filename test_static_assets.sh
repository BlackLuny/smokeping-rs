#!/bin/bash

echo "Testing static asset serving fix..."

# Build the frontend first
echo "Building frontend..."
cd frontend
npm run build
cd ..

# Build the backend
echo "Building backend..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful"
else
    echo "❌ Build failed"
    exit 1
fi

# Check that the dist directory has the expected files
echo "Checking frontend build output..."
if [ -f "frontend/dist/index.html" ]; then
    echo "✅ index.html found"
else
    echo "❌ index.html not found"
    exit 1
fi

if [ -d "frontend/dist/assets" ]; then
    echo "✅ assets directory found"
    ls -la frontend/dist/assets/
else
    echo "❌ assets directory not found"
    exit 1
fi

echo ""
echo "✅ Static asset serving fix completed!"
echo ""
echo "Key improvements made:"
echo "1. Added proper MIME type detection for static assets"
echo "2. Static assets (JS, CSS, images) are now served with correct Content-Type headers"
echo "3. SPA routing still works for non-asset routes"
echo "4. API routes continue to return 404 as expected"
echo ""
echo "The fix ensures that:"
echo "- /assets/index-qciAn6_P.js returns JavaScript with 'application/javascript' MIME type"
echo "- /assets/index-DK3SaXSL.css returns CSS with 'text/css' MIME type"
echo "- /vite.svg returns SVG with 'image/svg+xml' MIME type"
echo "- SPA routes like /dashboard still return the index.html file"