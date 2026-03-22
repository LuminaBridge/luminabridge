#!/bin/bash
# LuminaBridge Performance Benchmark Script
# This script runs various performance benchmarks for the LuminaBridge API

set -e

# Configuration
BASE_URL="${BASE_URL:-http://localhost:3000}"
API_TOKEN="${API_TOKEN:-sk-test-token}"
DURATION="${DURATION:-30s}"
CONNECTIONS="${CONNECTIONS:-100}"
THREADS="${THREADS:-12}"

echo "🌉 LuminaBridge Performance Benchmark"
echo "======================================"
echo "Base URL: $BASE_URL"
echo "Duration: $DURATION"
echo "Connections: $CONNECTIONS"
echo "Threads: $THREADS"
echo ""

# Check if wrk is installed
if ! command -v wrk &> /dev/null; then
    echo "❌ wrk is not installed. Please install it first:"
    echo "   macOS: brew install wrk"
    echo "   Linux: apt-get install wrk"
    echo "   Windows: Download from https://github.com/wg/wrk"
    exit 1
fi

# Health check endpoint benchmark
echo "📊 Benchmark: Health Check Endpoint"
echo "-----------------------------------"
wrk -t$THREADS -c$CONNECTIONS -d$DURATION "$BASE_URL/health"
echo ""

# API versions endpoint benchmark
echo "📊 Benchmark: API Versions Endpoint"
echo "------------------------------------"
wrk -t$THREADS -c$CONNECTIONS -d$DURATION "$BASE_URL/v1"
echo ""

# Models endpoint benchmark (requires auth)
echo "📊 Benchmark: Models Endpoint (Authenticated)"
echo "---------------------------------------------"
wrk -t$THREADS -c$CONNECTIONS -d$DURATION \
    -H "Authorization: Bearer $API_TOKEN" \
    "$BASE_URL/v1/models"
echo ""

# Chat completions endpoint benchmark (requires auth and POST)
echo "📊 Benchmark: Chat Completions Endpoint"
echo "----------------------------------------"
cat > /tmp/chat_body.json << EOF
{
    "model": "gpt-3.5-turbo",
    "messages": [
        {"role": "user", "content": "Hello"}
    ],
    "max_tokens": 10
}
EOF

wrk -t$THREADS -c$CONNECTIONS -d$DURATION \
    -H "Authorization: Bearer $API_TOKEN" \
    -H "Content-Type: application/json" \
    --post /tmp/chat_body.json \
    "$BASE_URL/v1/chat/completions"
echo ""

# Cleanup
rm -f /tmp/chat_body.json

echo "✅ Benchmark completed!"
