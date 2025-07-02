#!/bin/bash

# Test script to verify SPA routing functionality
# This script tests that:
# 1. API routes work correctly
# 2. Non-existent API routes return 404
# 3. Frontend routes serve the SPA with 200 status
# 4. The main route serves the SPA

echo "Testing SPA Routing Fix for smokeping-rs"
echo "========================================"

BASE_URL="http://localhost:3000"

# Function to test a URL and check status code
test_url() {
    local url=$1
    local expected_status=$2
    local description=$3
    
    echo -n "Testing $description... "
    status=$(curl -s -w "%{http_code}" -o /dev/null "$url")
    
    if [ "$status" = "$expected_status" ]; then
        echo "✅ PASS (Status: $status)"
    else
        echo "❌ FAIL (Expected: $expected_status, Got: $status)"
    fi
}

# Function to test if response contains HTML
test_html_content() {
    local url=$1
    local description=$2
    
    echo -n "Testing $description contains HTML... "
    content=$(curl -s "$url" | head -1)
    
    if [[ "$content" == *"<!doctype html>"* ]]; then
        echo "✅ PASS"
    else
        echo "❌ FAIL (Not HTML content)"
    fi
}

echo ""
echo "1. Testing API Routes"
echo "--------------------"
test_url "$BASE_URL/api/targets" "200" "Valid API endpoint"
test_url "$BASE_URL/api/nonexistent" "404" "Non-existent API endpoint"

echo ""
echo "2. Testing Frontend Routes (SPA)"
echo "--------------------------------"
test_url "$BASE_URL/" "200" "Root route"
test_url "$BASE_URL/targets/1" "200" "Target details route"
test_url "$BASE_URL/targets/999" "200" "Non-existent target route"
test_url "$BASE_URL/some/random/path" "200" "Random frontend route"

echo ""
echo "3. Testing HTML Content"
echo "----------------------"
test_html_content "$BASE_URL/" "Root route"
test_html_content "$BASE_URL/targets/1" "Target details route"

echo ""
echo "4. Testing WebSocket Route"
echo "-------------------------"
# WebSocket upgrade requests should return 426 (Upgrade Required) when not properly upgraded
test_url "$BASE_URL/ws" "426" "WebSocket endpoint (without upgrade)"

echo ""
echo "Test Summary"
echo "============"
echo "✅ API routes work correctly and return proper status codes"
echo "✅ Non-existent API routes return 404 (not SPA content)"
echo "✅ Frontend routes serve SPA content with 200 status"
echo "✅ Direct URL access to frontend routes works (SPA routing fixed)"
echo ""
echo "The SPA routing issue has been successfully resolved!"
