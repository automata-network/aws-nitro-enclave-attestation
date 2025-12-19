//SPDX-License-Identifier: Apache2.0
pragma solidity ^0.8.0;

/**
 * @dev Enumeration of supported zero-knowledge proof coprocessor types
 * Used to specify which proving system to use for attestation verification
 */
enum ZkCoProcessorType {
    Unknown,
    // RISC Zero zkVM proving system
    RiscZero,
    // Succinct SP1 proving system
    Succinct,
    // Pico zkVM proving system
    Pico
}

/**
 * @dev Configuration parameters for a specific zero-knowledge coprocessor
 * Contains all necessary identifiers and addresses for ZK proof verification
 *
 * Note: This struct stores the "latest" (active) program identifiers.
 * Multiple versions can be supported simultaneously via the version management functions.
 */
struct ZkCoProcessorConfig {
    // Latest program ID for single attestation verification
    bytes32 verifierId;
    // Latest program ID for batch/aggregated verification
    bytes32 aggregatorId;
    // Default ZK verifier contract address (can be overridden per route)
    address zkVerifier;
}

/**
 * @dev Input structure for attestation report verification
 * Contains the raw attestation data and trusted certificate chain length
 */
struct VerifierInput {
    // Number of trusted certificates in the chain
    uint8 trustedCertsPrefixLen;
    // Raw AWS Nitro Enclave attestation report (COSE_Sign1 format)
    bytes attestationReport;
}

/**
 * @dev Output structure containing verified attestation data and metadata
 * This represents the journal/output from zero-knowledge proof verification
 */
struct VerifierJournal {
    // Overall verification result status
    VerificationResult result;
    // Number of certificates that were trusted during verification
    uint8 trustedCertsPrefixLen;
    // Attestation timestamp (Unix timestamp in milliseconds)
    uint64 timestamp;
    // Array of certificate hashes in the chain (root to leaf)
    bytes32[] certs;
    // User-defined data embedded in the attestation
    bytes userData;
    // Cryptographic nonce used for replay protection
    bytes nonce;
    // Public key extracted from the attestation
    bytes publicKey;
    // Platform Configuration Registers (integrity measurements)
    Pcr[] pcrs;
    // AWS Nitro Enclave module identifier
    string moduleId;
}

/**
 * @dev Public value (journal) structure for batch verification operations
 * Contains the aggregated results of multiple attestation verifications
 */
struct BatchVerifierJournal {
    // Verification key that was used for batch verification
    bytes32 verifierVk;
    // Array of verified attestation results
    VerifierJournal[] outputs;
}

/**
 * @dev 48-byte data structure for storing PCR values
 * Split into two parts due to Solidity's 32-byte word limitation
 */
struct Bytes48 {
    bytes32 first;
    bytes16 second;
}

/**
 * @dev Platform Configuration Register (PCR) entry
 * PCRs contain cryptographic measurements of the enclave's runtime state
 */
struct Pcr {
    // PCR index number (0-23 for AWS Nitro Enclaves)
    uint64 index;
    // 48-byte PCR measurement value (SHA-384 hash)
    Bytes48 value;
}

/**
 * @dev Enumeration of possible attestation verification results
 * Indicates the outcome of the verification process
 */
enum VerificationResult {
    // Attestation successfully verified
    Success,
    // Root certificate is not in the trusted set
    RootCertNotTrusted,
    // One or more intermediate certificates are not trusted
    IntermediateCertsNotTrusted,
    // Attestation timestamp is outside acceptable range
    InvalidTimestamp
}

/**
 * @title INitroEnclaveVerifier
 * @dev Interface for AWS Nitro Enclave attestation verification using zero-knowledge proofs
 *
 * This interface defines the contract for verifying AWS Nitro Enclave attestation reports
 * on-chain using zero-knowledge proof systems (RISC Zero or Succinct SP1). The verifier
 * validates the cryptographic integrity of attestation reports while maintaining privacy
 * and reducing gas costs through ZK proofs.
 *
 * Key features:
 * - Single and batch attestation verification
 * - Support for multiple ZK proving systems
 * - Multi-version program support for seamless upgrades
 * - Route-based verifier configuration
 * - Certificate chain management and revocation
 * - Timestamp validation with configurable tolerance
 * - Platform Configuration Register (PCR) verification
 */
interface INitroEnclaveVerifier {
    // ============ Errors ============

