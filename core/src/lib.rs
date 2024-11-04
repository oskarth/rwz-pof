pub mod engine;
pub mod types;

pub use engine::{create_signed_message, generate_proof, get_deterministic_signing_key};
pub use types::{DealInfo, SignedMessage};

// Re-export essential RISC0 components that consumers might need
pub use methods::{RWZ_POF_GUEST_ELF, RWZ_POF_GUEST_ID};
