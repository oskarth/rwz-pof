use risc0_zkvm::Receipt;
use rwz_pof_core::SignedMessage;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Storage {
    commitments: HashMap<String, Vec<SignedMessage>>,
    proofs: HashMap<String, Receipt>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            commitments: HashMap::new(),
            proofs: HashMap::new(),
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
}
