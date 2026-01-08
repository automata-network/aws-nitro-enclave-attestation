#!/bin/bash

set -e

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

# Tracking arrays for summary
declare -a VERIFIED_CHAINS=()
declare -a SKIPPED_CHAINS=()
declare -a FAILED_CHAINS=()

print_usage() {
    cat << EOF
Multi-Chain Contract Verification Tool for NitroEnclaveVerifier

Usage: $0 [OPTIONS]

Options:
    -c, --chain CHAIN_NAME          Verify on a specific chain (e.g., sepolia, base)
    -m, --multiple CHAIN1,CHAIN2    Verify on multiple specific chains (comma-separated)
    -a, --all                       Verify on all chains with deployments
    -l, --list                      List all available chains and their deployment status
    -h, --help                      Show this help message

Environment Variables:
    ETHERSCAN_API_KEY               API key for Etherscan verification (required)

Examples:
    # Verify on Sepolia testnet
    $0 --chain sepolia

    # Verify on multiple chains
    $0 --multiple sepolia,base-sepolia,arbitrum-sepolia

    # Verify on all deployed chains
    $0 --all

    # List available chains
    $0 --list

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

    if [ -z "${ETHERSCAN_API_KEY:-}" ]; then
        echo "Error: ETHERSCAN_API_KEY environment variable is not set"
        exit 1
    fi
}

list_chains() {
    echo "Available chains in deploy-config.json:"
    echo "========================================"
    echo ""

    local config_file="$CONTRACTS_DIR/deploy-config.json"
    local chains=$(jq -r '.chains | keys[]' "$config_file")

    printf "%-20s %-12s %-15s\n" "CHAIN" "CHAIN ID" "DEPLOYMENT"
    printf "%-20s %-12s %-15s\n" "-----" "--------" "----------"

    for chain in $chains; do
        local chain_id=$(jq -r ".chains[\"$chain\"].chainId" "$config_file")
        local deployment_file="$CONTRACTS_DIR/deployments/${chain_id}.json"

        local status="Not deployed"
        if [ -f "$deployment_file" ]; then
            local verifier_addr=$(jq -r '.VERIFIER // empty' "$deployment_file")
            if [ -n "$verifier_addr" ]; then
                status="${verifier_addr:0:10}..."
            fi
        fi

        printf "%-20s %-12s %-15s\n" "$chain" "$chain_id" "$status"
    done

    echo ""
}

get_chain_config() {
    local chain_name=$1
    local config_file="$CONTRACTS_DIR/deploy-config.json"

    if ! jq -e ".chains[\"$chain_name\"]" "$config_file" > /dev/null 2>&1; then
        echo "Error: Chain '$chain_name' not found in deploy-config.json"
        return 1
    fi

    CHAIN_ID=$(jq -r ".chains[\"$chain_name\"].chainId" "$config_file")
    RPC_URL=$(jq -r ".chains[\"$chain_name\"].rpc" "$config_file")
    return 0
}

