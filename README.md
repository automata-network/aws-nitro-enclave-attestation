<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/automata-network/automata-brand-kit/main/PNG/ATA_White%20Text%20with%20Color%20Logo.png">
    <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/automata-network/automata-brand-kit/main/PNG/ATA_Black%20Text%20with%20Color%20Logo.png">
    <img src="https://raw.githubusercontent.com/automata-network/automata-brand-kit/main/PNG/ATA_White%20Text%20with%20Color%20Logo.png" width="50%">
  </picture>
</div>

# AWS Nitro Enclave Attestation SDK
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

A comprehensive SDK for AWS Nitro Enclave attestation verification that generates zero-knowledge proofs for on-chain verification with batch processing capabilities.

## Features

* **Zero-Knowledge Proof Generation**
  * Creates on-chain verifiable zero-knowledge proofs (ZKPs) for attestation reports
  * Supports both single and batch attestation verification
* **Multi-Backend Support**
  * Compatible with both Risc0 and Succinct proving systems, providing a unified user experience
  * Optimized performance profiles for different use cases
* **Batch Verification**
  * Supports attestation report batch verification to significantly reduce on-chain verification costs
* **Smart Contract Integration**
  * Includes on-chain verification contracts for seamless blockchain integration
  * Gas-optimized verification with certificate revocation support
* **Comprehensive CLI Tool**
  * Comprehensive CLI tool for proof generation, verification, and debugging

## Predeploy Contracts

## Mainnet Deployments

