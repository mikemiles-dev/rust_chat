#!/bin/bash

# Rust Chat Server Startup Script
# Starts the chat server in a tmux session with TLS support

set -e

# ============================================
# Configuration
# ============================================

TMUX_SESSION="chat"
SERVER_ADDR="0.0.0.0:8443"
MAX_CLIENTS="100"
PROJECT_DIR="$HOME/rust_chat"

# ============================================
# Start Server
# ============================================

echo "=== Rust Chat Server Startup ==="
echo ""

# Prompt for domain name if not set via environment variable
if [ -z "$CHAT_DOMAIN" ]; then
    read -p "Enter your domain name (e.g., milesrust.chat): " CHAT_DOMAIN
    if [ -z "$CHAT_DOMAIN" ]; then
        echo "Error: Domain name is required for TLS"
        exit 1
    fi
fi

# TLS Certificate Paths (based on domain)
TLS_CERT_PATH="/etc/letsencrypt/live/$CHAT_DOMAIN/fullchain.pem"
TLS_KEY_PATH="/etc/letsencrypt/live/$CHAT_DOMAIN/privkey.pem"

echo "Using domain: $CHAT_DOMAIN"
echo ""

# Check if project directory exists
if [ ! -d "$PROJECT_DIR" ]; then
    echo "Error: Project directory not found at $PROJECT_DIR"
    echo "Please update PROJECT_DIR in this script"
    exit 1
fi

# Check if session already exists
if tmux has-session -t $TMUX_SESSION 2>/dev/null; then
    echo "Error: tmux session '$TMUX_SESSION' already exists"
    echo ""
    echo "Options:"
    echo "  1. Attach to existing session: tmux attach -t $TMUX_SESSION"
    echo "  2. Kill existing session: tmux kill-session -t $TMUX_SESSION"
    echo "  3. Use a different session name (edit this script)"
    exit 1
fi

# Change to project directory
cd "$PROJECT_DIR"

# Build server
echo "Building server in release mode..."
cargo build --release --bin server

if [ $? -ne 0 ]; then
    echo "Error: Build failed"
    exit 1
fi

echo ""
echo "Starting chat server..."
echo ""
echo "Configuration:"
echo "  Session: $TMUX_SESSION"
echo "  Address: $SERVER_ADDR"
echo "  Max Clients: $MAX_CLIENTS"

# Check if TLS certificates exist
if [ -f "$TLS_CERT_PATH" ] && [ -f "$TLS_KEY_PATH" ]; then
    echo "  TLS: ✓ ENABLED"
    echo "  Cert: $TLS_CERT_PATH"
    echo "  Key: $TLS_KEY_PATH"
    TLS_ENV="TLS_CERT_PATH=$TLS_CERT_PATH TLS_KEY_PATH=$TLS_KEY_PATH"
    CLIENT_CONNECT="tls://$CHAT_DOMAIN:8443"
else
    echo "  TLS: ✗ DISABLED (certificates not found)"
    echo ""
    echo "  WARNING: Server will run WITHOUT encryption!"
    echo "  To enable TLS, run: sudo certbot certonly --standalone -d $CHAT_DOMAIN"
    echo ""
    TLS_ENV=""
    CLIENT_CONNECT="$CHAT_DOMAIN:8443"
fi

echo ""

# Create tmux session and start server
tmux new-session -d -s $TMUX_SESSION \
    "CHAT_SERVER_ADDR=$SERVER_ADDR CHAT_SERVER_MAX_CLIENTS=$MAX_CLIENTS $TLS_ENV ./target/release/server"

# Wait a moment for server to start
sleep 1

# Check if session is still running (server didn't crash immediately)
if ! tmux has-session -t $TMUX_SESSION 2>/dev/null; then
    echo "Error: Server failed to start"
    echo "Check the logs above for errors"
    exit 1
fi

echo "✓ Server started successfully!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Server Management:"
echo "  tmux attach -t $TMUX_SESSION    # View server console"
echo "  tmux ls                          # List all sessions"
echo "  Ctrl+B, then D                   # Detach (keeps running)"
echo "  tmux kill-session -t $TMUX_SESSION  # Stop server"
echo ""
echo "Server Commands (when attached):"
echo "  /help       # Show available commands"
echo "  /list       # List connected users"
echo "  /kick USER  # Kick a user"
echo "  /quit       # Shutdown server"
echo ""
echo "Client Connection:"
echo "  Server: $CLIENT_CONNECT"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "To view server now:"
echo "  tmux attach -t $TMUX_SESSION"
echo ""