get_deployment_address() {
    local chain_id=$1
    local deployment_file="$CONTRACTS_DIR/deployments/${chain_id}.json"

    if [ ! -f "$deployment_file" ]; then
        return 1
    fi

    CONTRACT_ADDRESS=$(jq -r '.VERIFIER // empty' "$deployment_file")

    if [ -z "$CONTRACT_ADDRESS" ]; then
        return 1
    fi

    return 0
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

verify_chain() {
    local chain_name=$1

    echo ""
    echo "=========================================="
    echo "Verifying: $chain_name"
    echo "=========================================="

    # Get chain config
    if ! get_chain_config "$chain_name"; then
        echo "Skipping: Chain config not found"
        SKIPPED_CHAINS+=("$chain_name (config not found)")
        return 0
    fi

    # Check deployment exists
    if ! get_deployment_address "$CHAIN_ID"; then
        echo "Skipping: No deployment found for chain ID $CHAIN_ID"
        SKIPPED_CHAINS+=("$chain_name (no deployment)")
        return 0
    fi

    echo "Chain ID: $CHAIN_ID"
    echo "Contract: $CONTRACT_ADDRESS"

    cd "$CONTRACTS_DIR"

    # Run verification
    local cmd="forge verify-contract"
    cmd="$cmd --chain-id $CHAIN_ID"
    cmd="$cmd --etherscan-api-key $ETHERSCAN_API_KEY"
    cmd="$cmd --constructor-args $CONSTRUCTOR_ARGS"
    cmd="$cmd --watch"
    cmd="$cmd $CONTRACT_ADDRESS"
    cmd="$cmd src/NitroEnclaveVerifier.sol:NitroEnclaveVerifier"

    echo "Running verification..."

    if eval $cmd; then
        echo "Verification successful for $chain_name"
        VERIFIED_CHAINS+=("$chain_name")
    else
        echo "Verification failed for $chain_name"
        FAILED_CHAINS+=("$chain_name")
    fi

    return 0
}

verify_multiple_chains() {
    local chains=$1

    IFS=',' read -ra CHAIN_ARRAY <<< "$chains"

    for chain in "${CHAIN_ARRAY[@]}"; do
        chain=$(echo "$chain" | xargs)  # Trim whitespace
        verify_chain "$chain"
    done
}

verify_all_chains() {
    local config_file="$CONTRACTS_DIR/deploy-config.json"
    local chains=$(jq -r '.chains | keys[]' "$config_file")

    for chain in $chains; do
        verify_chain "$chain"
    done
}

print_summary() {
    echo ""
    echo "=========================================="
    echo "VERIFICATION SUMMARY"
    echo "=========================================="

    if [ ${#VERIFIED_CHAINS[@]} -gt 0 ]; then
        echo ""
        echo "Verified (${#VERIFIED_CHAINS[@]}):"
        for chain in "${VERIFIED_CHAINS[@]}"; do
            echo "  - $chain"
        done
    fi

    if [ ${#SKIPPED_CHAINS[@]} -gt 0 ]; then
        echo ""
        echo "Skipped (${#SKIPPED_CHAINS[@]}):"
        for chain in "${SKIPPED_CHAINS[@]}"; do
            echo "  - $chain"
        done
    fi

    if [ ${#FAILED_CHAINS[@]} -gt 0 ]; then
        echo ""
        echo "Failed (${#FAILED_CHAINS[@]}):"
        for chain in "${FAILED_CHAINS[@]}"; do
            echo "  - $chain"
        done
    fi

    echo ""
    echo "=========================================="

    # Return non-zero if any failures
    if [ ${#FAILED_CHAINS[@]} -gt 0 ]; then
        return 1
    fi
    return 0
}

# Parse arguments
CHAIN_NAME=""
MULTIPLE_CHAINS=""
VERIFY_ALL=false
LIST_CHAINS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -c|--chain)
            CHAIN_NAME="$2"
            shift 2
            ;;
        -m|--multiple)
            MULTIPLE_CHAINS="$2"
            shift 2
            ;;
        -a|--all)
            VERIFY_ALL=true
            shift
            ;;
        -l|--list)
            LIST_CHAINS=true
            shift
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

# Handle list command separately (doesn't require API key)
if [ "$LIST_CHAINS" = true ]; then
    list_chains
    exit 0
fi

# Validate arguments
if [ -z "$CHAIN_NAME" ] && [ -z "$MULTIPLE_CHAINS" ] && [ "$VERIFY_ALL" = false ]; then
    echo "Error: No verification target specified"
    echo ""
    print_usage
    exit 1
fi

# Main execution
check_requirements
get_constructor_args

if [ -n "$CHAIN_NAME" ]; then
    verify_chain "$CHAIN_NAME"
elif [ -n "$MULTIPLE_CHAINS" ]; then
    verify_multiple_chains "$MULTIPLE_CHAINS"
elif [ "$VERIFY_ALL" = true ]; then
    verify_all_chains
fi

print_summary
