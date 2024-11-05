use risc0_zkvm::Receipt;
use rwz_pof_core::{DealInfo, SignedMessage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::serde::iso8601;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProofJobStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofGenerationJob {
    pub id: String,
    pub status: ProofJobStatus,
    pub deal_id: String,
    pub required_amount: u64,
    #[serde(with = "iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(with = "iso8601")]
    pub updated_at: OffsetDateTime,
    // Only store deal_info and verified_amount in the job
    //     pub proof: Option<(Receipt, DealInfo, u64)>,
    pub proof: Option<(DealInfo, u64)>,
    pub error: Option<String>,
}

impl ProofGenerationJob {
    pub fn new(deal_id: String, required_amount: u64) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            status: ProofJobStatus::Pending,
            deal_id,
            required_amount,
            created_at: now,
            updated_at: now,
            proof: None,
            error: None,
        }
    }
}

#[derive(Clone)]
pub struct Storage {
    commitments: HashMap<String, Vec<SignedMessage>>,
    proofs: HashMap<String, Receipt>,
    proof_jobs: HashMap<String, ProofGenerationJob>, // Indexed by job_id
}

impl Storage {
    pub fn new() -> Self {
        Self {
            commitments: HashMap::new(),
            proofs: HashMap::new(),
            proof_jobs: HashMap::new(),
        }
    }

    pub fn add_commitment(&mut self, deal_id: String, commitment: SignedMessage) {
        self.commitments
            .entry(deal_id)
            .or_insert_with(Vec::new)
            .push(commitment);
    }

    pub fn get_commitments(&self, deal_id: &str) -> Option<&Vec<SignedMessage>> {
        self.commitments.get(deal_id)
    }

    pub fn add_proof(&mut self, deal_id: String, receipt: Receipt) {
        self.proofs.insert(deal_id, receipt);
    }

    pub fn get_proof(&self, deal_id: &str) -> Option<&Receipt> {
        self.proofs.get(deal_id)
    }
    pub fn create_proof_job(
        &mut self,
        deal_id: String,
        required_amount: u64,
    ) -> ProofGenerationJob {
        let job = ProofGenerationJob::new(deal_id, required_amount);
        self.proof_jobs.insert(job.id.clone(), job.clone());
        job
    }

    pub fn get_proof_job(&self, job_id: &str) -> Option<&ProofGenerationJob> {
        self.proof_jobs.get(job_id)
    }

    // Modified to return both job info and receipt if available
    pub fn get_proof_job_with_receipt(
        &self,
        job_id: &str,
    ) -> Option<(ProofGenerationJob, Option<&Receipt>)> {
        self.proof_jobs.get(job_id).map(|job| {
            let receipt = if job.status == ProofJobStatus::Completed {
                self.get_proof(&job.deal_id)
            } else {
                None
            };
            (job.clone(), receipt)
        })
    }

    pub fn update_proof_job(
        &mut self,
        job_id: &str,
        mut updater: impl FnMut(&mut ProofGenerationJob),
    ) -> Option<ProofGenerationJob> {
        if let Some(job) = self.proof_jobs.get_mut(job_id) {
            job.updated_at = OffsetDateTime::now_utc();
            updater(job);
            Some(job.clone())
        } else {
            None
        }
    }
}
