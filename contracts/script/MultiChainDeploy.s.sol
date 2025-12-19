// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {NitroEnclaveVerifierScript} from "./NitroEnclaveVerifier.s.sol";

contract MultiChainDeployScript is NitroEnclaveVerifierScript {
    using stdJson for string;

    struct ChainConfig {
        string name;
        uint256 chainId;
        string rpc;
        string explorer;
    }

    struct DeploymentConfig {
        string rootCert;
        string sp1ProgramId;
        string risc0ProgramId;
    }

    struct VerifierDeployment {
        string comment;
        mapping(uint256 => address) deployments;
    }

    function loadConfig() internal view returns (string memory) {
        string memory configPath = "deploy-config.json";
        return vm.readFile(configPath);
    }

    function getChainConfig(string memory config, string memory chainName) internal pure returns (ChainConfig memory) {
        string memory basePath = string(abi.encodePacked(".chains.", chainName));

        ChainConfig memory chainConfig;
        chainConfig.name = chainName;
        chainConfig.chainId = config.readUint(string(abi.encodePacked(basePath, ".chainId")));
        chainConfig.rpc = config.readString(string(abi.encodePacked(basePath, ".rpc")));
        chainConfig.explorer = config.readString(string(abi.encodePacked(basePath, ".explorer")));

        return chainConfig;
    }

    function getDeploymentConfig(string memory config) internal pure returns (DeploymentConfig memory) {
        DeploymentConfig memory deployConfig;
        deployConfig.rootCert = config.readString(".deployment.rootCert");
        deployConfig.sp1ProgramId = config.readString(".deployment.sp1ProgramId");
        deployConfig.risc0ProgramId = config.readString(".deployment.risc0ProgramId");

        return deployConfig;
    }

    function getVerifierAddress(string memory config, string memory verifierType, uint256 chainId)
        internal
        view
        returns (address)
    {
        string memory path =
            string(abi.encodePacked(".verifiers.", verifierType, ".deployments.", vm.toString(chainId)));

        if (!vm.keyExistsJson(config, path)) {
            return address(0);
        }

        return config.readAddress(path);
    }

    function deployToChain(string memory chainName) public {
        string memory config = loadConfig();
        ChainConfig memory chainConfig = getChainConfig(config, chainName);
        DeploymentConfig memory deployConfig = getDeploymentConfig(config);

        console.log("==================================================");
        console.log("Deploying to chain:", chainName);
        console.log("Chain ID:", chainConfig.chainId);
        console.log("RPC:", chainConfig.rpc);
        console.log("==================================================");

        vm.createSelectFork(chainConfig.rpc);
        require(block.chainid == chainConfig.chainId, "Chain ID mismatch");

        address sp1Verifier = getVerifierAddress(config, "sp1", chainConfig.chainId);
        address risc0Verifier = getVerifierAddress(config, "risc0", chainConfig.chainId);

        if (sp1Verifier != address(0)) {
            console.log("Using existing SP1 Verifier from config:", sp1Verifier);
            if (!isDeployed("SP1_VERIFIER")) {
                saveDeployed("SP1_VERIFIER", sp1Verifier);
            }
        } else if (isDeployed("SP1_VERIFIER")) {
            console.log("Using existing SP1 Verifier from deployments:", readDeployed("SP1_VERIFIER"));
        } else {
            console.log("SP1 Verifier not configured for this chain, deploying new one...");
            deploySP1Verifier();
        }

        if (risc0Verifier != address(0)) {
            console.log("Using existing RISC0 Verifier from config:", risc0Verifier);
            if (!isDeployed("RISC0_VERIFIER")) {
                saveDeployed("RISC0_VERIFIER", risc0Verifier);
            }
        } else if (isDeployed("RISC0_VERIFIER")) {
            console.log("Using existing RISC0 Verifier from deployments:", readDeployed("RISC0_VERIFIER"));
        } else {
            console.log("RISC0 Verifier not configured for this chain, deploying new one...");
            deployRisc0Verifier();
        }

        if (deployVerifier()) {
            console.log("Setting root certificate...");
            setRootCert(deployConfig.rootCert);
        } else {
            console.log("NitroEnclaveVerifier already deployed:", readDeployed("VERIFIER"));
        }

        console.log("Setting SP1 ZK verifier configuration...");
        setZkVerifier(deployConfig.sp1ProgramId);

        console.log("Setting RISC0 ZK verifier configuration...");
        setZkVerifier(deployConfig.risc0ProgramId);

        console.log("==================================================");
        console.log("Deployment completed for", chainName);
        console.log("NitroEnclaveVerifier:", readDeployed("VERIFIER"));
        console.log(
            "Explorer:",
            string(abi.encodePacked(chainConfig.explorer, "/address/", vm.toString(readDeployed("VERIFIER"))))
        );
        console.log("==================================================\n");
    }

    function deployToMultipleChains(string[] memory chainNames) public {
        for (uint256 i = 0; i < chainNames.length; i++) {
            deployToChain(chainNames[i]);
            console.log("Successfully deployed to", chainNames[i]);
        }
    }

    function deployAll() public {
        string memory config = loadConfig();
        string[] memory keys = vm.parseJsonKeys(config, ".chains");

        console.log("Found", keys.length, "chains in configuration");
        console.log("Starting multi-chain deployment...\n");

        deployToMultipleChains(keys);

        console.log("Multi-chain deployment finished!");
    }
}
