use crate::storage::Storage;
use rwz_pof_core::{
    create_signed_message, generate_proof, get_deterministic_signing_key, DealInfo, SignedMessage,
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use warp::{reply::json, Reply};

// Request/Response types
#[derive(Debug, Deserialize)]
pub struct CommitmentRequest {
    bank_index: u64,
    amount: u64,
    #[serde(default = "default_deal_id")]
    deal_id: String,
    #[serde(default = "default_buyer")]
    buyer: String,
}

fn default_deal_id() -> String {
    "DEAL123".to_string()
}

fn default_buyer() -> String {
    "buyer123".to_string()
}

#[derive(Debug, Deserialize)]
pub struct ProofRequest {
    required_amount: u64,
    deal_id: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    deal_id: String,
}

#[derive(Debug, Serialize)]
pub struct CommitmentResponse {
    signed_message: SignedMessage,
}

#[derive(Debug, Serialize)]
pub struct ProofResponse {
    success: bool,
    verified_amount: u64,
    deal_info: DealInfo,
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    verified: bool,
    deal_info: Option<DealInfo>,
}

// Error responses
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

// Handlers
pub async fn handle_commitment(
    req: CommitmentRequest,
    mut storage: Storage,
) -> Result<impl Reply, Infallible> {
    println!("Handling commitment request: {:?}", req);

    let signing_key = get_deterministic_signing_key(req.bank_index);

    match create_signed_message(&signing_key, req.amount, req.deal_id.clone(), req.buyer) {
        Ok(signed_message) => {
            storage.add_commitment(req.deal_id.clone(), signed_message.clone());

            // Debug: verify storage after adding
            if let Some(commitments) = storage.get_commitments(&req.deal_id) {
                println!(
                    "Stored commitments for deal {}: {}",
                    req.deal_id,
                    commitments.len()
                );
            }

            Ok(json(&CommitmentResponse { signed_message }))
        }
        Err(e) => {
            println!("Error creating commitment: {}", e);
            Ok(json(&ErrorResponse {
                error: format!("Failed to create commitment: {}", e),
            }))
        }
    }
}

pub async fn handle_proof(
    req: ProofRequest,
    mut storage: Storage,
) -> Result<impl Reply, Infallible> {
    println!("Handling proof request for deal {}", req.deal_id);

    let commitments = match storage.get_commitments(&req.deal_id) {
        Some(commits) => commits,
        None => {
            println!("No commitments found for deal {}", req.deal_id);
            return Ok(json(&ErrorResponse {
                error: format!("No commitments found for deal {}", req.deal_id),
            }));
        }
    };

    println!(
        "Found {} commitments for deal {}",
        commitments.len(),
        req.deal_id
    );

    if commitments.len() < 2 {
        println!("Not enough commitments (need 2, got {})", commitments.len());
        return Ok(json(&ErrorResponse {
            error: format!(
                "Not enough commitments for proof generation. Need 2, got {}",
                commitments.len()
            ),
        }));
    }

    let lb1_signed = commitments[0].clone();
    let lb2_signed = commitments[1].clone();

    println!(
        "Generating proof with amounts: {} and {}",
        lb1_signed.message.amount, lb2_signed.message.amount
    );

    match generate_proof(lb1_signed, lb2_signed, req.required_amount) {
        Ok((receipt, deal_info, verified_amount)) => {
            println!(
                "Proof generated successfully. Verified amount: {}",
                verified_amount
            );
            storage.add_proof(req.deal_id, receipt);

            Ok(json(&ProofResponse {
                success: true,
                verified_amount,
                deal_info,
            }))
        }
        Err(e) => {
            println!("Error generating proof: {}", e);
            Ok(json(&ErrorResponse {
                error: format!("Failed to generate proof: {}", e),
            }))
        }
    }
}

pub async fn handle_verify(
    req: VerifyRequest,
    mut storage: Storage,
) -> Result<impl Reply, Infallible> {
    println!("Handling verify request for deal {}", req.deal_id);

    match storage.get_proof(&req.deal_id) {
        Some(receipt) => {
            println!("Found proof for deal {}", req.deal_id);
            match receipt.verify(rwz_pof_core::RWZ_POF_GUEST_ID) {
                Ok(()) => {
                    println!("Proof verified successfully");
                    match receipt.journal.decode::<(DealInfo, u64)>() {
                        Ok((deal_info, verified_amount)) => {
                            println!("Decoded deal info - verified amount: {}", verified_amount);
                            Ok(json(&VerifyResponse {
                                verified: true,
                                deal_info: Some(deal_info),
                            }))
                        }
                        Err(e) => {
                            println!("Error decoding proof data: {}", e);
                            Ok(json(&ErrorResponse {
                                error: format!("Failed to decode proof data: {}", e),
                            }))
                        }
                    }
                }
                Err(e) => {
                    println!("Error verifying proof: {}", e);
                    Ok(json(&ErrorResponse {
                        error: format!("Proof verification failed: {}", e),
                    }))
                }
            }
        }
        None => {
            println!("No proof found for deal {}", req.deal_id);
            Ok(json(&ErrorResponse {
                error: format!("No proof found for deal {}", req.deal_id),
            }))
        }
    }
}
