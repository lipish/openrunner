#!/bin/bash
# Start both backend and web frontend for debugging

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "Starting OpenRunner development environment..."
echo ""

# Start backend in background
echo "[Backend] Starting Rust server..."
cd "$PROJECT_ROOT"
cargo run &
BACKEND_PID=$!

# Wait a bit for backend to start
sleep 2

# Start frontend
echo "[Frontend] Starting web dev server..."
cd "$PROJECT_ROOT/web"
if [ ! -d "node_modules" ]; then
    npm install
fi
npm run dev &
FRONTEND_PID=$!

# Trap to cleanup on exit
cleanup() {
    echo ""
    echo "Shutting down..."
    kill $BACKEND_PID 2>/dev/null || true
    kill $FRONTEND_PID 2>/dev/null || true
    exit 0
}
trap cleanup INT TERM

# Wait for both
wait
