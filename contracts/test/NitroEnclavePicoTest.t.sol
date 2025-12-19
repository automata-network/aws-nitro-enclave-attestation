// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import {Test} from "forge-std/Test.sol";
import {
    NitroEnclaveVerifier,
    ZkCoProcessorType,
    ZkCoProcessorConfig,
    VerifierJournal
} from "../src/NitroEnclaveVerifier.sol";
import {PicoVerifier} from "../src/pico/PicoVerifier.sol";

contract NitroEnclavePicoTest is Test {
    NitroEnclaveVerifier verifier;
    PicoVerifier picoVerifier;
    address admin = address(0x01);
    string picoInputJson = vm.readFile(string.concat(vm.projectRoot(), "/test/assets/inputs.json"));
    string picoAggregatedInputJson =
        vm.readFile(string.concat(vm.projectRoot(), "/test/assets/inputs-aggregated.json"));

    function setUp() public {
        vm.startPrank(admin);

        // deploy contracts
        verifier = new NitroEnclaveVerifier(admin, 27900202, new bytes32[](0));
        picoVerifier = new PicoVerifier();

        // add root certificate
        bytes memory awsRoot = vm.readFileBinary(string.concat(vm.projectRoot(), "/test/assets/aws_root.der"));
        verifier.setRootCert(sha256(awsRoot));

        // configure Pico zkVerifier
        bytes32 nitroPicoVerifierVkey = abi.decode(vm.parseJson(picoInputJson, ".riscvVKey"), (bytes32));
        bytes32 nitroPicoAggregatorVkey = abi.decode(vm.parseJson(picoAggregatedInputJson, ".riscvVKey"), (bytes32));
        bytes32 nitroPicoVerifierProofId = 0x38a3d34f08d8af64b947e861eb80b8404affdf756add5f577e79931598ba585a; // not in the input.json but you can get this from OnchainProof returned by the CLI
        ZkCoProcessorConfig memory picoConfig = ZkCoProcessorConfig({
            verifierId: nitroPicoVerifierVkey, aggregatorId: nitroPicoAggregatorVkey, zkVerifier: address(picoVerifier)
        });
        verifier.setZkConfiguration(ZkCoProcessorType.Pico, picoConfig, nitroPicoVerifierProofId);

        vm.stopPrank();
    }

    function testVerifyNitroPicoProof() public {
        // prevent InvalidTimestamp errors
        vm.warp(1723799509);

        (, bytes memory publicValues, uint256[8] memory proofArray) = _parsePicoInput(picoInputJson);

        VerifierJournal memory journal = verifier.verify(publicValues, ZkCoProcessorType.Pico, abi.encode(proofArray));

        assertEq(uint8(journal.result), uint8(0));
    }

    function testBatchVerifyNitroPicoProof() public {
        // prevent InvalidTimestamp errors
        vm.warp(1723799509);

        (, bytes memory publicValues, uint256[8] memory proofArray) = _parsePicoInput(picoAggregatedInputJson);

        VerifierJournal[] memory journals =
            verifier.batchVerify(publicValues, ZkCoProcessorType.Pico, abi.encode(proofArray));

        uint256 n = journals.length;
        for (uint256 i = 0; i < n; i++) {
            assertEq(uint8(journals[i].result), uint8(0));
        }
    }

    function _parsePicoInput(string memory json)
        private
        pure
        returns (bytes32 riscvVKey, bytes memory publicValues, uint256[8] memory proofArray)
    {
        riscvVKey = abi.decode(vm.parseJson(json, ".riscvVKey"), (bytes32));
        publicValues = abi.decode(vm.parseJson(json, ".publicValues"), (bytes));

        bytes32[] memory proofBytes32 = abi.decode(vm.parseJson(json, ".proof"), (bytes32[]));
        for (uint256 i = 0; i < 8; i++) {
            proofArray[i] = uint256(proofBytes32[i]);
        }
    }
}
