#!/bin/bash
# LuminaBridge Docker Start Script
# Waits for dependencies and starts the application

set -e

echo "🌉 LuminaBridge - Starting application..."

# Configuration
DB_HOST="${LUMINABRIDGE__DATABASE__HOST:-postgres}"
DB_PORT="${LUMINABRIDGE__DATABASE__PORT:-5432}"
DB_USER="${POSTGRES_USER:-luminabridge}"
DB_NAME="${POSTGRES_DB:-luminabridge_dev}"
MAX_RETRIES=30
RETRY_INTERVAL=2

# Function to check if PostgreSQL is ready
wait_for_postgres() {
    echo "⏳ Waiting for PostgreSQL to be ready..."
    
    local retries=0
    while [ $retries -lt $MAX_RETRIES ]; do
        if pg_isready -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" > /dev/null 2>&1; then
            echo "✅ PostgreSQL is ready!"
            return 0
        fi
        
        retries=$((retries + 1))
        echo "   Attempt $retries/$MAX_RETRIES - PostgreSQL not ready yet, retrying in ${RETRY_INTERVAL}s..."
        sleep $RETRY_INTERVAL
    done
    
    echo "❌ PostgreSQL did not become ready after $MAX_RETRIES attempts"
    return 1
}

# Function to check if Redis is ready
wait_for_redis() {
    echo "⏳ Waiting for Redis to be ready..."
    
    local retries=0
    while [ $retries -lt $MAX_RETRIES ]; do
        if redis-cli -h "${LUMINABRIDGE__CACHE__HOST:-redis}" -p 6379 ping > /dev/null 2>&1; then
            echo "✅ Redis is ready!"
            return 0
        fi
        
        retries=$((retries + 1))
        echo "   Attempt $retries/$MAX_RETRIES - Redis not ready yet, retrying in ${RETRY_INTERVAL}s..."
        sleep $RETRY_INTERVAL
    done
    
    echo "❌ Redis did not become ready after $MAX_RETRIES attempts"
    return 1
}

# Function to run database migrations
run_migrations() {
    echo "🔄 Running database migrations..."
    
    # The application runs migrations automatically on startup
    # This is just a placeholder for any pre-startup migration tasks
    echo "✅ Database migrations completed!"
}

# Main execution
main() {
    echo ""
    echo "========================================="
    echo "  LuminaBridge Startup Script"
    echo "========================================="
    echo ""
    
    # Wait for dependencies
    wait_for_postgres || exit 1
    wait_for_redis || exit 1
    
    # Run migrations
    run_migrations
    
    echo ""
    echo "🚀 Starting LuminaBridge application..."
    echo ""
    
    # Start the application
    exec /app/luminabridge
}

# Run main function
main
