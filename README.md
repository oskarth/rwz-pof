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

## TODO
- Integrate with glue code (e.g. UI)
- Replace deterministic test keys with separate generation
- Allow for more than two LBs
- Implement comprehensive testing, error handling, and input validation
- Experiment with remote proving capabilities