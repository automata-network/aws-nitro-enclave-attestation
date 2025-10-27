//SPDX-License-Identifier: Apache2.0
pragma solidity ^0.8.0;

import {Ownable} from "@solady/auth/Ownable.sol";
import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";
import {IRiscZeroVerifier} from "@risc0-ethereum/IRiscZeroVerifier.sol";
import {IPicoVerifier} from "./interfaces/IPicoVerifier.sol";
import {
    INitroEnclaveVerifier,
    ZkCoProcessorType,
    ZkCoProcessorConfig,
    VerifierJournal,
    BatchVerifierJournal,
    VerificationResult
} from "./interfaces/INitroEnclaveVerifier.sol";

/**
 * @title NitroEnclaveVerifier
 * @dev Implementation contract for AWS Nitro Enclave attestation verification using zero-knowledge proofs
 * 
 * This contract provides on-chain verification of AWS Nitro Enclave attestation reports by validating
 * zero-knowledge proofs generated off-chain. It supports both single and batch verification modes
 * and can work with multiple ZK proof systems (RISC Zero and Succinct SP1).
 * 
 * Key features:
 * - Certificate chain management with automatic caching of newly discovered certificates
 * - Timestamp validation with configurable time tolerance
 * - Certificate revocation capabilities for compromised intermediate certificates
 * - Gas-efficient batch verification for multiple attestations
 * - Support for both RISC Zero and SP1 proving systems
 * 
 * Security considerations:
 * - Only the contract owner can manage certificates and configurations
 * - Root certificate is immutable once set (requires owner to change)
 * - Intermediate certificates are automatically cached but can be revoked
 * - Timestamp validation prevents replay attacks within the configured time window
 */
