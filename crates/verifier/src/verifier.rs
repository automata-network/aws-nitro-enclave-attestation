use alloy_primitives::Bytes;
use serde_bytes::ByteBuf;

use crate::{
    stub::{Pcr, VerificationResult, VerifierInput, VerifierJournal},
    AttestationReport,
};

fn get_option_bytes(val: &Option<ByteBuf>) -> Bytes {
    val.as_ref().map(|n| n.to_vec()).unwrap_or_default().into()
}

pub fn verify_attestation_report(input: &VerifierInput) -> anyhow::Result<VerifierJournal> {
    let report = AttestationReport::parse(&input.attestationReport)?;

    let doc = report.doc();
    let cert_chain =
        report.authenticate(input.trustedCertsPrefixLen as usize, doc.timestamp / 1000)?;

    let user_data = get_option_bytes(&doc.user_data);
    let nonce = get_option_bytes(&doc.nonce);
    let public_key = get_option_bytes(&doc.public_key);
    let pcrs = doc
        .pcrs
        .iter()
        .map(|(index, value)| Pcr {
            index: *index,
            value: value.into(),
        })
        .filter(|pcr| !pcr.value.is_zero())
        .collect::<Vec<_>>();

    let output = VerifierJournal {
        result: VerificationResult::Success,
        certs: cert_chain.digest().to_vec(),
        trustedCertsPrefixLen: input.trustedCertsPrefixLen,
        userData: user_data.into(),
        nonce: nonce.into(),
        publicKey: public_key.into(),
        pcrs,
        moduleId: doc.module_id.clone(),
        timestamp: doc.timestamp,
    };

    Ok(output)
}
