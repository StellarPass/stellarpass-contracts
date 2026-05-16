# CredChain Soroban Contract

Soroban contract for issuing non-transferable (soulbound) academic micro-credentials on Stellar testnet.

## MVP Scope
- Authorized institution issues a credential to a student address.
- Credential stores immutable metadata:
  - `institution`
  - `course`
  - `issued_at` (unix seconds)
  - `credential_hash` (document hash)
- Public read methods for verification.
- No transfer function (soulbound behavior).

## Stack
- Rust
- `soroban-sdk`
- Stellar testnet

## Quickstart
1. Install Stellar CLI and Rust toolchain for `wasm32-unknown-unknown`.
2. Build and test:
   - `cargo test`
   - `cargo build --target wasm32-unknown-unknown --release`
3. Deploy to testnet:
   - `stellar contract deploy ...`

## Repo Structure
- `src/lib.rs`: contract implementation
- `src/test.rs`: unit tests
- `Cargo.toml`: build configuration
- `docs/`: architecture and security notes

## Open Contribution Areas
- Batch issuance API
- Revocation model and status transitions
- Event indexing conventions for verifier clients
- Property/fuzz tests

