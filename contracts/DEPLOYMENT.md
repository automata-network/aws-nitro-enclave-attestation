# Multi-Chain Deployment Guide

This guide explains how to deploy NitroEnclaveVerifier contracts across multiple blockchain networks using the automated deployment tools.

## Prerequisites

1. **Install Dependencies**
   ```bash
   # Foundry
   curl -L https://foundry.paradigm.xyz | bash
   foundryup
   
   # jq (for JSON processing)
   # macOS
   brew install jq
   # Ubuntu/Debian
   sudo apt-get install jq
   ```

2. **Set Environment Variables**
   ```bash
   export PRIVATE_KEY=your_private_key_here
   export ETHERSCAN_API_KEY=your_etherscan_api_key_here  # Optional, for contract verification
   ```

3. **Prepare Program IDs**
   ```bash
   # Generate SP1 and RISC0 program IDs
   cd ..
   cargo install --path crates/nitro-attest-cli
   nitro-attest-cli upload --sp1 --out samples/sp1_program_id.json
   nitro-attest-cli upload --risc0 --out samples/risc0_program_id.json
   ```

## Deployment Configuration

All deployment configurations are stored in `contracts/deploy-config.json`. This file contains:

- **chains**: List of supported blockchain networks with RPC URLs and explorers
- **deployment**: Configuration for root certificate, program IDs, and verifier settings
- **verifiers**: Official SP1 and RISC0 verifier contract addresses for each chain

### Supported Networks

**Mainnet:**
- Ethereum Mainnet
- Base
- Optimism
- Arbitrum One
- Polygon
- BNB Smart Chain
- Avalanche C-Chain
- Automata
- World Chain

**Testnet:**
- Sepolia
- Base Sepolia
- Optimism Sepolia
- Arbitrum Sepolia
- Polygon Amoy
- BNB Smart Chain Testnet
- Avalanche Fuji
- Automata Testnet
- Unichain Sepolia

## Deployment Methods

### Method 1: Deploy to a Single Chain

```bash
cd contracts
../scripts/multi_chain_deploy.sh --chain sepolia
```

### Method 2: Deploy to Multiple Specific Chains

```bash
../scripts/multi_chain_deploy.sh --multiple sepolia,base-sepolia,arbitrum-sepolia
```

### Method 3: Deploy to All Chains

```bash
../scripts/multi_chain_deploy.sh --all
```

### Method 4: Dry Run (Simulation)

Test deployment without broadcasting transactions:

```bash
../scripts/multi_chain_deploy.sh --chain sepolia --dry-run
```

### Method 5: List Available Chains

```bash
../scripts/multi_chain_deploy.sh --list
```

## Deployment Process

For each chain, the deployment script will:

1. **Check for existing verifiers**: Uses official SP1 Gateway and RISC0 verifier addresses if available
2. **Deploy verifiers if needed**: Deploys new verifiers if not using official ones. The SP1 fallback deploys the bundled v6.1.0 Groth16 verifier.
3. **Deploy NitroEnclaveVerifier**: Deploys the main attestation verifier contract
4. **Set root certificate**: Configures the AWS Nitro root certificate
5. **Configure ZK verifiers**: Sets up SP1 and RISC0 program IDs and verifier addresses
6. **Save deployment info**: Stores deployment addresses in `deployments/{chainId}.json`

## Deployment Artifacts

After deployment, you'll find:

- **`deployments/{chainId}.json`**: Individual deployment files for each chain
  ```json
  {
    "VERIFIER": "0x...",
    "SP1_VERIFIER": "0x...",
    "RISC0_VERIFIER": "0x..."
  }
  ```

- **Broadcast logs**: Transaction details in `broadcast/MultiChainDeploy.s.sol/{chainId}/`

## Generating Deployment Summary

After deploying to multiple chains, generate a comprehensive deployment summary:

```bash
cd contracts
../scripts/generate_deployment_summary.sh > DEPLOYMENTS.md
```

This creates a markdown file with:
- All deployment addresses organized by mainnet/testnet
- Links to block explorers
- Chain IDs and network names

## Verifying Contracts

Contract verification is automatically attempted if `ETHERSCAN_API_KEY` is set. For manual verification:

```bash
forge verify-contract \
  --chain-id 11155111 \
  --constructor-args $(cast abi-encode "constructor(uint256,bytes32[])" 10800 "[]") \
  CONTRACT_ADDRESS \
  src/NitroEnclaveVerifier.sol:NitroEnclaveVerifier \
  --etherscan-api-key $ETHERSCAN_API_KEY
```

## Updating Existing Deployments

To update configuration on an already deployed contract:

```bash
# Update root certificate
forge script script/NitroEnclaveVerifier.s.sol \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY \
  --broadcast \
  --sig 'setRootCert(string)' \
  ../samples/aws_root.der

# Update ZK verifier configuration
forge script script/NitroEnclaveVerifier.s.sol \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY \
  --broadcast \
  --sig 'setZkVerifier(string)' \
  ../samples/sp1_program_id.json
```

## Troubleshooting

### Issue: "PRIVATE_KEY environment variable is not set"

**Solution**: Export your private key
```bash
export PRIVATE_KEY=0x...
```

### Issue: "Chain not found in deploy-config.json"

**Solution**: Add the chain configuration to `deploy-config.json`:
```json
{
  "chains": {
    "your-chain": {
      "chainId": 12345,
      "rpc": "https://rpc.your-chain.io",
      "explorer": "https://explorer.your-chain.io"
    }
  }
}
```

### Issue: "Deployment failed due to insufficient funds"

**Solution**: Ensure your deployment address has enough native tokens for:
- Contract deployment gas
- Configuration transaction gas
- Add a buffer of ~0.1 ETH (or equivalent) per chain

### Issue: "Verifier already deployed but not in deployments file"

**Solution**: Manually add the address to the appropriate `deployments/{chainId}.json` file:
```json
{
  "VERIFIER": "0xExistingAddress..."
}
```

## Advanced Usage

### Using Different Root Certificates

To use a different root certificate:

1. Place your certificate in `samples/your_root.der`
2. Update `deploy-config.json`:
   ```json
   {
     "deployment": {
       "rootCert": "../samples/your_root.der"
     }
   }
   ```

### Adding New Chains

1. Add chain configuration to `deploy-config.json`
2. Add verifier addresses if using official deployments
3. Run deployment script with your new chain

## Gas Estimates

Approximate gas costs per chain (as of 2025):

| Network | NitroEnclaveVerifier | SP1Verifier | RISC0Verifier | Total |
|---------|---------------------|-------------|---------------|-------|
| Ethereum | ~2.5M gas | ~4M gas | ~3M gas | ~9.5M gas |
| L2 Networks | ~2.5M gas | ~4M gas | ~3M gas | ~9.5M gas |

Note: L2 networks have significantly lower gas prices, making deployment more cost-effective.

## Security Considerations

1. **Private Key Management**: Never commit private keys to version control
2. **Verify Contracts**: Always verify contracts on block explorers after deployment
3. **Test on Testnets**: Deploy to testnets first to verify everything works
4. **Multi-Sig Ownership**: Consider transferring ownership to a multi-sig wallet after deployment
5. **Audit Root Certificates**: Ensure you're using the correct AWS Nitro root certificate

## Support

For issues or questions:
- Open an issue: https://github.com/automata-network/aws-nitro-enclave-attestation/issues
- Check existing deployments in `contracts/deployments/`
- Review deployment logs in `contracts/broadcast/`
