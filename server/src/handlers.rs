use crate::storage::Storage;
use crate::worker::ProofWorker;
use rwz_pof_core::{
    create_signed_message, generate_proof, get_deterministic_signing_key, DealInfo, SignedMessage,
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use warp::{reply::json, Reply};

use crate::storage::ProofJobStatus;
use time::OffsetDateTime;

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

#[derive(Debug, Deserialize)]
pub struct CreateProofJobRequest {
    deal_id: String,
    required_amount: u64,
}

#[derive(Debug, Serialize)]
pub struct CreateProofJobResponse {
    job_id: String,
}

#[derive(Debug, Serialize)]
pub struct GetProofJobResponse {
    status: ProofJobStatus,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    proof: Option<ProofResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

// Error responses
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

// Handlers
pub async fn handle_commitment(
    req: CommitmentRequest,
    storage: Arc<Mutex<Storage>>,
) -> Result<impl Reply, Infallible> {
    println!("Handling commitment request: {:?}", req);

    let signing_key = get_deterministic_signing_key(req.bank_index);

    match create_signed_message(&signing_key, req.amount, req.deal_id.clone(), req.buyer) {
        Ok(signed_message) => {
            // Lock storage for modification
            let mut storage = storage.lock().unwrap();
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
    storage: Arc<Mutex<Storage>>,
) -> Result<impl Reply, Infallible> {
    println!("Handling proof request for deal {}", req.deal_id);

    // First lock storage for reading commitments
    let storage_guard = storage.lock().unwrap();

    let commitments = match storage_guard.get_commitments(&req.deal_id) {
        Some(commits) => commits.clone(), // Clone the commits while we have the lock
        None => {
            println!("No commitments found for deal {}", req.deal_id);
            return Ok(json(&ErrorResponse {
                error: format!("No commitments found for deal {}", req.deal_id),
            }));
        }
    };

    // Drop the lock so we don't hold it during proof generation
    drop(storage_guard);

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
            // Get a new mutable lock for storing the proof
            let mut storage = storage.lock().unwrap();
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
    storage: Arc<Mutex<Storage>>,
) -> Result<impl Reply, Infallible> {
    println!("Handling verify request for deal {}", req.deal_id);

    // Lock storage for reading proof
    let storage = storage.lock().unwrap();
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

pub async fn handle_create_proof_job(
    req: CreateProofJobRequest,
    storage: Arc<Mutex<Storage>>,
    proof_worker: Arc<ProofWorker>,
) -> Result<impl Reply, Infallible> {
    println!("Creating proof job for deal {}", req.deal_id);

    let job = {
        let mut storage = storage.lock().unwrap();
        storage.create_proof_job(req.deal_id.clone(), req.required_amount)
    };

    // Spawn background task to generate proof
    let storage_clone = Arc::clone(&storage);
    let job_id = job.id.clone();

    tokio::spawn(async move {
        proof_worker.process_job(job_id, storage_clone).await;
    });

    Ok(json(&CreateProofJobResponse { job_id: job.id }))
}

pub async fn handle_get_proof_job(
    job_id: String,
    storage: Arc<Mutex<Storage>>,
) -> Result<impl Reply, Infallible> {
    let storage = storage.lock().unwrap();

    match storage.get_proof_job_with_receipt(&job_id) {
        Some((job, _receipt)) => Ok(json(&GetProofJobResponse {
            status: job.status.clone(),
            created_at: job.created_at,
            updated_at: job.updated_at,
            proof: job.proof.map(|(deal_info, verified_amount)| ProofResponse {
                success: true,
                verified_amount,
                deal_info,
            }),
            error: job.error.clone(),
        })),
        None => Ok(json(&ErrorResponse {
            error: format!("Proof job not found: {}", job_id),
        })),
    }
}