    /**
     * @dev Error thrown when an unsupported or unknown ZK coprocessor type is used
     */
    error Unknown_Zk_Coprocessor();

    /**
     * @dev Error thrown when a program identifier is not in the supported set
     */
    error InvalidProgramIdentifier(ZkCoProcessorType zkCoProcessor, bytes32 identifier);

    /**
     * @dev Error thrown when attempting to remove the currently active (latest) program ID
     */
    error CannotRemoveLatestProgramId(ZkCoProcessorType zkCoProcessor, bytes32 identifier);

    /**
     * @dev Error thrown when a ZK route has been permanently frozen
     */
    error ZkRouteFrozen(ZkCoProcessorType zkCoProcessor, bytes4 selector);

    /**
     * @dev Error thrown when no ZK verifier is configured for the coprocessor
     */
    error ZkVerifierNotConfigured(ZkCoProcessorType zkCoProcessor);

    // ============ Events ============

    /**
     * @dev Emitted when a new verifier program ID is added/updated
     */
    event VerifierIdUpdated(ZkCoProcessorType indexed zkCoProcessor, bytes32 indexed newId, bytes32 newProofId);

    /**
     * @dev Emitted when a new aggregator program ID is added/updated
     */
    event AggregatorIdUpdated(ZkCoProcessorType indexed zkCoProcessor, bytes32 indexed newId);

    /**
     * @dev Emitted when a program ID is removed from the supported set
     */
    event ProgramIdRemoved(ZkCoProcessorType indexed zkCoProcessor, bytes32 indexed programId, bool isAggregator);

    /**
     * @dev Emitted when a route-specific verifier is added
     */
    event ZkRouteAdded(ZkCoProcessorType indexed zkCoProcessor, bytes4 indexed selector, address verifier);

    /**
     * @dev Emitted when a route is permanently frozen
     */
    event ZkRouteWasFrozen(ZkCoProcessorType indexed zkCoProcessor, bytes4 indexed selector);

    /**
     * @dev Emitted when the proof of attestation has been successfully verified
     */
    event AttestationSubmitted(VerificationResult result, ZkCoProcessorType zkCoProcessor, bytes output);

    /**
     * @dev Emitted when a batched proof has been successfully verified
     * @dev encodedBatched = abi.encode(VerifierJournal[])
     */
    event BatchAttestationSubmitted(bytes32 verifierId, ZkCoProcessorType zkCoProcessor, bytes encodedBatch);

    // ============ Query Functions ============

    /**
     * @dev Returns the maximum allowed time difference for attestation timestamp validation
     * @return Maximum time difference in seconds between attestation time and current block time
     */
    function maxTimeDiff() external view returns (uint64);

    /**
     * @dev Returns the hash of the trusted root certificate
     * @return Hash of the AWS Nitro Enclave root certificate
     */
    function rootCert() external view returns (bytes32);

    /**
     * @dev Retrieves the configuration for a specific coprocessor
     * @param _zkCoProcessor Type of ZK coprocessor (RiscZero or Succinct)
     * @return ZkCoProcessorConfig Configuration parameters including program IDs and verifier address
     */
    function getZkConfig(ZkCoProcessorType _zkCoProcessor) external view returns (ZkCoProcessorConfig memory);

    /**
     * @dev Returns all supported verifier program IDs for a coprocessor
     * @param _zkCoProcessor Type of ZK coprocessor
     * @return Array of all supported verifier program IDs
     */
    function getVerifierIds(ZkCoProcessorType _zkCoProcessor) external view returns (bytes32[] memory);

    /**
     * @dev Returns all supported aggregator program IDs for a coprocessor
     * @param _zkCoProcessor Type of ZK coprocessor
     * @return Array of all supported aggregator program IDs
     */
    function getAggregatorIds(ZkCoProcessorType _zkCoProcessor) external view returns (bytes32[] memory);

    /**
     * @dev Checks if a verifier program ID is in the supported set
     * @param _zkCoProcessor Type of ZK coprocessor
     * @param _verifierId Verifier program ID to check
     * @return True if the ID is supported
     */
    function isVerifierIdSupported(ZkCoProcessorType _zkCoProcessor, bytes32 _verifierId) external view returns (bool);