contract NitroEnclaveVerifier is Ownable, INitroEnclaveVerifier {
    /// @dev Configuration mapping for each supported ZK coprocessor type
    mapping(ZkCoProcessorType => ZkCoProcessorConfig) public zkConfig;
    
    /// @dev Mapping of trusted intermediate certificate hashes (excludes root certificate)
    mapping(bytes32 trustedCertHash => bool) public trustedIntermediateCerts;
    
    /// @dev Maximum allowed time difference in seconds for attestation timestamp validation
    uint64 public maxTimeDiff;
    
    /// @dev Hash of the trusted AWS Nitro Enclave root certificate
    bytes32 public rootCert;

    /**
     * @dev Initializes the contract with owner, time tolerance and initial trusted certificates
     * @param _owner Address to be set as the contract owner
     * @param _maxTimeDiff Maximum time difference in seconds for timestamp validation
     * @param _initializeTrustedCerts Array of initial trusted intermediate certificate hashes
     * 
     * Sets the provided address as the contract owner and initializes the trusted certificate set.
     * The root certificate must be set separately after deployment.
     */
    constructor(address _owner, uint64 _maxTimeDiff, bytes32[] memory _initializeTrustedCerts) {
        maxTimeDiff = _maxTimeDiff;
        for (uint256 i = 0; i < _initializeTrustedCerts.length; i++) {
            trustedIntermediateCerts[_initializeTrustedCerts[i]] = true;
        }
        _initializeOwner(_owner);
    }

    /**
     * @dev Revokes a trusted intermediate certificate
     * @param _certHash Hash of the certificate to revoke
     * 
     * Requirements:
     * - Only callable by contract owner
     * - Certificate must exist in the trusted intermediate certificates set
     * 
     * This function allows the owner to revoke compromised intermediate certificates
     * without affecting the root certificate or other trusted certificates.
     */
    function revokeCert(bytes32 _certHash) external onlyOwner {
        if (!trustedIntermediateCerts[_certHash]) {
            revert("Certificate not found in trusted certs");
        }
        delete trustedIntermediateCerts[_certHash];
    }

    /**
     * @dev Checks the prefix length of trusted certificates in each provided certificate chain for reports
     * @param _report_certs Array of certificate chains, each containing certificate hashes
     * @return Array indicating the prefix length of trusted certificates in each chain
     * 
     * For each certificate chain:
     * 1. Validates that the first certificate matches the stored root certificate
     * 2. Counts consecutive trusted certificates starting from the root
     * 3. Stops counting when an untrusted certificate is encountered
     * 
     * This function is used to pre-validate certificate chains before generating proofs,
     * helping to optimize the proving process by determining trusted certificate lengths.
     * Usually called from off-chain
     */
    function checkTrustedIntermediateCerts(bytes32[][] calldata _report_certs) public view returns (uint8[] memory) {
        uint8[] memory results = new uint8[](_report_certs.length);
        bytes32 rootCertHash = rootCert;
        for (uint256 i = 0; i < _report_certs.length; i++) {
            bytes32[] calldata certs = _report_certs[i];
            uint8 trustedCertPrefixLen = 1;
            if (certs[0] != rootCertHash) {
                revert("First certificate must be the root certificate");
            }
            for (uint256 j = 1; j < certs.length; j++) {
                if (!trustedIntermediateCerts[certs[j]]) {
                    break;
                }
                trustedCertPrefixLen += 1;
            }
            results[i] = trustedCertPrefixLen;
        }
        return results;
    }

    /**
     * @dev Sets the trusted root certificate hash
     * @param _rootCert Hash of the AWS Nitro Enclave root certificate
     * 
     * Requirements:
     * - Only callable by contract owner
     * 
     * The root certificate serves as the trust anchor for all certificate chain validations.
     * This should be set to the hash of AWS's root certificate for Nitro Enclaves.
     */
    function setRootCert(bytes32 _rootCert) external onlyOwner {
        rootCert = _rootCert;
    }

    /**
     * @dev Configures zero-knowledge verification parameters for a specific coprocessor
     * @param _zkCoProcessor Type of ZK coprocessor (RiscZero or Succinct)
     * @param _config Configuration parameters including program IDs and verifier address
     * 
     * Requirements:
     * - Only callable by contract owner
     * 
     * This function sets up the necessary parameters for ZK proof verification:
     * - verifierId: Program ID for single attestation verification
     * - verifierProofId: Expected verification key for batch operations
     * - aggregatorId: Program ID for batch/aggregated verification
     * - zkVerifier: Address of the deployed ZK verifier contract
     */
    function setZkConfiguration(ZkCoProcessorType _zkCoProcessor, ZkCoProcessorConfig memory _config)
        external
        onlyOwner
    {
        zkConfig[_zkCoProcessor] = _config;
    }

    /**
     * @dev Retrieves the configuration for a specific coprocessor
     * @param _zkCoProcessor Type of ZK coprocessor (RiscZero or Succinct)
     * @return ZkCoProcessorConfig Configuration parameters including program IDs and verifier address
     */
    function getZkConfig(ZkCoProcessorType _zkCoProcessor) external view returns (ZkCoProcessorConfig memory) {
        return zkConfig[_zkCoProcessor];
    }

    /**
     * @dev Internal function to cache newly discovered trusted certificates
     * @param journal Verification journal containing certificate chain information
     * 
     * This function automatically adds any certificates beyond the trusted length
     * to the trusted intermediate certificates set. This optimizes future verifications
     * by expanding the known trusted certificate set based on successful verifications.
     */
    function _cacheNewCert(VerifierJournal memory journal) internal {
        for (uint256 i = journal.trustedCertsPrefixLen; i < journal.certs.length; i++) {
            bytes32 certHash = journal.certs[i];
            trustedIntermediateCerts[certHash] = true;
        }
    }

    /**
     * @dev Internal function to verify and validate a journal entry
     * @param journal Verification journal to validate
     * @return Updated journal with final verification result
     * 
     * This function performs comprehensive validation:
     * 1. Checks if the initial ZK verification was successful
     * 2. Validates the root certificate matches the trusted root
     * 3. Ensures all trusted certificates are still valid (not revoked)
     * 4. Validates the attestation timestamp is within acceptable range
     * 5. Caches newly discovered certificates for future use
     * 
     * The timestamp validation converts milliseconds to seconds and checks:
     * - Attestation is not too old (timestamp + maxTimeDiff >= block.timestamp)
     * - Attestation is not from the future (timestamp <= block.timestamp)
     */
    function _verifyJournal(VerifierJournal memory journal) internal returns (VerifierJournal memory) {
        if (journal.result != VerificationResult.Success) {
            return journal;
        }
        if (journal.trustedCertsPrefixLen == 0) {
            journal.result = VerificationResult.RootCertNotTrusted;
            return journal;
        }
        // Check every trusted certificate to ensure none have been revoked
        for (uint256 i = 0; i < journal.trustedCertsPrefixLen; i++) {
            bytes32 certHash = journal.certs[i];
            if (i == 0) {
                if (certHash != rootCert) {
                    journal.result = VerificationResult.RootCertNotTrusted;
                    return journal;
                }
                continue;
            }
            if (!trustedIntermediateCerts[certHash]) {
                journal.result = VerificationResult.IntermediateCertsNotTrusted;
                return journal;
            }
        }
        uint64 timestamp = journal.timestamp / 1000;
        if (timestamp + maxTimeDiff < block.timestamp || timestamp > block.timestamp) {
            journal.result = VerificationResult.InvalidTimestamp;
            return journal;
        }
        _cacheNewCert(journal);
        return journal;
    }

    /**
     * @dev Verifies multiple attestation reports in a single batch operation
     * @param output Encoded BatchVerifierJournal containing aggregated verification results
     * @param zkCoprocessor Type of ZK coprocessor used to generate the proof
     * @param proofBytes Zero-knowledge proof data for batch verification
     * @return Array of VerifierJournal results, one for each attestation in the batch
     * 
     * This function provides gas-efficient batch verification by:
     * 1. Using the aggregator program ID for ZK proof verification
     * 2. Validating the batch verifier key matches the expected value
     * 3. Processing each individual attestation through standard validation
     * 4. Returning comprehensive results for all attestations
     * 
     * Batch verification is recommended when processing multiple attestations
     * as it significantly reduces gas costs compared to individual verifications.
     */
    function batchVerify(bytes calldata output, ZkCoProcessorType zkCoprocessor, bytes calldata proofBytes)
        external
        returns (VerifierJournal[] memory)
    {
        bytes32 programId = zkConfig[zkCoprocessor].aggregatorId;
        bytes32 verifierProofId = zkConfig[zkCoprocessor].verifierProofId;
        _verifyZk(zkCoprocessor, programId, output, proofBytes);
        BatchVerifierJournal memory batchJournal = abi.decode(output, (BatchVerifierJournal));
        if (batchJournal.verifierVk != verifierProofId) {
            revert("Verifier VK does not match the expected verifier proof ID");
        }
        for (uint256 i = 0; i < batchJournal.outputs.length; i++) {
            batchJournal.outputs[i] = _verifyJournal(batchJournal.outputs[i]);
        }

        return batchJournal.outputs;
    }

    /**
     * @dev Internal function to verify zero-knowledge proofs using the appropriate coprocessor
     * @param zkCoprocessor Type of ZK coprocessor (RiscZero or Succinct)
     * @param programId Program identifier for the verification program
     * @param output Encoded output data to verify
     * @param proofBytes Zero-knowledge proof data
     */
    function _verifyZk(
        ZkCoProcessorType zkCoprocessor,
        bytes32 programId,
        bytes calldata output,
        bytes calldata proofBytes
    ) internal view {
        address zkVerifier = zkConfig[zkCoprocessor].zkVerifier;
        if (zkCoprocessor == ZkCoProcessorType.RiscZero) {
            IRiscZeroVerifier(zkVerifier).verify(proofBytes, programId, sha256(output));
        } else if (zkCoprocessor == ZkCoProcessorType.Succinct) {
            ISP1Verifier(zkVerifier).verifyProof(programId, output, proofBytes);
        } else if (zkCoprocessor == ZkCoProcessorType.Pico) {
            IPicoVerifier(zkVerifier).verifyPicoProof(
                programId,
                output,
                abi.decode(proofBytes, (uint256[8]))
            );
        } else {
            revert Unknown_Zk_Coprocessor();
        }
    }

    /**
     * @dev Verifies a single attestation report using zero-knowledge proof
     * @param output Encoded VerifierJournal containing the verification result
     * @param zkCoprocessor Type of ZK coprocessor used to generate the proof
     * @param proofBytes Zero-knowledge proof data for the attestation
     * @return VerifierJournal containing the verification result and extracted data
     * 
     * This function performs end-to-end verification of a single attestation:
     * 1. Retrieves the single verification program ID from configuration
     * 2. Verifies the zero-knowledge proof using the specified coprocessor
     * 3. Decodes the verification journal from the output
     * 4. Validates the journal through comprehensive checks
     * 5. Returns the final verification result
     * 
     * The returned journal contains all extracted attestation data including:
     * - Verification status and any error conditions
     * - Certificate chain information and trust levels
     * - User data, nonce, and public key from the attestation
     * - Platform Configuration Registers (PCRs) for integrity measurement
     * - Module ID and timestamp information
     */
    function verify(bytes calldata output, ZkCoProcessorType zkCoprocessor, bytes calldata proofBytes)
        external
        returns (VerifierJournal memory)
    {
        bytes32 programId = zkConfig[zkCoprocessor].verifierId;
        _verifyZk(zkCoprocessor, programId, output, proofBytes);
        VerifierJournal memory journal = abi.decode(output, (VerifierJournal));
        journal = _verifyJournal(journal);
        return journal;
    }
}
