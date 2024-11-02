# RWZ-POF UI Integration Plan

*NOTE: This plan was generated quickly with the help of AI tools, likely contains errors.*

## Overview
Plan for integrating RISC0-based proof generation system with a minimal web UI for demonstration/testing.

## Core Engine Integration

### Existing Components
- **Host Program**: Handles key generation, signing, and proof generation
  ```rust
  // Key functions available:
  create_signed_message(signing_key: &SigningKey, amount: u64, deal_id: String, buyer: String) -> SignedMessage
  get_deterministic_signing_key(offset: u64) -> SigningKey
  // Proof generation uses:
  risc0_zkvm::{default_prover, ExecutorEnv}
  ```

- **Guest Program**: Verifies signatures and amounts
  ```rust
  // Core verification functions:
  verify_signature(signed: &SignedMessage) -> bool
  get_valid_pubkeys() -> [Vec<u8>; 2]
  // Journal output: (DealInfo, verified_amount)
  ```

### Test Data Generation
For MVP, we'll reuse test data generation:
```rust
- Use deterministic keys (SEED = 31337)
- Default test deal: "DEAL123" / "buyer123"
- Support for two lending banks (LB1, LB2)
```

## Implementation Plan

### 1. Backend Server
```
cargo new rwz-pof-server
```

#### Endpoints
```
POST /lb/commitment
- Input: { bank_index: number, amount: number }
- Output: { signed_message: SignedMessage }

POST /bb/proof
- Input: { required_amount: number, commitments: SignedMessage[] }
- Output: { receipt: Receipt, verified_amount: number }

POST /sb/verify
- Input: { receipt: Receipt }
- Output: { verified: boolean, deal_info: DealInfo }
```

#### Integration with Core
```rust
// Example integration
use methods::{RWZ_POF_GUEST_ELF, RWZ_POF_GUEST_ID};

async fn handle_commitment(bank_index: u64, amount: u64) {
    let key = get_deterministic_signing_key(bank_index);
    create_signed_message(&key, amount, "DEAL123", "buyer123")
}

async fn generate_proof(signed_msgs: Vec<SignedMessage>, required: u64) -> Receipt {
    let env = ExecutorEnv::builder()
        .write(&signed_msgs[0])?
        .write(&signed_msgs[1])?
        .write(&required)?
        .build()?;
    
    default_prover().prove(env, RWZ_POF_GUEST_ELF)?.receipt
}
```

### 2. In-Memory Storage
```rust
struct Storage {
    commitments: HashMap<String, Vec<SignedMessage>>,
    proofs: HashMap<String, Receipt>,
}
```

### 3. Frontend UI
```
npx create-react-app rwz-pof-ui
```

#### Components
```
src/
  components/
    LBCommitment.tsx      # Lending bank interface
    BBDashboard.tsx       # Buyer's bank dashboard
    SBVerification.tsx    # Seller verification
    Navigation.tsx        # Role switching
```

#### State Management
```typescript
interface CommitmentState {
  lb1Amount?: number;
  lb2Amount?: number;
  proof?: string;
  verificationStatus: 'pending' | 'verified' | 'failed';
}
```

### 4. Development Steps

1. **Backend Setup**
   - Set up Rust HTTP server
   - Create basic endpoints
   - Wire up core engine functions
   - Add in-memory storage

2. **Frontend Dev**
   - Basic UI components
   - API integration
   - Status display
   - Basic error handling

3. **Integration**
   - Connect frontend to backend
   - Test full flow:
     1. LB1 commits amount
     2. LB2 commits amount
     3. BB generates proof
     4. SB verifies

### 5. Testing Flow
```
1. Start backend:
   cargo run

2. Start frontend:
   cd ui && npm start

3. Test sequence:
   - Open http://localhost:3000
   - Switch to LB1, enter amount
   - Switch to LB2, enter amount
   - Switch to BB, generate proof
   - Switch to SB, verify
```

## MVP Limitations/Shortcuts
- No persistence
- Uses deterministic test keys
- Single hardcoded deal
- Basic error handling
- No authentication
- No concurrent deals

## Future Improvements
- Proper key management
- Multiple deal support
- Persistent storage
- Better error handling
- Real authentication
- Status webhooks

## Questions/Decisions
- API format for proofs/signatures
- Error handling strategy
- Storage format
- Connection configuration