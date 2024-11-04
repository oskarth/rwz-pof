use serde::{Deserialize, Serialize};

// Constants
pub const SEED: u64 = 31337;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealInfo {
    pub amount: u64,
    pub deal_id: String,
    pub buyer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedMessage {
    pub pubkey: Vec<u8>,
    pub message: DealInfo,
    pub signature: Vec<u8>,
}

// Add result type for error handling
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Proof generation failed: {0}")]
    ProofError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::Error),

    #[error("RISC0 error: {0}")]
    Risc0Error(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
