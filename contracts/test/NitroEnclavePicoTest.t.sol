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

    function setUp() public {
        // prevent InvalidTimestamp errors
        vm.warp(1723799509);
        
        vm.startPrank(admin);

        // deploy contracts
        verifier = new NitroEnclaveVerifier(admin, 3600 * 3, new bytes32[](0));
        picoVerifier = new PicoVerifier();

        // add root certificate
        bytes memory awsRoot = vm.readFileBinary(string.concat(vm.projectRoot(), "/test/assets/aws_root.der"));
        verifier.setRootCert(sha256(awsRoot));

        // configure Pico zkVerifier
        bytes32 nitroPicoVkey = abi.decode(vm.parseJson(picoInputJson, ".riscvVKey"), (bytes32));
        ZkCoProcessorConfig memory picoConfig = ZkCoProcessorConfig({
            verifierId: nitroPicoVkey,
            verifierProofId: bytes32(0),
            aggregatorId: bytes32(0),
            zkVerifier: address(picoVerifier)
        });
        verifier.setZkConfiguration(ZkCoProcessorType.Pico, picoConfig);

        vm.stopPrank();
    }

    function testVerifyNitroPicoProof() public {
        bytes memory publicValues = abi.decode(vm.parseJson(picoInputJson, ".publicValues"), (bytes));
        bytes32[] memory proofBytes32 = abi.decode(vm.parseJson(picoInputJson, ".proof"), (bytes32[]));

        uint256[8] memory proofArray;
        for (uint256 i = 0; i < 8; i++) {
            proofArray[i] = uint256(proofBytes32[i]);
        }

        VerifierJournal memory journal = verifier.verify(publicValues, ZkCoProcessorType.Pico, abi.encode(proofArray));

        assertEq(uint8(journal.result), uint8(0));
    }
}
