#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONTRACTS_DIR="$PROJECT_ROOT/contracts"

# Load .env from project root
if [ -f "$PROJECT_ROOT/.env" ]; then
    set -a
    source "$PROJECT_ROOT/.env"
    set +a
fi

# Constants
OWNER_ADDRESS="0xC9b9010654694AF1aa538d108e2140E318Fa78fF"

print_usage() {
    cat << EOF
Contract Verification Tool for NitroEnclaveVerifier

Usage: $0 [OPTIONS]

Options:
    -c, --chain CHAIN_NAME      Chain name to verify on (e.g., sepolia, base)
    -v, --verifier TYPE         Verifier type: etherscan (default) or blockscout
    -u, --verifier-url URL      Custom verifier URL (required for blockscout)
    -h, --help                  Show this help message

Environment Variables:
    ETHERSCAN_API_KEY           API key for Etherscan verification (required for etherscan verifier)

Examples:
    # Verify on Sepolia using Etherscan
    $0 --chain sepolia

    # Verify on a chain using Blockscout
    $0 --chain mychain --verifier blockscout --verifier-url https://explorer.mychain.io/api

EOF
}

check_requirements() {
    if ! command -v forge &> /dev/null; then
        echo "Error: forge command not found. Please install Foundry:"
        echo "https://getfoundry.sh/"
        exit 1
    fi

    if ! command -v cast &> /dev/null; then
        echo "Error: cast command not found. Please install Foundry:"
        echo "https://getfoundry.sh/"
        exit 1
    fi

    if ! command -v jq &> /dev/null; then
        echo "Error: jq command not found. Please install jq:"
        echo "https://stedolan.github.io/jq/download/"
        exit 1
    fi

    if [ ! -f "$CONTRACTS_DIR/deploy-config.json" ]; then
        echo "Error: deploy-config.json not found in $CONTRACTS_DIR"
        exit 1
    fi
}

get_chain_config() {
    local chain_name=$1
    local config_file="$CONTRACTS_DIR/deploy-config.json"

    if ! jq -e ".chains[\"$chain_name\"]" "$config_file" > /dev/null 2>&1; then
        echo "Error: Chain '$chain_name' not found in deploy-config.json"
        exit 1
    fi

    CHAIN_ID=$(jq -r ".chains[\"$chain_name\"].chainId" "$config_file")
    RPC_URL=$(jq -r ".chains[\"$chain_name\"].rpc" "$config_file")
}

get_deployment_address() {
    local chain_id=$1
    local deployment_file="$CONTRACTS_DIR/deployments/${chain_id}.json"

    if [ ! -f "$deployment_file" ]; then
        echo "Error: Deployment file not found: $deployment_file"
        exit 1
    fi

    CONTRACT_ADDRESS=$(jq -r '.VERIFIER' "$deployment_file")

    if [ "$CONTRACT_ADDRESS" == "null" ] || [ -z "$CONTRACT_ADDRESS" ]; then
        echo "Error: VERIFIER address not found in deployment file"
        exit 1
    fi
}

get_constructor_args() {
    local config_file="$CONTRACTS_DIR/deploy-config.json"
    local max_time_diff=$(jq -r '.deployment.maxTimeDiff' "$config_file")

    # Encode constructor args: (address owner, uint64 maxTimeDiff, bytes32[] _trustedCerts)
    CONSTRUCTOR_ARGS=$(cast abi-encode "constructor(address,uint64,bytes32[])" \
        "$OWNER_ADDRESS" \
        "$max_time_diff" \
        "[]")
}

verify_contract() {
    local chain_name=$1
    local verifier=$2
    local verifier_url=$3

    echo "=========================================="
    echo "Verifying NitroEnclaveVerifier"
    echo "=========================================="
    echo "Chain: $chain_name"
    echo "Chain ID: $CHAIN_ID"
    echo "Contract: $CONTRACT_ADDRESS"
    echo "Verifier: $verifier"
    if [ -n "$verifier_url" ]; then
        echo "Verifier URL: $verifier_url"
    fi
    echo "=========================================="

    cd "$CONTRACTS_DIR"

    local cmd="forge verify-contract"
    cmd="$cmd --chain-id $CHAIN_ID"
    cmd="$cmd --constructor-args $CONSTRUCTOR_ARGS"
    cmd="$cmd --watch"

    if [ "$verifier" == "etherscan" ]; then
        if [ -z "${ETHERSCAN_API_KEY:-}" ]; then
            echo "Error: ETHERSCAN_API_KEY environment variable is not set"
            exit 1
        fi
        cmd="$cmd --etherscan-api-key $ETHERSCAN_API_KEY"
    elif [ "$verifier" == "blockscout" ]; then
        if [ -z "$verifier_url" ]; then
            echo "Error: --verifier-url is required for blockscout verifier"
            exit 1
        fi
        cmd="$cmd --verifier blockscout"
        cmd="$cmd --verifier-url $verifier_url"
    else
        echo "Error: Unknown verifier type: $verifier"
        exit 1
    fi

    cmd="$cmd $CONTRACT_ADDRESS"
    cmd="$cmd src/NitroEnclaveVerifier.sol:NitroEnclaveVerifier"

    echo ""
    echo "Running: $cmd"
    echo ""

    eval $cmd

    if [ $? -eq 0 ]; then
        echo ""
        echo "=========================================="
        echo "Verification successful!"
        echo "=========================================="
    else
        echo ""
        echo "=========================================="
        echo "Verification failed!"
        echo "=========================================="
        exit 1
    fi
}

# Parse arguments
CHAIN_NAME=""
VERIFIER="etherscan"
VERIFIER_URL=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -c|--chain)
            CHAIN_NAME="$2"
            shift 2
            ;;
        -v|--verifier)
            VERIFIER="$2"
            shift 2
            ;;
        -u|--verifier-url)
            VERIFIER_URL="$2"
            shift 2
            ;;
        -h|--help)
            print_usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            print_usage
            exit 1
            ;;
    esac
done

# Validate arguments
if [ -z "$CHAIN_NAME" ]; then
    echo "Error: --chain is required"
    echo ""
    print_usage
    exit 1
fi

# Main execution
check_requirements
get_chain_config "$CHAIN_NAME"
get_deployment_address "$CHAIN_ID"
get_constructor_args
verify_contract "$CHAIN_NAME" "$VERIFIER" "$VERIFIER_URL"
