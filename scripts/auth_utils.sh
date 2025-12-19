#!/bin/bash
# Shared authentication utilities for forge scripts

# Global variables set by setup_auth
AUTH_FLAGS=""
export OWNER=""

# Setup authentication mode
# Usage: setup_auth "$KEYSTORE_PATH"
# Sets AUTH_FLAGS to either "--private-key $PRIVATE_KEY" or "--keystore $path"
# Also derives and exports OWNER address
setup_auth() {
    local keystore_path="$1"

    if [ -n "$keystore_path" ]; then
        # Keystore mode
        if [ ! -f "$keystore_path" ]; then
            echo "Error: Keystore file not found: $keystore_path"
            exit 1
        fi

        # Prompt for password once
        echo -n "Enter keystore password: "
        read -s KEYSTORE_PASSWORD
        echo ""

        # Derive address from keystore
        OWNER=$(cast wallet address --keystore "$keystore_path" --password "$KEYSTORE_PASSWORD")
        if [ $? -ne 0 ] || [ -z "$OWNER" ]; then
            echo "Error: Failed to unlock keystore. Check your password."
            exit 1
        fi
        export OWNER

        AUTH_FLAGS="--keystore $keystore_path --password $KEYSTORE_PASSWORD"
        echo "Using keystore authentication: $keystore_path"
        echo "Owner address: $OWNER"
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
        OWNER=$(cast wallet address --private-key "$PRIVATE_KEY")
        export OWNER
        echo "Using private key authentication"
        echo "Owner address: $OWNER"
    fi
}