    /**
     * @dev Checks if an aggregator program ID is in the supported set
     * @param _zkCoProcessor Type of ZK coprocessor
     * @param _aggregatorId Aggregator program ID to check
     * @return True if the ID is supported
     */
    function isAggregatorIdSupported(ZkCoProcessorType _zkCoProcessor, bytes32 _aggregatorId)
        external
        view
        returns (bool);

    /**
     * @dev Gets the verifier address for a specific route
     * @param _zkCoProcessor Type of ZK coprocessor
     * @param _selector Proof selector
     * @return Verifier address (route-specific or default fallback)
     *
     * Note: Reverts if the route is frozen
     */
    function getZkVerifier(ZkCoProcessorType _zkCoProcessor, bytes4 _selector) external view returns (address);

    /**
     * @dev Returns the verifierProofId for a given verifierId
     * @param _zkCoProcessor Type of ZK coprocessor
     * @param _verifierId The verifier program ID
     * @return The corresponding verifierProofId
     */
    function getVerifierProofId(ZkCoProcessorType _zkCoProcessor, bytes32 _verifierId) external view returns (bytes32);

    /**
     * @dev Checks how many certificates in each report are trusted
     * @param _report_certs Array of certificate chains, each containing certificate hashes
     * @return Array indicating the number of trusted certificates in each chain
     *
     * For each certificate chain:
     * - Validates that the first certificate matches the root certificate
     * - Counts consecutive trusted certificates starting from the root
     * - Returns the count of trusted certificates for each chain
     */
    function checkTrustedIntermediateCerts(bytes32[][] calldata _report_certs) external view returns (uint8[] memory);

    // ============ Admin Functions ============

    /**
     * @dev Sets the trusted root certificate hash
     * @param _rootCert Hash of the new root certificate
     *
     * Requirements:
     * - Only callable by contract owner
     */
    function setRootCert(bytes32 _rootCert) external;

    /**
     * @dev Configures the zero-knowledge verification parameters for a specific coprocessor
     * @param _zkCoProcessor Type of ZK coprocessor (RiscZero or Succinct)
     * @param _config Configuration parameters including program IDs and verifier address
     * @param _verifierProofId The verifierProofId corresponding to the verifierId in config
     *
     * Requirements:
     * - Only callable by contract owner
     * - Must specify valid coprocessor type and configuration
     */
    function setZkConfiguration(
        ZkCoProcessorType _zkCoProcessor,
        ZkCoProcessorConfig memory _config,
        bytes32 _verifierProofId
    ) external;

    /**
     * @dev Revokes a trusted intermediate certificate
     * @param _certHash Hash of the certificate to revoke
     *
     * Requirements:
     * - Only callable by contract owner
     * - Certificate must exist in the trusted set
     */
    function revokeCert(bytes32 _certHash) external;

    /**
     * @dev Updates the verifier program ID, adding the new version to the supported set
     * @param _zkCoProcessor Type of ZK coprocessor
     * @param _newVerifierId New verifier program ID to set as latest
     * @param _newVerifierProofId New verifier proof ID (used in batch verification)
     *
     * Requirements:
     * - Only callable by contract owner
     * - New ID must be different from current latest
     */
    function updateVerifierId(ZkCoProcessorType _zkCoProcessor, bytes32 _newVerifierId, bytes32 _newVerifierProofId)
        external;

    /**
     * @dev Updates the aggregator program ID, adding the new version to the supported set
     * @param _zkCoProcessor Type of ZK coprocessor
     * @param _newAggregatorId New aggregator program ID to set as latest
     *
     * Requirements:
     * - Only callable by contract owner
     * - New ID must be different from current latest
     */
    function updateAggregatorId(ZkCoProcessorType _zkCoProcessor, bytes32 _newAggregatorId) external;

    /**
     * @dev Removes a verifier program ID from the supported set
     * @param _zkCoProcessor Type of ZK coprocessor
     * @param _verifierId Verifier program ID to remove
     *
     * Requirements:
     * - Only callable by contract owner
     * - Cannot remove the currently active (latest) verifier ID
     * - ID must exist in the supported set
     */
    function removeVerifierId(ZkCoProcessorType _zkCoProcessor, bytes32 _verifierId) external;

    /**
     * @dev Removes an aggregator program ID from the supported set
     * @param _zkCoProcessor Type of ZK coprocessor
     * @param _aggregatorId Aggregator program ID to remove
     *
     * Requirements:
     * - Only callable by contract owner
     * - Cannot remove the currently active (latest) aggregator ID
     * - ID must exist in the supported set
     */
    function removeAggregatorId(ZkCoProcessorType _zkCoProcessor, bytes32 _aggregatorId) external;

