use k256::ecdsa::signature::Verifier;
use k256::ecdsa::{Signature, SigningKey, VerifyingKey};
use k256::{elliptic_curve::generic_array::GenericArray, SecretKey};
use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};

const SEED: u64 = 31337;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct DealInfo {
    amount: u64,
    deal_id: String,
    buyer: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SignedMessage {
    pubkey: Vec<u8>,
    message: DealInfo,
    signature: Vec<u8>,
}

fn generate_deterministic_pubkey(offset: u64) -> Vec<u8> {
    let seed_bytes = (SEED.wrapping_add(offset)).to_le_bytes();
    let mut key_bytes = [0u8; 32];
    key_bytes[..8].copy_from_slice(&seed_bytes);

    let generic_bytes = GenericArray::from_slice(&key_bytes);
    let secret_key = SecretKey::from_bytes(generic_bytes).expect("Invalid key bytes");
    VerifyingKey::from(SigningKey::from(secret_key))
        .to_sec1_bytes()
        .to_vec()
}

fn get_valid_pubkeys() -> [Vec<u8>; 2] {
    [
        generate_deterministic_pubkey(0), // LB1
        generate_deterministic_pubkey(1), // LB2
    ]
}

fn verify_signature(signed: &SignedMessage) -> bool {
    // Get valid pubkeys and check if the signature's pubkey is in our set
    let valid_pubkeys = get_valid_pubkeys();
    if !valid_pubkeys.contains(&signed.pubkey) {
        return false;
    }

    let verifying_key =
        VerifyingKey::from_sec1_bytes(&signed.pubkey).expect("Invalid public key format");

    let signature = Signature::from_slice(&signed.signature).expect("Invalid signature format");

    let message_bytes = bincode::serialize(&signed.message).expect("Failed to serialize message");

    verifying_key.verify(&message_bytes, &signature).is_ok()
}

fn main() {
    let start = env::cycle_count();

    // Read private inputs
    let lb1_signed: SignedMessage = env::read();
    let lb2_signed: SignedMessage = env::read();
    let proof_amount: u64 = env::read();

    // Verify both signatures
    assert!(
        verify_signature(&lb1_signed),
        "LB1 signature verification failed"
    );
    assert!(
        verify_signature(&lb2_signed),
        "LB2 signature verification failed"
    );

    // Verify public keys are unique so that signatures cannot be repeated
    assert!(lb1_signed.pubkey != lb2_signed.pubkey);

    // Verify both messages refer to the same deal
    assert_eq!(
        lb1_signed.message.deal_id, lb2_signed.message.deal_id,
        "Deal IDs don't match"
    );
    assert_eq!(
        lb1_signed.message.buyer, lb2_signed.message.buyer,
        "Buyers don't match"
    );

    // Verify total amount meets proof requirement
    let total_amount = lb1_signed.message.amount + lb2_signed.message.amount;
    assert!(
        total_amount >= proof_amount,
        "Total amount {} less than required amount {}",
        total_amount,
        proof_amount
    );

    // Create minimal verification info
    let verification_info = DealInfo {
        amount: proof_amount, // Only show required amount (60), not total (80)
        deal_id: lb1_signed.message.deal_id.clone(),
        buyer: lb1_signed.message.buyer.clone(),
    };

    env::commit(&(verification_info, proof_amount));

    let end = env::cycle_count();
    eprintln!("total_cycle_count: {}", end - start);
}

#[cfg(test)]
mod tests {
    use super::*;
    use k256::ecdsa::SigningKey;
    use rand_core::OsRng;

    fn get_test_signing_key(offset: u64) -> SigningKey {
        let seed_bytes = (SEED.wrapping_add(offset)).to_le_bytes();
        let mut key_bytes = [0u8; 32];
        key_bytes[..8].copy_from_slice(&seed_bytes);

        let generic_bytes = GenericArray::from_slice(&key_bytes);
        let secret_key = SecretKey::from_bytes(generic_bytes).expect("Invalid key bytes");
        SigningKey::from(secret_key)
    }

    fn create_test_signed_message(
        signing_key: &SigningKey,
        amount: u64,
        deal_id: String,
        buyer: String,
    ) -> SignedMessage {
        let deal_info = DealInfo {
            amount,
            deal_id,
            buyer,
        };

        let message_bytes = bincode::serialize(&deal_info).unwrap();
        let signature = signing_key.sign(&message_bytes);
        let verifying_key = VerifyingKey::from(signing_key);

        SignedMessage {
            pubkey: verifying_key.to_sec1_bytes().to_vec(),
            message: deal_info,
            signature: signature.to_bytes().to_vec(),
        }
    }

    #[test]
    fn test_verify_signature() {
        let signing_key = SigningKey::random(&mut OsRng);

        let signed_msg = create_test_signed_message(
            &signing_key,
            500,
            "DEAL001".to_string(),
            "buyer1".to_string(),
        );

        assert!(verify_signature(&signed_msg));
    }

    #[test]
    fn test_invalid_signature() {
        let signing_key = SigningKey::random(&mut OsRng);
        let wrong_key = SigningKey::random(&mut OsRng);

        let mut signed_msg = create_test_signed_message(
            &signing_key,
            500,
            "DEAL001".to_string(),
            "buyer1".to_string(),
        );

        // Replace pubkey with wrong one to make signature invalid
        signed_msg.pubkey = VerifyingKey::from(&wrong_key).to_sec1_bytes().to_vec();

        assert!(!verify_signature(&signed_msg));
    }

    #[test]
    fn test_verify_invalid_signature() {
        // Generate a key with different seed/offset
        let invalid_key = get_test_signing_key(999);

        let invalid_msg = create_test_signed_message(
            &invalid_key,
            500,
            "DEAL001".to_string(),
            "buyer1".to_string(),
        );

        assert!(!verify_signature(&invalid_msg));
    }
}
