use k256::ecdsa::{signature::Signer, SigningKey, VerifyingKey};
use k256::{elliptic_curve::generic_array::GenericArray, SecretKey};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

use crate::types::{CoreError, DealInfo, Result, SignedMessage, SEED};

pub fn get_deterministic_signing_key(offset: u64) -> SigningKey {
    let seed_bytes = (SEED.wrapping_add(offset)).to_le_bytes();
    let mut key_bytes = [0u8; 32];
    key_bytes[..8].copy_from_slice(&seed_bytes);

    let generic_bytes = GenericArray::from_slice(&key_bytes);
    let secret_key = SecretKey::from_bytes(generic_bytes).expect("Invalid key bytes");
    SigningKey::from(secret_key)
}

pub fn create_signed_message(
    signing_key: &SigningKey,
    amount: u64,
    deal_id: String,
    buyer: String,
) -> Result<SignedMessage> {
    let deal_info = DealInfo {
        amount,
        deal_id,
        buyer,
    };

    let message_bytes = bincode::serialize(&deal_info)?;
    let signature: k256::ecdsa::Signature = signing_key.sign(&message_bytes);
    let verifying_key = VerifyingKey::from(signing_key);

    Ok(SignedMessage {
        pubkey: verifying_key.to_sec1_bytes().to_vec(),
        message: deal_info,
        signature: signature.to_bytes().to_vec(),
    })
}

pub fn generate_proof(
    lb1_signed: SignedMessage,
    lb2_signed: SignedMessage,
    proof_amount: u64,
) -> Result<(Receipt, DealInfo, u64)> {
    let env = ExecutorEnv::builder()
        .write(&lb1_signed)
        .map_err(|e| CoreError::Risc0Error(e.to_string()))?
        .write(&lb2_signed)
        .map_err(|e| CoreError::Risc0Error(e.to_string()))?
        .write(&proof_amount)
        .map_err(|e| CoreError::Risc0Error(e.to_string()))?
        .build()
        .map_err(|e| CoreError::Risc0Error(e.to_string()))?;

    let prover = default_prover();

    let prove_info = prover
        .prove(env, crate::RWZ_POF_GUEST_ELF)
        .map_err(|e| CoreError::ProofError(e.to_string()))?;

    let receipt = prove_info.receipt;

    // Extract journal data
    let (deal_info, verified_amount): (DealInfo, u64) = receipt
        .journal
        .decode()
        .map_err(|e| CoreError::Risc0Error(e.to_string()))?;

    Ok((receipt, deal_info, verified_amount))
}

#[cfg(test)]
mod tests {
    use super::*;
    use k256::ecdsa::signature::Verifier;

    #[test]
    fn test_create_and_verify_message() {
        let key = get_deterministic_signing_key(0);
        let msg = create_signed_message(&key, 100, "TEST1".into(), "buyer1".into()).unwrap();

        let verifying_key = VerifyingKey::from_sec1_bytes(&msg.pubkey).unwrap();
        let message_bytes = bincode::serialize(&msg.message).unwrap();
        let signature = k256::ecdsa::Signature::from_slice(&msg.signature).unwrap();

        assert!(verifying_key.verify(&message_bytes, &signature).is_ok());
    }

    #[test]
    fn test_proof_generation() {
        let lb1_key = get_deterministic_signing_key(0);
        let lb2_key = get_deterministic_signing_key(1);

        let lb1_signed =
            create_signed_message(&lb1_key, 50, "DEAL123".into(), "buyer123".into()).unwrap();
        let lb2_signed =
            create_signed_message(&lb2_key, 30, "DEAL123".into(), "buyer123".into()).unwrap();

        let (receipt, deal_info, verified_amount) =
            generate_proof(lb1_signed, lb2_signed, 60).unwrap();

        assert_eq!(deal_info.deal_id, "DEAL123");
        assert_eq!(verified_amount, 60);

        // Verify the receipt
        receipt.verify(crate::RWZ_POF_GUEST_ID).unwrap();
    }
}
