#!/bin/bash
set -e

echo "🐳 Starting Docker containers..."
docker-compose up -d

echo "⏳ Waiting for databases to be ready..."
sleep 10

echo "📊 Initializing test data..."
./init-test-data.sh

echo "🔨 Building the project..."
cargo build --release

echo "🚀 Running DBMSCleaner..."
./target/release/DBMSCleaner

echo "✅ Test completed!"
echo ""
echo "To stop the databases, run: docker-compose down"