| Network | Chain ID | NitroEnclaveVerifier | SP1 Verifier | RISC0 Verifier |
|---------|----------|---------------------|--------------|----------------|
| Ethereum Mainnet | 1 | [`0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B`](https://etherscan.io/address/0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B) | [`0x397A5f7f3dBd538f23DE225B51f532c34448dA9B`](https://etherscan.io/address/0x397A5f7f3dBd538f23DE225B51f532c34448dA9B) | [`0x8EaB2D97Dfce405A1692a21b3ff3A172d593D319`](https://etherscan.io/address/0x8EaB2D97Dfce405A1692a21b3ff3A172d593D319) |
| Optimism | 10 | [`0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B`](https://optimistic.etherscan.io/address/0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B) | [`0x397A5f7f3dBd538f23DE225B51f532c34448dA9B`](https://optimistic.etherscan.io/address/0x397A5f7f3dBd538f23DE225B51f532c34448dA9B) | [`0x0b144E07A0826182B6b59788c34b32Bfa86Fb711`](https://optimistic.etherscan.io/address/0x0b144E07A0826182B6b59788c34b32Bfa86Fb711) |
| Polygon | 137 | [`0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B`](https://polygonscan.com/address/0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B) | [`0x3B6041173B80E77f038f3F2C0f9744f04837185e`](https://polygonscan.com/address/0x3B6041173B80E77f038f3F2C0f9744f04837185e) | [`0xdBAD523786971B75A7b1c1CFdCfECDeb59A764B9`](https://polygonscan.com/address/0xdBAD523786971B75A7b1c1CFdCfECDeb59A764B9) |
| Arbitrum One | 42161 | [`0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B`](https://arbiscan.io/address/0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B) | [`0x397A5f7f3dBd538f23DE225B51f532c34448dA9B`](https://arbiscan.io/address/0x397A5f7f3dBd538f23DE225B51f532c34448dA9B) | [`0x0b144E07A0826182B6b59788c34b32Bfa86Fb711`](https://arbiscan.io/address/0x0b144E07A0826182B6b59788c34b32Bfa86Fb711) |
| Avalanche C-Chain | 43114 | [`0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B`](https://snowtrace.io/address/0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B) | [`0x3B6041173B80E77f038f3F2C0f9744f04837185e`](https://snowtrace.io/address/0x3B6041173B80E77f038f3F2C0f9744f04837185e) | [`0x0b144E07A0826182B6b59788c34b32Bfa86Fb711`](https://snowtrace.io/address/0x0b144E07A0826182B6b59788c34b32Bfa86Fb711) |
| World Chain | 480 | [`0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B`](https://worldscan.org/address/0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B) | [`0x3B6041173B80E77f038f3F2C0f9744f04837185e`](https://worldscan.org/address/0x3B6041173B80E77f038f3F2C0f9744f04837185e) | [`0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187`](https://worldscan.org/address/0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187) |
| BNB Smart Chain | 56 | [`0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B`](https://bscscan.com/address/0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B) | [`0x940467b232cAD6A44FF36F2FBBe98CBd6509EFf2`](https://bscscan.com/address/0x940467b232cAD6A44FF36F2FBBe98CBd6509EFf2) | [`0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187`](https://bscscan.com/address/0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187) |
| Automata | 65536 | [`0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B`](https://explorer.ata.network/address/0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B) | [`0x5d1B5BEd5365942421afD473c121abEAc138A3E1`](https://explorer.ata.network/address/0x5d1B5BEd5365942421afD473c121abEAc138A3E1) | [`0x18Fe24Cf185DD45533b24384666aF3be1b74cC64`](https://explorer.ata.network/address/0x18Fe24Cf185DD45533b24384666aF3be1b74cC64) |
| Base | 8453 | [`0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B`](https://basescan.org/address/0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B) | [`0x397A5f7f3dBd538f23DE225B51f532c34448dA9B`](https://basescan.org/address/0x397A5f7f3dBd538f23DE225B51f532c34448dA9B) | [`0x0b144E07A0826182B6b59788c34b32Bfa86Fb711`](https://basescan.org/address/0x0b144E07A0826182B6b59788c34b32Bfa86Fb711) |

## Testnet Deployments

| Network | Chain ID | NitroEnclaveVerifier | SP1 Verifier | RISC0 Verifier |
|---------|----------|---------------------|--------------|----------------|
| Sepolia | 11155111 | [`0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B`](https://sepolia.etherscan.io/address/0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B) | [`0x397A5f7f3dBd538f23DE225B51f532c34448dA9B`](https://sepolia.etherscan.io/address/0x397A5f7f3dBd538f23DE225B51f532c34448dA9B) | [`0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187`](https://sepolia.etherscan.io/address/0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187) |
| Optimism Sepolia | 11155420 | [`0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B`](https://sepolia-optimistic.etherscan.io/address/0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B) | [`0x397A5f7f3dBd538f23DE225B51f532c34448dA9B`](https://sepolia-optimistic.etherscan.io/address/0x397A5f7f3dBd538f23DE225B51f532c34448dA9B) | [`0xB369b4dd27FBfb59921d3A4a3D23AC2fc32FB908`](https://sepolia-optimistic.etherscan.io/address/0xB369b4dd27FBfb59921d3A4a3D23AC2fc32FB908) |
| Unichain Sepolia | 1301 | [`0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B`](https://sepolia.uniscan.xyz/address/0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B) | [`0x3B6041173B80E77f038f3F2C0f9744f04837185e`](https://sepolia.uniscan.xyz/address/0x3B6041173B80E77f038f3F2C0f9744f04837185e) | [`0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187`](https://sepolia.uniscan.xyz/address/0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187) |
| Automata Testnet | 1398243 | [`0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B`](https://explorer-testnet.ata.network/address/0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B) | [`0x7291752B7c1e0E69adF9801865b25435b0bE4Fc6`](https://explorer-testnet.ata.network/address/0x7291752B7c1e0E69adF9801865b25435b0bE4Fc6) | [`0xaE7F7EC735b6A90366e55f87780b36e7e6Ec3c65`](https://explorer-testnet.ata.network/address/0xaE7F7EC735b6A90366e55f87780b36e7e6Ec3c65) |
| Arbitrum Sepolia | 421614 | [`0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B`](https://sepolia.arbiscan.io/address/0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B) | [`0x397A5f7f3dBd538f23DE225B51f532c34448dA9B`](https://sepolia.arbiscan.io/address/0x397A5f7f3dBd538f23DE225B51f532c34448dA9B) | [`0x0b144E07A0826182B6b59788c34b32Bfa86Fb711`](https://sepolia.arbiscan.io/address/0x0b144E07A0826182B6b59788c34b32Bfa86Fb711) |
| Avalanche Fuji | 43113 | [`0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B`](https://testnet.snowtrace.io/address/0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B) | [`0x3B6041173B80E77f038f3F2C0f9744f04837185e`](https://testnet.snowtrace.io/address/0x3B6041173B80E77f038f3F2C0f9744f04837185e) | [`0x0b144E07A0826182B6b59788c34b32Bfa86Fb711`](https://testnet.snowtrace.io/address/0x0b144E07A0826182B6b59788c34b32Bfa86Fb711) |
| Hoodi | 560048 | [`0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B`](https://hoodi.etherscan.io/address/0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B) | [`0x7DA83eC4af493081500Ecd36d1a72c23F8fc2abd`](https://hoodi.etherscan.io/address/0x7DA83eC4af493081500Ecd36d1a72c23F8fc2abd) | [`0x32Db7dc407AC886807277636a1633A1381748DD8`](https://hoodi.etherscan.io/address/0x32Db7dc407AC886807277636a1633A1381748DD8) |
| Polygon Amoy | 80002 | [`0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B`](https://amoy.polygonscan.com/address/0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B) | [`0x3B6041173B80E77f038f3F2C0f9744f04837185e`](https://amoy.polygonscan.com/address/0x3B6041173B80E77f038f3F2C0f9744f04837185e) | [`0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187`](https://amoy.polygonscan.com/address/0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187) |
| Base Sepolia | 84532 | [`0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B`](https://sepolia.basescan.org/address/0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B) | [`0x397A5f7f3dBd538f23DE225B51f532c34448dA9B`](https://sepolia.basescan.org/address/0x397A5f7f3dBd538f23DE225B51f532c34448dA9B) | [`0x0b144E07A0826182B6b59788c34b32Bfa86Fb711`](https://sepolia.basescan.org/address/0x0b144E07A0826182B6b59788c34b32Bfa86Fb711) |
| BNB Smart Chain Testnet | 97 | [`0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B`](https://testnet.bscscan.com/address/0x08e49F31Ab11b17f3a5BaA36e6744E9B532bC87B) | [`0x3B6041173B80E77f038f3F2C0f9744f04837185e`](https://testnet.bscscan.com/address/0x3B6041173B80E77f038f3F2C0f9744f04837185e) | [`0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187`](https://testnet.bscscan.com/address/0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187) |


| ZkType | Verifier ID | Verifier Proof ID | Aggregator ID |
| ------ | ----------- | ----------------- | ------------- |
| Risc0  | 0x933285e8d3076b68cd7b59e611924a9aa2a0b836033213f5f6527b3fbd8897b4 | 0x933285e8d3076b68cd7b59e611924a9aa2a0b836033213f5f6527b3fbd8897b4 | 0x3b2a92fbc8ed23754da30eeaa6bfd72c6ee1ed7485a62d11c358092aa2ce91a7 |
| SP1    | 0x00643c7149cf335e7ec9d3f3301e69658a7f0ef2bc7546509c257ed8809f28e1 | 0xa4381e329fd7cc73667e3a595896e6019577f8534219d571b1fd4a38e1289f00 | 0x00294928e44f0cdc9c74848c4cafcdb29f733a3bc07408c240be3d5afe750b3e |
<!-- | Pico    | 0x009fa7467192bf60230f423dcc0b880ebebbffe955d7f75a8ac9bcbf5a58ba98 | 0x38a3d34f08d8af64b947e861eb80b8404affdf756add5f577e79931598ba585a | 0x00093dbf39d4986be382e062dcbf34d2bc8105637de89bdaaa588014a9c53e9b | -->

> [!IMPORTANT]
>
> The program IDs above are derived from the ELFs committed under
> `crates/{sp1,risc0}-methods/elf/` and only match a build that consumes
> those ELFs reproducibly (`REPRODUCIBLE_BUILD` unset / `prebuilt` /
> `docker`). A local non-reproducible build will produce different IDs
> and the CLI will print a banner on startup. After regenerating ELFs,
> run `nitro-attest-cli upload --out samples/<vm>_program_id.json [--sp1 | --risc0]`
> to dump the fresh IDs.

> [!NOTE]
>
> Pico zkVM does not currently provide remote provers. For that reason, Pico integration is only available on local for testing purposes.
>
> We advise you use a machine that is equipped with at least 256GB of memory to run the prover.
>
> It took us hours to generate EVM proofs (over Koalabear Field) by running the zkVM on an [Azure NC40ads H100 v5](https://learn.microsoft.com/en-us/azure/virtual-machines/sizes/gpu-accelerated/ncadsh100v5-series?tabs=sizebasic) instance.
>
> Currently the Pico SDK does not support GPU proving, and we believe the proving speed will significantly improve after GPU support is enabled.

## Generating Attestation Reports

This repository does not include the attestation report generation functionality. Please refer to the following resources:

* [aws-nitro-enclaves-sdk-c](https://github.com/aws/aws-nitro-enclaves-sdk-c)
* [Cryptographic attestation](https://docs.aws.amazon.com/enclaves/latest/user/set-up-attestation.html)

## Using the Prover SDK

The AWS Nitro Enclave Attestation Prover provides a comprehensive SDK for generating zero-knowledge proofs of attestation report validity. This SDK supports both RISC0 and SP1 proving systems and can be integrated into your Rust applications.

### Installation

Add the prover to your `Cargo.toml`:

```toml
[dependencies]
aws-nitro-enclave-attestation-prover = { git = "https://github.com/automata-network/aws-nitro-enclave-attestation" }
```

### Examples

<details>
<summary><b>1. Basic Single Attestation Proof</b></summary>

```rust
use aws_nitro_enclave_attestation_prover::{NitroEnclaveProver, ProverConfig};

fn main() -> anyhow::Result<()> {
    // Configure the prover (RISC0 example)
    let config = ProverConfig::risc0();
    
    // Create prover instance
    let prover = NitroEnclaveProver::new(config, None);
    
    // Load attestation report
    let report_bytes = std::fs::read("samples/attestation_1.report")?;
    
    // Generate proof
    let result = prover.prove_attestation_report(report_bytes)?;
    
    // Save proof result
    std::fs::write("proof.json", result.encode_json()?)?;
    
    println!("Proof generated successfully!");
    println!("{}", String::from_utf8_lossy(&result.encode_json()?));
    
    Ok(())
}
```

</details>

<details>
<summary><b>2. Batch Proving with Aggregation</b></summary>

```rust
use aws_nitro_enclave_attestation_prover::{NitroEnclaveProver, ProverConfig};

fn prove_multiple_reports() -> anyhow::Result<()> {
    let config = ProverConfig::sp1();
    let prover = NitroEnclaveProver::new(config, None);
    
    // Load multiple attestation reports
    let reports = vec![
        std::fs::read("samples/attestation_1.report")?,
        std::fs::read("samples/attestation_2.report")?,
    ];
    
    // Generate aggregated proof for all reports
    let reports_count = reports.len();
    let result = prover.prove_multiple_reports(reports)?;
    
    println!("Aggregated proof generated for {} reports", reports_count);
    println!("{}", String::from_utf8_lossy(&result.encode_json()?));
    
    Ok(())
}
```
</details>

<details>
<summary><b>3. Smart Contract Integration</b></summary>

For optimal gas efficiency, integrate with the Nitro Enclave Verifier contract:

```rust
use aws_nitro_enclave_attestation_prover::{
    NitroEnclaveProver, ProverConfig,
    NitroEnclaveVerifierContract
};
use alloy_primitives::Address;

async fn prove_with_contract() -> anyhow::Result<()> {
    // Connect to deployed verifier contract
    let contract_address: Address = "0x1234567890123456789012345678901234567890".parse()?;
    let rpc_url = "https://1rpc.io/holesky";
    let verifier = NitroEnclaveVerifierContract::dial(rpc_url, contract_address, None)?;
    let config = ProverConfig::risc0();
    let prover = NitroEnclaveProver::new(config, Some(verifier));
    
    let report_bytes = std::fs::read("samples/attestation_2.report")?;
    
    // Prove with contract optimization
    let result = prover.prove_attestation_report(report_bytes)?;
    
    // The result.onchain_proof is ready for contract submission
    std::fs::write("proof.json", result.encode_json()?)?;
    
    println!("Aggregation Proof generated successfully!");
    println!("{}", String::from_utf8_lossy(&result.encode_json()?));
    let result = prover.verify_on_chain(&result)?;
    println!("onchain verfication result: {:?}", result);
    
    Ok(())
}
```
</details>

## Getting Started with CLI Tools

### Prerequisites

Ensure you have the following installed:
- [Rust](https://rustup.rs/) — the toolchain version is pinned by
  `rust-toolchain.toml`, `rustup` will fetch it automatically.
- [Foundry](https://getfoundry.sh/) for smart contract development
- [RiscZero](https://dev.risczero.com/api/zkvm/install) — only needed to
  rebuild the RISC0 ELFs locally (`REPRODUCIBLE_BUILD=disable …`); the
  default `prebuilt` mode consumes the committed `risc0-*.elf` and works
  with just Rust installed.
- [Succinct](https://docs.succinct.xyz/docs/sp1/getting-started/install) —
  same note as RiscZero, only needed for local SP1 ELF rebuilds.
- [Docker](https://docs.docker.com/get-docker/) — only required for
  `REPRODUCIBLE_BUILD=docker` (containerized reproducible rebuilds and CI).

<details>
<summary><b>1. Generate Zero-Knowledge Proofs</b></summary>

Generate proofs for single or multiple attestation reports:

```bash
$ cargo install --path crates/nitro-attest-cli
$ export VERIFIER=$(NitroEnclaveVerifier address) RPC_URL=http://localhost:8545
$ export DEV_MODE=true # Enable the dev mode for faster execution and generating fake proof

# Generate proof using SP1 backend
$ nitro-attest-cli prove --sp1 --report samples/attestation_1.report --out proof.json

# Generate proof using RISC0 backend  
$ nitro-attest-cli prove --risc0 --report samples/attestation_1.report --out proof.json

# Batch verification with multiple reports
$ nitro-attest-cli prove --sp1 --report samples/attestation_1.report --report samples/attestation_2.report --out samples/proofs/aggregated_proof.json

# Verify proof on-chain
$ nitro-attest-cli proof verify-on-chain --proof samples/proofs/aggregated_proof.json
```

</details>

<details>
<summary><b>2. Inspect Attestation Reports</b></summary>

Examine the contents of attestation reports for debugging and verification:

```bash
$ nitro-attest-cli debug doc --report samples/attestation_1.report
```

**Example Output:**
```
Doc:
    Module ID: i-07fd4cc4df935eab0-enc01915a74e6ed4aa6
    Timestamp: Aug 16 09:11:49 2024 +00:00(1723799509)
    Digest: SHA384
    PublicKey: 0x5075626c69634b657928343032626137353561336335346339653737643937656233663035663562383232373732326666383631653465633537623137356634636263656135613463343534643437613863316637386466343931373533623931346231313738333335636334326435653332666337323864393932613064333337333662633137336529
    UserData: 0x4175746f6d617461204d50432044656d6f
    Nonce: 0x31323334
    PCR[3]: 0xb0c424e9f3727f78f370d4332f3e6e2bb02a288d9bc3c4697102d70744de0b064366fbb3190402deeb4d144e4ab17d4f
    PCR[4]: 0xdcd9866c46ee2878f5fd80f955c12a8c11de276346846579d0d077933757988144c96dc4c5fb708c20c04a4ee34639ab
Cert Chain:
    [0] Digest: 0x641a0321a3e244efe456463195d606317ed7cdcc3c1756e09893f3c68f79bb5b
        Valid: Oct 28 13:28:05 2019 +00:00(1572269285) - Oct 28 14:28:05 2049 +00:00(2519044085)
    [1] Digest: 0x348cc5b001ba75f7d3733ef512463194fea6781954fd416455699d4deb361acf
        Valid: Aug 15 03:20:59 2024 +00:00(1723692059) - Sep  4 04:20:59 2024 +00:00(1725423659)
    [2] Digest: 0x3792fe9068de61899676dfb2f31bf64a72439cf4883d3216629d3404b727c58d
        Valid: Aug 16 00:33:37 2024 +00:00(1723768417) - Aug 21 13:33:37 2024 +00:00(1724247217)
    [3] Digest: 0xb3b18683c518f2c462cd0252034e6a4758c42907add1880bd29a5e0a79aed71b
        Valid: Aug 16 09:11:11 2024 +00:00(1723799471) - Aug 17 09:11:11 2024 +00:00(1723885871)
    [4] Digest: 0x30941d6b61e8cd57b80a6da3705ec072adaa8acb514fbfd9b54ce3393a257e4f
        Valid: Aug 16 09:11:46 2024 +00:00(1723799506) - Aug 16 12:11:49 2024 +00:00(1723810309)
```

</details>

## Getting Started with On-Chain Verification

The NitroEnclaveVerifier smart contract provides efficient on-chain verification of zero-knowledge proofs generated by the SDK. It supports both single and batch verification modes with advanced certificate caching optimization.

<details>
<summary><b>1. Contract Deployment</b></summary>

Deploy the verifier contract to your target network:

```bash
$ cd contracts

# Deploy the NitroEnclaveVerifier
$ forge script script/NitroEnclaveVerifier.s.sol --rpc-url $RPC_URL --private-key $PRIVATE_KEY --broadcast --sig 'deployVerifier()'

# Deploy the SP1Verifier
# If you want to use the official pre-deployed gateway, please refer to https://github.com/succinctlabs/sp1-contracts/blob/main/contracts/deployments/
# and export SP1_VERIFIER=$sp1VerifierAddr
# Otherwise, deploySP1Verifier() deploys the bundled SP1 v6.1.0 Groth16 verifier.
$ forge script script/NitroEnclaveVerifier.s.sol --rpc-url $RPC_URL --private-key $PRIVATE_KEY --broadcast --sig 'deploySP1Verifier()'

# Deploy the Risc0Verifier
# If you want to use the official pre-deployed contract, please refer to https://github.com/risc0/risc0-ethereum/blob/main/contracts/deployment.toml
# and export RISC0_VERIFIER=$risc0VerifierAddr
$ forge script script/NitroEnclaveVerifier.s.sol --rpc-url $RPC_URL --private-key $PRIVATE_KEY --broadcast --sig 'deployRisc0Verifier()'
```

The contract deployment information will be saved in the deployments folder.

</details>

<details>
<summary><b>2. Contract Configuration</b></summary>

Configure the verifier contract with appropriate settings:

```bash
# set the root cert (required)
$ forge script script/NitroEnclaveVerifier.s.sol --rpc-url $RPC_URL --private-key $PRIVATE_KEY --broadcast --sig 'setRootCert(string)' ../samples/aws_root.der

# Set the zk verifier
# Note: sp1_program_id.json and risc0_program_id.json can be generated by `nitro-attest-cli upload --out ${path} [--sp1 | --risc0]`
$ forge script script/NitroEnclaveVerifier.s.sol --rpc-url $RPC_URL --private-key $PRIVATE_KEY --broadcast --sig 'setZkVerifier(string)' ../samples/sp1_program_id.json # sp1
$ forge script script/NitroEnclaveVerifier.s.sol --rpc-url $RPC_URL --private-key $PRIVATE_KEY --broadcast --sig 'setZkVerifier(string)' ../samples/risc0_program_id.json # risc0
```

</details>

## Development

### Reproducible ELF Builds

The SP1 and RISC0 guest ELFs are committed under
`crates/{sp1,risc0}-methods/elf/` so that downstream builds get the same
program IDs as the deployed verifier contracts. The build mode is
selected by the `REPRODUCIBLE_BUILD` environment variable, parsed at
`build.rs` time:

| Mode | When to use | Toolchain required |
|------|-------------|--------------------|
| `prebuilt` (unset / empty) | Default. Consumes the committed ELFs as-is. Fast, no zkVM toolchain needed. | Rust only |
| `docker` | Reproducibly rebuild the ELFs inside a container and overwrite the committed copies. | Rust + Docker + `sp1up` / `rzup` |
| anything else (e.g. `disable`) | Local non-reproducible rebuild for iteration; program IDs will drift. | Rust + `sp1up` / `rzup` |

```bash
# Default: use committed ELFs
$ cargo build

# Reproducibly rebuild inside Docker (this is what CI verifies)
$ REPRODUCIBLE_BUILD=docker cargo build -p sp1-methods -p risc0-methods

# Local iteration build (NOT reproducible)
$ REPRODUCIBLE_BUILD=disable cargo build
```

When the binary is launched without a reproducible build, the CLI prints
a stderr warning banner.

A GitHub Actions workflow (`.github/workflows/verify-elf.yml`) rebuilds
both ELFs in docker on every PR and fails if the committed binaries drift.

### Building Smart Contracts

We use [Foundry](https://getfoundry.sh/) for smart contract development. If you don't have it installed, please follow the [installation guide](https://getfoundry.sh/introduction/installation).

```bash
# Initialize and update submodules
$ git submodule update --init --recursive

# Navigate to contracts directory and build
$ cd contracts
$ forge build
```

### Project Structure

```
├── samples/               # Sample attestation reports and proofs
├── contracts/              # Smart contracts for on-chain verification
│   ├── src/
│   │   ├── NitroEnclaveVerifier.sol     # Main verifier contract
│   │   └── interfaces/
│   │       └── INitroEnclaveVerifier.sol # Contract interface
│   ├── script/             # Deployment scripts
│   ├── test/               # Contract tests
│   └── lib/                # Contract dependencies
└── crates/                 # Rust workspace crates
     ├── nitro-attest-cli/  # CLI application
     ├── prover/            # Proof generation logic
     ├── verifier/          # Verification utilities
     ├── risc0-methods/     # RISC0-specific zkVM methods
     ├── sp1-methods/       # SP1-specific zkVM methods
     ├── pico-methods/      # Pico-specific zkVM methods
     └── x509-verifier-rust-crypto/ # X509 certificate verification
```

## Performance Benchmarks

This section provides comprehensive performance metrics for both RISC0 and SP1 proving systems, demonstrating the efficiency gains from certificate caching and batch verification.

> [!NOTE]
> Proving a single Nitro Enclave attestation report requires approximately 300M cycles, primarily due to the need to verify certificate chains and document correctness through 6 P384 signature verifications, which constitute the majority of the computational overhead. To reduce ZKP proving costs, we have implemented caching at the contract level, which can reduce P384 signature verifications for a single report to as few as 1 verification. This caching system ensures security while supporting revocation operations - when a certificate is revoked, all related leaf certificate caches are invalidated. The caching system only optimizes certificate chain relationship verification; individual certificate validation (such as time validity) is still performed. The specific optimization results are shown below.

<details>
<summary><b>1. Proving Cycles by Cached Certificate Count</b></summary>

The following table shows how certificate caching reduces computational overhead. When more certificates are cached (trusted certs prefix length), fewer certificates need to be verified in the ZK circuit, resulting in lower cycle counts:

#### RISC0

| Cached Certificates | Proving Cycles | Cycles Improvement |
| ------------------- | -------------- | ------------------ |
| 0 (cache disabled)  | 390,594,560    | Baseline           |
| 1                   | 326,107,136    | 16.5% reduction    |
| 2                   | 261,095,424    | 33.2% reduction    |
| 3                   | 196,083,712    | 49.8% reduction    |
| 4                   | 131,072,000    | 66.4% reduction    |
| 5                   | 66,060,288     | 83.1% reduction    |

#### SP1 (Succinct)
| Cached Certificates | Proving Cycles | Cycles Improvement |
| ------------------- | -------------- | ------------------ |
| 0 (cache disabled)  | 285,573,454    | Baseline           |
| 1                   | 238,129,471    | 16.6% reduction    |
| 2                   | 190,785,832    | 33.2% reduction    |
| 3                   | 143,767,478    | 49.7% reduction    |
| 4                   | 96,838,778     | 66.1% reduction    |
| 5                   | 49,534,287     | 82.6% reduction    |

</details>

<details>
<summary><b>2. Proving Cycles for Proof Aggregation</b></summary>

The following table shows the additional cycles used for aggregation.

#### Succinct (SP1)
| Aggregated Reports | Proving Cycles | Proving Cycles per Report | Cycles Improvement |
| ------------------ | -------------- | ------------------------- | ------------------ |
| 1                  | 1,368,769      | 1,368,769                 | Baseline           |
| 2                  | 1,738,923      | 869,461                   | 36.5% reduction    |
| 5                  | 2,884,700      | 576,940                   | 57.8% reduction    |
| 10                 | 4,830,986      | 483,098                   | 64.7% reduction    |
| 100                | 34,680,156     | 346,801                   | 74.7% reduction    |

> [!NOTE]
> **RISC0 (Boundless) Aggregation Costs**
>
> Unlike SP1 which uses efficient recursion circuits to verify child proofs as "assumptions", Boundless executes full Groth16 proof verification internally within the zkVM. This approach incurs approximately **200+ million cycles per additional proof** verified, meaning aggregation costs scale linearly rather than amortizing.
>
> Before using proof aggregation with Boundless, users should compare the additional proving cycle costs against the gas savings from batching on-chain verifications. In many cases, submitting individual Groth16 proofs for on-chain verification may be more cost-effective than paying for the expensive in-zkVM proof verification.

</details>

<details>
<summary><b>3. On-Chain Verification Gas Costs</b></summary>

Gas costs for verifying attestation proofs on Ethereum mainnet (as of block 21,000,000):

#### Single Attestation Verification

| Backend | Verification Gas |
| ------- | ---------------- |
| RISC0   | 257,741          |
| SP1     | 220,333          |

#### Batch Verification Gas Costs

Batch verification provides significant gas savings compared to individual verifications:

| Attestations | RISC0 Total Gas | SP1 Total Gas |
| ------------ | --------------- | ------------- |
| 1            | 259,482         | 234,098       |
| 2            | 260,376         | 235,218       |
| 5            | 306,731         | 282,276       |
| 10           | 384,552         | 361,372       |

</details>

## Troubleshooting

<details>
<summary>Remote Proving API Key Issues</summary>

**SP1 Network Key Missing:**
```
NETWORK_PRIVATE_KEY environment variable is not set. Please set it to your private key or use the .private_key() method.
```

**RISC0 Boundless Key Missing:**
```
missing BOUNDLESS_PRIVATE_KEY
```

**Solution:**
- For SP1 remote proving: Set `NETWORK_PRIVATE_KEY` environment variable with your SP1 network private key
- For RISC0 remote proving via Boundless: Set the following environment variables:
  - `BOUNDLESS_RPC_URL` — Boundless network RPC URL
  - `BOUNDLESS_PRIVATE_KEY` — Your wallet private key (hex-encoded)
  - Exactly one storage uploader:
    - `PINATA_JWT` (+ optional `PINATA_API_URL`, `IPFS_GATEWAY_URL`) for IPFS via Pinata, or
    - `FILE_PATH` for the local-file uploader (useful for offline / dev setups)

    See also the upstream [Boundless storage providers](https://docs.boundless.network/developers/tutorials/request#storage-providers) docs.
- For local testing without remote proving: Set `DEV_MODE=true` to generate development proofs

```bash
# For SP1 production remote proving
export NETWORK_PRIVATE_KEY=your_sp1_network_private_key

# For RISC0 production remote proving (via Boundless)
export BOUNDLESS_RPC_URL=https://rpc.boundless.xyz
export BOUNDLESS_PRIVATE_KEY=your_wallet_private_key
export PINATA_JWT=your_pinata_jwt           # or FILE_PATH=/path/to/local/store

# Optional Boundless configuration
export BOUNDLESS_VERIFIER_PROGRAM_URL=ipfs://...   # Pre-uploaded verifier ELF URL
export BOUNDLESS_AGGREGATOR_PROGRAM_URL=ipfs://... # Pre-uploaded aggregator ELF URL
export BOUNDLESS_MIN_PRICE=100000                  # Min price in wei per cycle
export BOUNDLESS_MAX_PRICE=1000000                 # Max price in wei per cycle
export BOUNDLESS_TIMEOUT=3600                      # Timeout in seconds
export BOUNDLESS_RAMP_UP_PERIOD=300                # Ramp-up period in seconds

# For development/testing
export DEV_MODE=true
```

</details>

<details>
<summary>Program ID Verification Failed</summary>

```
Error: Program ID verification failed: Failed to verify zkconfig for RiscZero

Caused by:
    Program ID mismatch with on-chain config: want: {verifierId=0x0000000000000000000000000000000000000000000000000000000000000000, verifierProofId=0x0000000000000000000000000000000000000000000000000000000000000000, aggregatorId=0x0000000000000000000000000000000000000000000000000000000000000000}, got: {verifierId=0xe012a57f515c0bd110db51b2887b36d874ad8d0f302d7f2c0562beb74d6b6729, verifierProofId=0xe012a57f515c0bd110db51b2887b36d874ad8d0f302d7f2c0562beb74d6b6729, aggregatorId=0x4d4bd302de3ae57d7de3a37fb6c27c6f6b217e6815af737dacb4ca6e45652494})
```

**Cause:** The NitroEnclaveVerifier contract hasn't been configured with the correct program IDs.

**Solution:** Configure the verifier contract with the appropriate program IDs:

```bash
# Upload and set RISC0 program IDs
nitro-attest-cli upload --risc0 --out samples/risc0_program_id.json
forge script script/NitroEnclaveVerifier.s.sol --rpc-url $RPC_URL --private-key $PRIVATE_KEY --broadcast --sig 'setZkVerifier(string)' samples/risc0_program_id.json

# Upload and set SP1 program IDs  
nitro-attest-cli upload --sp1 --out samples/sp1_program_id.json
forge script script/NitroEnclaveVerifier.s.sol --rpc-url $RPC_URL --private-key $PRIVATE_KEY --broadcast --sig 'setZkVerifier(string)' samples/sp1_program_id.json
```

Refer to [Contract Configuration](#contract-configuration) for complete setup instructions.

</details>

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Support

For questions and support, please open an issue in the GitHub repository.
