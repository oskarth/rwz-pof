use crate::storage::{ProofJobStatus, Storage};
use rwz_pof_core::generate_proof;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ProofWorker {}

impl ProofWorker {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn process_job(&self, job_id: String, storage: Arc<Mutex<Storage>>) {
        // Update job status to InProgress
        {
            let mut storage = storage.lock().unwrap();
            storage.update_proof_job(&job_id, |job| {
                job.status = ProofJobStatus::InProgress;
            });
        }

        // Get job details and commitments
        let (deal_id, required_amount, commitments) = {
            let storage = storage.lock().unwrap();
            let job = storage.get_proof_job(&job_id).unwrap();
            let commitments = storage
                .get_commitments(&job.deal_id)
                .map(|c| c.clone())
                .unwrap_or_default();
            (job.deal_id.clone(), job.required_amount, commitments)
        };

        if commitments.len() < 2 {
            let mut storage = storage.lock().unwrap();
            storage.update_proof_job(&job_id, |job| {
                job.status = ProofJobStatus::Failed;
                job.error = Some("Not enough commitments".to_string());
            });
            return;
        }

        // Generate proof
        let result = generate_proof(
            commitments[0].clone(),
            commitments[1].clone(),
            required_amount,
        );

        // Process the result outside the closure first
        match result {
            Ok((receipt, deal_info, verified_amount)) => {
                let mut storage = storage.lock().unwrap();
                // First store the receipt
                storage.add_proof(deal_id.clone(), receipt);
                // Then update the job
                storage.update_proof_job(&job_id, |job| {
                    job.status = ProofJobStatus::Completed;
                    job.proof = Some((deal_info.clone(), verified_amount));
                });
            }
            Err(e) => {
                let mut storage = storage.lock().unwrap();
                storage.update_proof_job(&job_id, |job| {
                    job.status = ProofJobStatus::Failed;
                    job.error = Some(e.to_string());
                });
            }
        }
    }
}
