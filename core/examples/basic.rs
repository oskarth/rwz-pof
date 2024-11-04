use rwz_pof_core::{create_signed_message, generate_proof, get_deterministic_signing_key};
use tracing_subscriber;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    // Generate keys for our two lending banks
    let lb1_key = get_deterministic_signing_key(0);
    let lb2_key = get_deterministic_signing_key(1);

    // Create signed messages for a deal requiring 60 total
    let lb1_signed = create_signed_message(
        &lb1_key,
        50, // LB1 provides 50
        "DEAL123".to_string(),
        "buyer123".to_string(),
    )
    .unwrap();

    let lb2_signed = create_signed_message(
        &lb2_key,
        30, // LB2 provides 30
        "DEAL123".to_string(),
        "buyer123".to_string(),
    )
    .unwrap();

    let proof_amount = 60u64; // We want to prove we have at least 60

    println!("Starting proof generation...");

    // Generate and verify the proof
    let (receipt, deal_info, verified_amount) =
        generate_proof(lb1_signed, lb2_signed, proof_amount).unwrap();

    println!("Proof generated successfully!");
    println!("Verified deal info: {:?}", deal_info);
    println!("Verified amount: {}", verified_amount);

    // Verify the receipt
    receipt.verify(rwz_pof_core::RWZ_POF_GUEST_ID).unwrap();
    println!("Receipt verification successful!");
}
