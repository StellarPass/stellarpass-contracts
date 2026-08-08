# StellarPass Contracts

The on-chain membership and access-control layer for StellarPass.

This repository contains the Soroban Rust contract that owns the authoritative state for StellarPass communities. The API, SDK, frontend, and third-party integrations should treat this contract and its events as the source of truth for access.

## What This Repo Owns

- Community identity: ID, admin wallet, name, and creation timestamp.
- One role record per wallet per community.
- Role grant, update, revocation, and optional expiry behavior.
- Direct `has_access` and `get_role` reads.
- Stable events consumed by `stellarpass-api`.
- Contract versioning and the controlled upgrade entry point.

It does **not** own community descriptions, images, Discord configuration, or other display metadata. Those belong to the API read model.

## Product Integration

```text
stellarpass-app / external apps
          |
          | wallet-signed transactions and reads
          v
StellarPass Soroban contract
          |
          | contract events
          v
stellarpass-api indexer
```

`stellarpass-sdk` wraps the contract interface for TypeScript consumers. `stellarpass-app` currently uses mock actions for its reviewer build, but its wallet integration is designed to call these contract methods once a real deployment is configured.

## Contract Methods

| Method | Authorization | Description |
|---|---|---|
| `__constructor(upgrade_admin)` | Deployment | One-time contract initialization |
| `create_community(admin, name)` | `admin` wallet | Creates a community and returns its ID |
| `grant_role(admin, community_id, member, role, expires_at)` | Community admin | Grants or updates one member role |
| `revoke_role(admin, community_id, member)` | Community admin | Revokes current access while preserving the record |
| `has_access(member, community_id)` | Public | Returns whether the wallet currently has access |
| `get_role(member, community_id)` | Public | Returns the current role record |
| `get_community(community_id)` | Public | Returns community details |
| `upgrade(upgrade_admin, wasm_hash)` | Upgrade admin | Updates contract WASM |
| `version()` | Public | Returns the contract version |

## Contract Rules

- The wallet creating a community is its admin in v1.
- The protocol upgrade admin can upgrade code only; it cannot administer communities.
- A wallet has exactly one role per community.
- An identical grant is an idempotent no-op.
- A changed role or expiry emits `role_updated`.
- A revoked role can be granted again.
- `expires_at <= ledger_timestamp` means expired.
- `None` means no expiry.

## Events

| Event | Topics | Data |
|---|---|---|
| `community_created` | `community_id`, `admin` | `name` |
| `role_granted` | `community_id`, `member` | `role`, `expires_at` |
| `role_updated` | `community_id`, `member` | `old_role`, `role`, `expires_at` |
| `role_revoked` | `community_id`, `member` | — |
| `contract_upgraded` | — | `wasm_hash` |

The API indexer depends on these event names and payloads. Event changes require a coordinated contract, API, SDK, and documentation update.

## Development

Prerequisites: Rust, the `wasm32v1-none` target, and the Stellar CLI for deployment.

```bash
rustup target add wasm32v1-none
cargo test
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo build --release --target wasm32v1-none
```

The optimized WASM artifact is consumed by `stellarpass-devops` deployment scripts. Deployment metadata belongs in `deployments/<network>.json`.

## Related Repositories

- [`stellarpass-api`](https://github.com/StellarPass/stellarpass-api): indexes these events and serves derived reads.
- [`stellarpass-sdk`](https://github.com/StellarPass/stellarpass-sdk): exposes typed contract methods to TypeScript.
- [`stellarpass-app`](https://github.com/StellarPass/stellarpass-app): provides the admin/member experience.
- [`stellarpass-docs`](https://github.com/StellarPass/stellarpass-docs): canonical protocol and event documentation.

## License

MIT
