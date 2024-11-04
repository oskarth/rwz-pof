use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use warp::{reply::json, Reply};
use rwz_pof_core::{
    create_signed_message, generate_proof, get_deterministic_signing_key,
    DealInfo, SignedMessage,
};
use crate::storage::Storage;

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
    println!("Handling commitment request for bank {}", req.bank_index);
    
    // Get deterministic key for the bank
    let signing_key = get_deterministic_signing_key(req.bank_index);
    
    // Create signed message
    match create_signed_message(
        &signing_key,
        req.amount,
        req.deal_id.clone(),
        req.buyer,
    ) {
        Ok(signed_message) => {
            // Store the commitment
            storage.add_commitment(req.deal_id, signed_message.clone());
            
            Ok(json(&CommitmentResponse { signed_message }))
        }
        Err(e) => {
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
    
    // Get commitments for the deal
    match storage.get_commitments(&req.deal_id) {
        Some(commitments) if commitments.len() >= 2 => {
            // Take the first two commitments
            let lb1_signed = commitments[0].clone();
            let lb2_signed = commitments[1].clone();
            
            // Generate proof
            match generate_proof(lb1_signed, lb2_signed, req.required_amount) {
                Ok((receipt, deal_info, verified_amount)) => {
                    // Store the proof
                    storage.add_proof(req.deal_id, receipt);
                    
                    Ok(json(&ProofResponse {
                        success: true,
                        verified_amount,
                        deal_info,
                    }))
                }
                Err(e) => {
                    Ok(json(&ErrorResponse {
                        error: format!("Failed to generate proof: {}", e),
                    }))
                }
            }
        }
        _ => {
            Ok(json(&ErrorResponse {
                error: "Not enough commitments for proof generation".to_string(),
            }))
        }
    }
}

pub async fn handle_verify(
    req: VerifyRequest,
    mut storage: Storage,
) -> Result<impl Reply, Infallible> {
    println!("Handling verify request for deal {}", req.deal_id);
    
    // Get the proof for verification
    match storage.get_proof(&req.deal_id) {
        Some(receipt) => {
            // Verify the receipt and decode journal
            match receipt.verify(rwz_pof_core::RWZ_POF_GUEST_ID) {
                Ok(()) => {
                    // Decode the journal to get deal info
                    match receipt.journal.decode::<(DealInfo, u64)>() {
                        Ok((deal_info, _)) => {
                            Ok(json(&VerifyResponse {
                                verified: true,
                                deal_info: Some(deal_info),
                            }))
                        }
                        Err(e) => {
                            Ok(json(&ErrorResponse {
                                error: format!("Failed to decode proof data: {}", e),
                            }))
                        }
                    }
                }
                Err(e) => {
                    Ok(json(&ErrorResponse {
                        error: format!("Proof verification failed: {}", e),
                    }))
                }
            }
        }
        None => {
            Ok(json(&ErrorResponse {
                error: "No proof found for the deal".to_string(),
            }))
        }
    }
}