#!/bin/bash
# Shared authentication utilities for forge scripts

# Global variables set by setup_auth
AUTH_FLAGS=""

# Setup authentication mode
# Usage: setup_auth "$KEYSTORE_PATH"
# Sets AUTH_FLAGS to either "--private-key $PRIVATE_KEY" or "--keystore $path"
setup_auth() {
    local keystore_path="$1"

    if [ -n "$keystore_path" ]; then
        # Keystore mode
        if [ ! -f "$keystore_path" ]; then
            echo "Error: Keystore file not found: $keystore_path"
            exit 1
        fi
        AUTH_FLAGS="--keystore $keystore_path"
        echo "Using keystore authentication: $keystore_path"
    else
        # Private key mode
        if [ -z "$PRIVATE_KEY" ]; then
            echo "Error: No authentication method provided"
            echo "Either:"
            echo "  - Set PRIVATE_KEY environment variable, OR"
            echo "  - Use --keystore <path> option"
            exit 1
        fi
        AUTH_FLAGS="--private-key $PRIVATE_KEY"
        echo "Using private key authentication"
    fi
}
