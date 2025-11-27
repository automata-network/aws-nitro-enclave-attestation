// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {console} from "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {NitroEnclaveVerifierScript} from "./NitroEnclaveVerifier.s.sol";

contract MultiChainSetZkConfigScript is NitroEnclaveVerifierScript {
    using stdJson for string;

    struct ChainConfig {
        string name;
        uint256 chainId;
        string rpc;
        string explorer;
    }

    struct DeploymentConfig {
        string sp1ProgramId;
        string risc0ProgramId;
    }

    function loadConfig() internal view returns (string memory) {
        string memory configPath = "deploy-config.json";
        return vm.readFile(configPath);
    }

    function getChainConfig(string memory config, string memory chainName)
        internal
        pure
        returns (ChainConfig memory)
    {
        string memory basePath = string(abi.encodePacked(".chains.", chainName));

        ChainConfig memory chainConfig;
        chainConfig.name = chainName;
        chainConfig.chainId = config.readUint(string(abi.encodePacked(basePath, ".chainId")));
        chainConfig.rpc = config.readString(string(abi.encodePacked(basePath, ".rpc")));
        chainConfig.explorer = config.readString(string(abi.encodePacked(basePath, ".explorer")));

        return chainConfig;
    }

    function getDeploymentConfig(string memory config)
        internal
        pure
        returns (DeploymentConfig memory)
    {
        DeploymentConfig memory deployConfig;
        deployConfig.sp1ProgramId = config.readString(".deployment.sp1ProgramId");
        deployConfig.risc0ProgramId = config.readString(".deployment.risc0ProgramId");

        return deployConfig;
    }

    function updateZkConfigOnChain(string memory chainName) public {
        string memory config = loadConfig();
        ChainConfig memory chainConfig = getChainConfig(config, chainName);
        DeploymentConfig memory deployConfig = getDeploymentConfig(config);

        console.log("==================================================");
        console.log("Updating ZK config on chain:", chainName);
        console.log("Chain ID:", chainConfig.chainId);
        console.log("RPC:", chainConfig.rpc);
        console.log("==================================================");

        vm.createSelectFork(chainConfig.rpc);
        require(block.chainid == chainConfig.chainId, "Chain ID mismatch");

        require(isDeployed("VERIFIER"), "VERIFIER not deployed on this chain");
        require(isDeployed("SP1_VERIFIER"), "SP1_VERIFIER not deployed on this chain");
        require(isDeployed("RISC0_VERIFIER"), "RISC0_VERIFIER not deployed on this chain");

        console.log("NitroEnclaveVerifier:", readDeployed("VERIFIER"));

        console.log("Updating SP1 ZK configuration...");
        setZkVerifier(deployConfig.sp1ProgramId);

        console.log("Updating RISC0 ZK configuration...");
        setZkVerifier(deployConfig.risc0ProgramId);

        console.log("==================================================");
        console.log("ZK config update completed for", chainName);
        console.log("==================================================\n");
    }

    function updateSp1ConfigOnChain(string memory chainName) public {
        string memory config = loadConfig();
        ChainConfig memory chainConfig = getChainConfig(config, chainName);
        DeploymentConfig memory deployConfig = getDeploymentConfig(config);

        console.log("==================================================");
        console.log("Updating SP1 config on chain:", chainName);
        console.log("Chain ID:", chainConfig.chainId);
        console.log("==================================================");

        vm.createSelectFork(chainConfig.rpc);
        require(block.chainid == chainConfig.chainId, "Chain ID mismatch");

        require(isDeployed("VERIFIER"), "VERIFIER not deployed on this chain");
        require(isDeployed("SP1_VERIFIER"), "SP1_VERIFIER not deployed on this chain");

        console.log("NitroEnclaveVerifier:", readDeployed("VERIFIER"));

        console.log("Updating SP1 ZK configuration...");
        setZkVerifier(deployConfig.sp1ProgramId);

        console.log("==================================================");
        console.log("SP1 config update completed for", chainName);
        console.log("==================================================\n");
    }

    function updateRisc0ConfigOnChain(string memory chainName) public {
        string memory config = loadConfig();
        ChainConfig memory chainConfig = getChainConfig(config, chainName);
        DeploymentConfig memory deployConfig = getDeploymentConfig(config);

        console.log("==================================================");
        console.log("Updating RISC0 config on chain:", chainName);
        console.log("Chain ID:", chainConfig.chainId);
        console.log("==================================================");

        vm.createSelectFork(chainConfig.rpc);
        require(block.chainid == chainConfig.chainId, "Chain ID mismatch");

        require(isDeployed("VERIFIER"), "VERIFIER not deployed on this chain");
        require(isDeployed("RISC0_VERIFIER"), "RISC0_VERIFIER not deployed on this chain");

        console.log("NitroEnclaveVerifier:", readDeployed("VERIFIER"));

        console.log("Updating RISC0 ZK configuration...");
        setZkVerifier(deployConfig.risc0ProgramId);

        console.log("==================================================");
        console.log("RISC0 config update completed for", chainName);
        console.log("==================================================\n");
    }

    function updateZkConfigOnMultipleChains(string[] memory chainNames) public {
        for (uint256 i = 0; i < chainNames.length; i++) {
            updateZkConfigOnChain(chainNames[i]);
        }
    }

    function updateSp1ConfigOnMultipleChains(string[] memory chainNames) public {
        for (uint256 i = 0; i < chainNames.length; i++) {
            updateSp1ConfigOnChain(chainNames[i]);
        }
    }

    function updateRisc0ConfigOnMultipleChains(string[] memory chainNames) public {
        for (uint256 i = 0; i < chainNames.length; i++) {
            updateRisc0ConfigOnChain(chainNames[i]);
        }
    }

    function updateZkConfigOnAllChains() public {
        string memory config = loadConfig();
        string[] memory keys = vm.parseJsonKeys(config, ".chains");

        console.log("Found", keys.length, "chains in configuration");
        console.log("Starting multi-chain ZK config update...\n");

        updateZkConfigOnMultipleChains(keys);

        console.log("Multi-chain ZK config update finished!");
    }

    function updateSp1ConfigOnAllChains() public {
        string memory config = loadConfig();
        string[] memory keys = vm.parseJsonKeys(config, ".chains");

        console.log("Found", keys.length, "chains in configuration");
        console.log("Starting multi-chain SP1 config update...\n");

        updateSp1ConfigOnMultipleChains(keys);

        console.log("Multi-chain SP1 config update finished!");
    }

    function updateRisc0ConfigOnAllChains() public {
        string memory config = loadConfig();
        string[] memory keys = vm.parseJsonKeys(config, ".chains");

        console.log("Found", keys.length, "chains in configuration");
        console.log("Starting multi-chain RISC0 config update...\n");

        updateRisc0ConfigOnMultipleChains(keys);

        console.log("Multi-chain RISC0 config update finished!");
    }
}
