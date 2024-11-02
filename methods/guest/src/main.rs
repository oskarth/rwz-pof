use risc0_zkvm::guest::env;

use k256::ecdsa::signature::Verifier;
use k256::ecdsa::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};

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

// TODO: Add this logic, requires persisting keys
// // List of valid lending banks public keys
// const VALID_PUBKEYS: [&[u8]; 2] = [
//     // Add your valid pubkeys here in SEC1 format
//     &[/* pubkey1 bytes */],
//     &[/* pubkey2 bytes */],
// ];

fn verify_signature(signed: &SignedMessage) -> bool {
    // // First verify the pubkey is in our valid set
    // if !VALID_PUBKEYS.contains(&&*signed.pubkey) {
    //     return false;
    // }

    let verifying_key =
        VerifyingKey::from_sec1_bytes(&signed.pubkey).expect("Invalid public key format");

    let signature = Signature::from_slice(&signed.signature).expect("Invalid signature format");

    let message_bytes = bincode::serialize(&signed.message).expect("Failed to serialize message");

    verifying_key.verify(&message_bytes, &signature).is_ok()
}

fn main() {
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

    // Commit the verified deal info and proved amount
    // We'll use LB1's deal info since we verified they match
    env::commit(&(lb1_signed.message, proof_amount));
}

#[cfg(test)]
mod tests {
    use super::*;
    use k256::ecdsa::SigningKey;
    use rand_core::OsRng;

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
}
