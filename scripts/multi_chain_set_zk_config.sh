#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONTRACTS_DIR="$PROJECT_ROOT/contracts"

cd "$CONTRACTS_DIR"

print_usage() {
    cat << EOF
Multi-Chain ZK Configuration Update Tool for NitroEnclaveVerifier

Usage: $0 [OPTIONS]

Options:
    -c, --chain CHAIN_NAME          Update ZK config on a specific chain (e.g., sepolia, base, arbitrum)
    -m, --multiple CHAIN1,CHAIN2    Update ZK config on multiple chains (comma-separated)
    -a, --all                       Update ZK config on all chains in deploy-config.json
    -t, --type TYPE                 ZK type to update: sp1, risc0, or both (default: both)
    -l, --list                      List all available chains
    -d, --dry-run                   Simulate update without broadcasting transactions
    -h, --help                      Show this help message

Environment Variables:
    PRIVATE_KEY                     Private key for transactions (required)

Examples:
    # Update both SP1 and RISC0 config on Sepolia
    $0 --chain sepolia

    # Update only SP1 config on multiple chains
    $0 --multiple sepolia,base-sepolia --type sp1

    # Update both configs on all chains
    $0 --all

    # Dry run update
    $0 --chain sepolia --dry-run

    # List available chains
    $0 --list

EOF
}

list_chains() {
    echo "Available chains in deploy-config.json:"
    echo "========================================"

    if [ ! -f "deploy-config.json" ]; then
        echo "Error: deploy-config.json not found"
        exit 1
    fi

    chains=$(jq -r '.chains | keys[]' deploy-config.json)

    for chain in $chains; do
        chain_id=$(jq -r ".chains.${chain}.chainId" deploy-config.json)
        rpc=$(jq -r ".chains.${chain}.rpc" deploy-config.json)
        echo "  - $chain (Chain ID: $chain_id)"
        echo "    RPC: $rpc"
    done

    echo ""
}

check_requirements() {
    if [ -z "$PRIVATE_KEY" ]; then
        echo "Error: PRIVATE_KEY environment variable is not set"
        echo "Please set it with: export PRIVATE_KEY=your_private_key"
        exit 1
    fi

    if ! command -v forge &> /dev/null; then
        echo "Error: forge command not found. Please install Foundry:"
        echo "https://getfoundry.sh/"
        exit 1
    fi

    if ! command -v jq &> /dev/null; then
        echo "Error: jq command not found. Please install jq:"
        echo "https://stedolan.github.io/jq/download/"
        exit 1
    fi

    if [ ! -f "deploy-config.json" ]; then
        echo "Error: deploy-config.json not found in $CONTRACTS_DIR"
        exit 1
    fi
}

get_function_name() {
    local target_type=$1
    local scope=$2

    case "$target_type" in
        sp1)
            case "$scope" in
                single) echo "updateSp1ConfigOnChain" ;;
                multiple) echo "updateSp1ConfigOnMultipleChains" ;;
                all) echo "updateSp1ConfigOnAllChains" ;;
            esac
            ;;
        risc0)
            case "$scope" in
                single) echo "updateRisc0ConfigOnChain" ;;
                multiple) echo "updateRisc0ConfigOnMultipleChains" ;;
                all) echo "updateRisc0ConfigOnAllChains" ;;
            esac
            ;;
        both)
            case "$scope" in
                single) echo "updateZkConfigOnChain" ;;
                multiple) echo "updateZkConfigOnMultipleChains" ;;
                all) echo "updateZkConfigOnAllChains" ;;
            esac
            ;;
    esac
}

update_on_chain() {
    local chain_name=$1
    local zk_type=$2
    local dry_run=$3

    echo "=========================================="
    echo "Updating ZK config on: $chain_name"
    echo "ZK Type: $zk_type"
    echo "=========================================="

    local func_name=$(get_function_name "$zk_type" "single")

    local cmd="forge script script/MultiChainSetZkConfig.s.sol:MultiChainSetZkConfigScript \
        --sig '${func_name}(string)' \
        '$chain_name'"

    if [ "$dry_run" != "true" ]; then
        cmd="$cmd --broadcast --private-key $PRIVATE_KEY"
    fi

    eval $cmd

    if [ $? -eq 0 ]; then
        echo "Successfully updated ZK config on $chain_name"
    else
        echo "Failed to update ZK config on $chain_name"
        return 1
    fi
}

update_on_multiple_chains() {
    local chains=$1
    local zk_type=$2
    local dry_run=$3

    IFS=',' read -ra CHAIN_ARRAY <<< "$chains"

    for chain in "${CHAIN_ARRAY[@]}"; do
        chain=$(echo "$chain" | xargs)
        update_on_chain "$chain" "$zk_type" "$dry_run"
        echo ""
    done
}

update_on_all_chains() {
    local zk_type=$1
    local dry_run=$2

    echo "=========================================="
    echo "Updating ZK config on ALL chains"
    echo "ZK Type: $zk_type"
    echo "=========================================="
    echo ""

    local func_name=$(get_function_name "$zk_type" "all")

    local cmd="forge script script/MultiChainSetZkConfig.s.sol:MultiChainSetZkConfigScript \
        --sig '${func_name}()'"

    if [ "$dry_run" != "true" ]; then
        cmd="$cmd --broadcast --private-key $PRIVATE_KEY"
    fi

    eval $cmd
}

CHAIN_NAME=""
MULTIPLE_CHAINS=""
UPDATE_ALL=false
ZK_TYPE="both"
DRY_RUN=false
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
            UPDATE_ALL=true
            shift
            ;;
        -t|--type)
            ZK_TYPE="$2"
            if [[ ! "$ZK_TYPE" =~ ^(sp1|risc0|both)$ ]]; then
                echo "Error: Invalid ZK type '$ZK_TYPE'. Must be sp1, risc0, or both."
                exit 1
            fi
            shift 2
            ;;
        -l|--list)
            LIST_CHAINS=true
            shift
            ;;
        -d|--dry-run)
            DRY_RUN=true
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

if [ "$LIST_CHAINS" = true ]; then
    list_chains
    exit 0
fi

if [ -z "$CHAIN_NAME" ] && [ -z "$MULTIPLE_CHAINS" ] && [ "$UPDATE_ALL" = false ]; then
    echo "Error: No target specified"
    echo ""
    print_usage
    exit 1
fi

check_requirements

if [ "$DRY_RUN" = true ]; then
    echo "DRY RUN MODE - No transactions will be broadcasted"
    echo ""
fi

if [ -n "$CHAIN_NAME" ]; then
    update_on_chain "$CHAIN_NAME" "$ZK_TYPE" "$DRY_RUN"
elif [ -n "$MULTIPLE_CHAINS" ]; then
    update_on_multiple_chains "$MULTIPLE_CHAINS" "$ZK_TYPE" "$DRY_RUN"
elif [ "$UPDATE_ALL" = true ]; then
    update_on_all_chains "$ZK_TYPE" "$DRY_RUN"
fi

echo ""
echo "=========================================="
echo "ZK config update process completed!"
echo "=========================================="