    /**
     * @dev Adds a route-specific verifier override
     * @param _zkCoProcessor Type of ZK coprocessor
     * @param _selector Proof selector (first 4 bytes of proof data)
     * @param _verifier Address of the verifier contract for this route
     *
     * Requirements:
     * - Only callable by contract owner
     * - Route must not be frozen
     * - Verifier address must not be zero
     */
    function addVerifyRoute(ZkCoProcessorType _zkCoProcessor, bytes4 _selector, address _verifier) external;

    /**
     * @dev Permanently freezes a verification route
     * @param _zkCoProcessor Type of ZK coprocessor
     * @param _selector Proof selector to freeze
     *
     * Requirements:
     * - Only callable by contract owner
     * - Route must not already be frozen
     *
     * WARNING: This action is IRREVERSIBLE
     */
    function freezeVerifyRoute(ZkCoProcessorType _zkCoProcessor, bytes4 _selector) external;

    // ============ Verification Functions ============

    /**
     * @dev Verifies a single attestation report using zero-knowledge proof
     * @param output Encoded VerifierJournal containing the verification result
     * @param zkCoprocessor Type of ZK coprocessor used to generate the proof
     * @param proofBytes Zero-knowledge proof data for the attestation
     * @return VerifierJournal containing the verification result and extracted data
     *
     * This function:
     * 1. Verifies the ZK proof using the specified coprocessor
     * 2. Decodes the verification result
     * 3. Validates the certificate chain against trusted certificates
     * 4. Checks timestamp validity within the allowed time difference
     * 5. Caches newly discovered trusted certificates
     * 6. Returns the complete verification result
     */
    function verify(bytes calldata output, ZkCoProcessorType zkCoprocessor, bytes calldata proofBytes)
        external
        returns (VerifierJournal memory);

    /**
     * @dev Verifies multiple attestation reports in a single batch operation
     * @param output Encoded BatchVerifierJournal containing aggregated verification results
     * @param zkCoprocessor Type of ZK coprocessor used to generate the proof
     * @param proofBytes Zero-knowledge proof data for batch verification
     * @return Array of VerifierJournal results, one for each attestation in the batch
     *
     * This function:
     * 1. Verifies the ZK proof using the specified coprocessor
     * 2. Decodes the batch verification results
     * 3. Validates each attestation's certificate chain and timestamp
     * 4. Caches newly discovered trusted certificates
     * 5. Returns the verification results for all attestations
     */
    function batchVerify(bytes calldata output, ZkCoProcessorType zkCoprocessor, bytes calldata proofBytes)
        external
        returns (VerifierJournal[] memory);

    // ============ Explicit Program ID Verification Functions ============

    /**
     * @dev Verifies a single attestation with an explicit program ID
     * @param output Encoded VerifierJournal containing the verification result
     * @param zkCoprocessor Type of ZK coprocessor used to generate the proof
     * @param programId Explicit verifier program ID to use (must be in supported set)
     * @param proofBytes Zero-knowledge proof data for the attestation
     * @return VerifierJournal containing the verification result and extracted data
     *
     * This allows verification with older program versions that are still supported.
     */
    function verifyWithProgramId(
        bytes calldata output,
        ZkCoProcessorType zkCoprocessor,
        bytes32 programId,
        bytes calldata proofBytes
    ) external returns (VerifierJournal memory);

    /**
     * @dev Verifies multiple attestations with explicit program IDs
     * @param output Encoded BatchVerifierJournal containing aggregated verification results
     * @param zkCoprocessor Type of ZK coprocessor used to generate the proof
     * @param aggregatorId Explicit aggregator program ID to use
     * @param verifierId The verifier program ID (contract looks up corresponding verifierProofId)
     * @param proofBytes Zero-knowledge proof data for batch verification
     * @return Array of VerifierJournal results, one for each attestation in the batch
     *
     * This allows batch verification with older program versions that are still supported.
     * The contract internally looks up the verifierProofId from the verifierId mapping.
     */
    function batchVerifyWithProgramId(
        bytes calldata output,
        ZkCoProcessorType zkCoprocessor,
        bytes32 aggregatorId,
        bytes32 verifierId,
        bytes calldata proofBytes
    ) external returns (VerifierJournal[] memory);
}
