// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {ZkCoProcessorConfig, ZkCoProcessorType, INitroEnclaveVerifier} from "../src/interfaces/INitroEnclaveVerifier.sol";
import {NitroEnclaveVerifier} from "../src/NitroEnclaveVerifier.sol";
import {SP1Verifier} from "@sp1-contracts/v5.0.0/SP1VerifierGroth16.sol";
import {ControlID, RiscZeroGroth16Verifier} from "@risc0-ethereum/groth16/RiscZeroGroth16Verifier.sol";
import {SP1VerifierGateway} from "@sp1-contracts/SP1VerifierGateway.sol";
import {LibString} from "@solady/utils/LibString.sol";
import {Ownable} from "@solady/auth/Ownable.sol";

contract NitroEnclaveVerifierScript is Script {
    using LibString for string;
    using LibString for uint256;

    function setUp() public {}

    function readDeployed(string memory key) internal view returns (address) {
        address addr = vm.envOr(key, address(0));
        if (addr != address(0)) {
            console.log(string(abi.encodePacked("read ", key, " from env:")), addr);
            return addr;
        }

        string memory fp = string(abi.encodePacked("deployments/", block.chainid.toString(), ".json"));
        if (vm.exists(fp)) {
            string memory deployment = vm.readFile(fp);
            string memory jsonKey = string(abi.encodePacked(".", key));
            if (vm.keyExistsJson(deployment, jsonKey)) {
                addr = vm.parseJsonAddress(deployment, jsonKey);
                console.log(string(abi.encodePacked("read ", key, " from deployment:")), addr);
                return addr;
            }
        }

        revert(string(abi.encodePacked("No deployment found for ", key, " from file or env, chainid:", block.chainid.toString())));
    }

    function isDeployed(string memory key) internal view returns (bool) {
        string memory fp = string(abi.encodePacked("deployments/", block.chainid.toString(), ".json"));
        if (vm.exists(fp)) {
            string memory deployment = vm.readFile(fp);
            string memory jsonKey = string(abi.encodePacked(".", key));
            return vm.keyExistsJson(deployment, jsonKey);
        }

        return false;
    }

    function saveDeployed(string memory key, address addr) internal {
        string memory fp = string(abi.encodePacked("deployments/", block.chainid.toString(), ".json"));
        string memory deployment = "{}";
        if (vm.exists(fp)) {
            deployment = vm.readFile(fp);
            string[] memory keys = vm.parseJsonKeys(deployment, ".");
            for (uint256 i = 0; i < keys.length; i++) {
                string memory keyPath = string(abi.encodePacked(".", keys[i]));
                vm.serializeAddress(deployment, keys[i], vm.parseJsonAddress(deployment, keyPath));
            }
        }
        deployment = vm.serializeAddress(deployment, key, addr);
        console.log(string(abi.encodePacked("save file ", fp, ": ", deployment)));
        vm.writeFile(fp, deployment);
    }

    function deploySP1Verifier() public {
        vm.startBroadcast();
        SP1Verifier sp1Verifier = new SP1Verifier();
        vm.stopBroadcast();
        require(address(sp1Verifier).code.length > 0, "SP1Verifier deployment failed");
        console.log("SP1Verifier deployed at", address(sp1Verifier));
        saveDeployed("SP1_VERIFIER", address(sp1Verifier));
    }

    function deployRisc0Verifier() public {
        vm.startBroadcast();
        RiscZeroGroth16Verifier risc0Verifier = new RiscZeroGroth16Verifier(ControlID.CONTROL_ROOT, ControlID.BN254_CONTROL_ID);
        vm.stopBroadcast();
        require(address(risc0Verifier).code.length > 0, "Risc0Verifier deployment failed");
        console.log("Risc0Verifier deployed at", address(risc0Verifier));
        saveDeployed("RISC0_VERIFIER", address(risc0Verifier));
    }

    function needsRedeploy(string memory key, bytes memory localBytecode) internal view returns (bool) {
        if (!isDeployed(key)) {
            console.log(string(abi.encodePacked(key, " not deployed yet")));
            return true;
        }
        
        address deployedAddr = readDeployed(key);
        bytes memory chainBytecode = deployedAddr.code;
        
        if (chainBytecode.length == 0) {
            console.log(string(abi.encodePacked(key, " address has no code, needs redeploy")));
            return true;
        }
        
        bytes32 localHash = keccak256(localBytecode);
        bytes32 chainHash = keccak256(chainBytecode);
        
        if (localHash != chainHash) {
            console.log(string(abi.encodePacked(key, " bytecode changed, needs redeploy")));
            console.log("Local bytecode hash:");
            console.logBytes32(localHash);
            console.log("Chain bytecode hash:");
            console.logBytes32(chainHash);
            return true;
        }
        
        console.log(string(abi.encodePacked(key, " bytecode matches, skip deployment")));
        return false;
    }

    function deployVerifier() public returns (bool) {
        string memory configPath = "deploy-config.json";
        string memory config = vm.readFile(configPath);
        uint64 maxTimeDiff = uint64(vm.parseJsonUint(config, ".deployment.maxTimeDiff"));
        
        bytes memory localBytecode = type(NitroEnclaveVerifier).runtimeCode;
        
        if (!needsRedeploy("VERIFIER", localBytecode)) {
            console.log("Skip NitroEnclaveVerifier deployment, using existing:", readDeployed("VERIFIER"));
            return false;
        }
        
        vm.startBroadcast();
        NitroEnclaveVerifier verifier = new NitroEnclaveVerifier(msg.sender, maxTimeDiff, new bytes32[](0));
        vm.stopBroadcast();
        require(address(verifier).code.length > 0, "NitroEnclaveVerifier deployment failed");
        console.log("NitroEnclaveVerifier deployed at", address(verifier));
        console.log("maxTimeDiff:", maxTimeDiff);
        saveDeployed("VERIFIER", address(verifier));
        return true;
    }

    function deployAll(string memory rootCert, string memory sp1Program, string memory risc0Program) public {
        if (deployVerifier()) {
            setRootCert(rootCert);
        }
        setZkVerifier(sp1Program);
        setZkVerifier(risc0Program);
    }

    function setRootCert(string memory path) public {
        INitroEnclaveVerifier verifier = INitroEnclaveVerifier(readDeployed("VERIFIER"));
        bytes memory _rootCert = vm.readFileBinary(path);
        vm.startBroadcast();
        verifier.setRootCert(sha256(_rootCert));
        vm.stopBroadcast();
        console.log("Root certificate set to");
        console.logBytes32(sha256(_rootCert));
    }

    function _getZkType(string memory zktype) internal pure returns (ZkCoProcessorType zkType) {
        if (zktype.eq("Succinct")) {
            zkType = ZkCoProcessorType.Succinct;
        } else if (zktype.eq("RiscZero")) {
            zkType = ZkCoProcessorType.RiscZero;
        } else {
            revert("unknown zkType");
        }
    }

    function setZkVerifier(string memory path) public {
        string memory proofJson = vm.readFile(path);
        bytes32 verifierId = vm.parseJsonBytes32(proofJson, ".program_id.verifier_id");
        bytes32 verifierProofId = vm.parseJsonBytes32(proofJson, ".program_id.verifier_proof_id");
        bytes32 aggregatorId = vm.parseJsonBytes32(proofJson, ".program_id.aggregator_id");
        string memory zktype = vm.parseJsonString(proofJson, ".zktype");
        ZkCoProcessorType zkType = _getZkType(zktype);

        console.log("Setting zk verifier for: ", zktype);
        console.log("Verifier ID: ");
        console.logBytes32(verifierId);
        console.log("Verifier Proof ID: ");
        console.logBytes32(verifierProofId);
        console.log("Aggregator ID: ");
        console.logBytes32(aggregatorId);

        ZkCoProcessorConfig memory config = ZkCoProcessorConfig({
            verifierId: verifierId,
            aggregatorId: aggregatorId,
            zkVerifier: address(0)
        });
        if (zkType == ZkCoProcessorType.RiscZero) {
            config.zkVerifier = readDeployed("RISC0_VERIFIER");
        } else if (zkType == ZkCoProcessorType.Succinct) {
            config.zkVerifier = readDeployed("SP1_VERIFIER");
        } else {
            revert("unknown zkType");
        }

        INitroEnclaveVerifier verifier = INitroEnclaveVerifier(readDeployed("VERIFIER"));
        ZkCoProcessorConfig memory remoteConfig = verifier.getZkConfig(zkType);
        bytes32 remoteVerifierProofId = verifier.getVerifierProofId(zkType, remoteConfig.verifierId);

        if (remoteConfig.verifierId == config.verifierId
            && remoteVerifierProofId == verifierProofId
            && remoteConfig.aggregatorId == config.aggregatorId
            && remoteConfig.zkVerifier == config.zkVerifier) {
            console.log(string(abi.encodePacked(zktype, " configuration matches remote, skip update")));
            return;
        }

        console.log(string(abi.encodePacked(zktype, " configuration differs, updating...")));
        vm.startBroadcast();
        verifier.setZkConfiguration(zkType, config, verifierProofId);
        vm.stopBroadcast();
    }

    function verify(string memory path) public {
        string memory proofJson = vm.readFile(path);
        bytes memory journal = vm.parseJsonBytes(proofJson, ".raw_proof.journal");
        bytes memory proof = vm.parseJsonBytes(proofJson, ".onchain_proof");
        string memory proofType = vm.parseJsonString(proofJson, ".proof_type");
        if (!proofType.eq("Verifier")) {
            revert(string(abi.encodePacked("Unsupported proof type: ", proofType, ", please use batchVerify() instead.")));
        }
        ZkCoProcessorType zkType = _getZkType(vm.parseJsonString(proofJson, ".zktype"));

        uint256 gas = gasleft();
        INitroEnclaveVerifier(readDeployed("VERIFIER")).verify(journal, zkType, proof);
        console.log(path, "verify gas:", gas - gasleft());
    }

    function batchVerify(string memory path) public {
        string memory proofJson = vm.readFile(path);
        bytes memory journal = vm.parseJsonBytes(proofJson, ".raw_proof.journal");
        bytes memory proof = vm.parseJsonBytes(proofJson, ".onchain_proof");
        ZkCoProcessorType zkType = _getZkType(vm.parseJsonString(proofJson, ".zktype"));
        string memory proofType = vm.parseJsonString(proofJson, ".proof_type");
        if (!proofType.eq("Aggregator")) {
            revert(string(abi.encodePacked("Unsupported proof type: ", proofType, ", please use verify() instead.")));
        }

        uint256 gas = gasleft();
        INitroEnclaveVerifier(readDeployed("VERIFIER")).batchVerify(journal, zkType, proof);
        console.log(path, "batch verify gas:", gas - gasleft());
    }
}
