# RWZ-POF (Real World Zeroes - Proof of Financing)

A RISC0 ZKVM-based system that provides zero-knowledge verification of lending bank commitments.

## Context

In M&A deals, buyers need to prove to the Seller/Seller's Board (S/SB) that they have acquired the necessary financing to complete a deal. Traditionally, this involves naming the lending banks that have committed to providing the financing. However, revealing the identities of committed banks can disadvantage the buyer - if S/SB prefers another buyer who is struggling to secure financing, they could share these bank names with their preferred buyer.

## Goal

Provide assurance to the S/SB that lending banks within a known network of reputable banks have provided their lending commitments for a deal proposed by a specific Buyer, without revealing the identities of these lending banks.

## Roles & Operations

- **Buyer/Buyer's Bank (BB)**: Runs the host program with access to their lending bank commitments, generates proofs
- **Seller/Seller's Board (S/SB)**: Verifies the generated proofs using the guest program
- **Lending Banks (LB)**: Provide signed commitments to the buyer

## Implementation

### Core Components

- **Host Program** (host/src/main.rs)
  - Generates lending bank signatures for commitments
  - Creates zero-knowledge proofs
- **Guest Program** (methods/guest/src/main.rs)
  - Verifies signatures are from authorized banks
  - Validates total committed amount meets requirements
  - Commits verified deal info to journal

## Development

See [RISC0 Getting Started Guide](https://dev.risczero.com/api/getting-started).

### Testing
```bash
cargo test --release
```

### Running
```bash
# Local proving (slower)
cargo run --release

# Dev mode for rapid prototyping
RISC0_DEV_MODE=true cargo run --release
```

### Expected output

```
Starting proof generation...
total_cycle_count: 22955819

Proof generated successfully!
Verified deal info: DealInfo { amount: 50, deal_id: "DEAL123", buyer: "buyer123" }
Verified amount: 60
Receipt verification successful!
```

## Performance notes

On a MacBook Pro 2021 (M1 Max, 64GB) `cargo run --release` takes ~2m (incl compile). This is without any form of customization or performance work.

Currently total cycle count is ~23M. A secp256k1 signature verification is ~500k cycles (? source needed). Most expensive part is most likely serialization/deserialization.

## Sequence diagram

```mermaid
sequenceDiagram
    participant BB as Buyer/Bank
    participant LB1 as LendingBank1
    participant LB2 as LendingBank2
    participant ZK as ZKProof
    participant SB as Seller/Board

    BB->>LB1: Request signature for amount X
    LB1-->>BB: Sign(amount=X, deal_id, buyer)
    BB->>LB2: Request signature for amount Y
    LB2-->>BB: Sign(amount=Y, deal_id, buyer)

    Note over BB,ZK: BB runs host program
    BB->>ZK: Prove(LB1.sig, LB2.sig, total≥Z)
    ZK-->>BB: proof

    BB->>SB: Submit proof
    Note over SB: Verify:<br/>1. Signatures valid<br/>2. LBs authorized<br/>3. Total ≥ required
    SB-->>BB: Verified/Rejected
```

## TODO
- Integrate with glue code (e.g. UI)
- Replace deterministic test keys with separate generation
- Allow for more than two LBs
- Implement comprehensive testing, error handling, and input validation
- Experiment with remote proving capabilities
- Improve performance