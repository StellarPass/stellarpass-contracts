# StellarPass Contracts

Soroban smart contracts for wallet-native community membership and access control on Stellar.

## Architecture

The `StellarPassContract` is the on-chain source of truth for communities and their membership records:

- **Communities** are created by wallet-authorized admins.
- **Roles** are granted, updated, and revoked by community admins.
- **Access** is verified directly against on-chain state.
- **Upgrades** are controlled by a dedicated upgrade-admin address with no community authority.

## Contract Methods

| Method | Auth | Description |
|---|---|---|
| `initialize(upgrade_admin)` | Deploy | One-time constructor |
| `create_community(admin, name)` | admin | Create new community, returns ID |
| `grant_role(admin, community_id, member, role, expires_at)` | admin | Grant or update membership |
| `revoke_role(admin, community_id, member)` | admin | Revoke access |
| `has_access(member, community_id)` | Public | Check access |
| `get_role(member, community_id)` | Public | Get role record |
| `get_community(community_id)` | Public | Get community info |
| `upgrade(upgrade_admin, wasm_hash)` | upgrade_admin | Upgrade contract WASM |
| `version()` | Public | Returns contract version |

## Events

| Event | Topics | Data |
|---|---|---|
| `community_created` | `community_id`, `admin` | `name` |
| `role_granted` | `community_id`, `member` | `role`, `expires_at` |
| `role_updated` | `community_id`, `member` | `old_role`, `role`, `expires_at` |
| `role_revoked` | `community_id`, `member` | — |
| `contract_upgraded` | — | `wasm_hash` |

## Access Rules

- No role record → no access.
- Revoked role → no access.
- Expired role (`expires_at <= ledger timestamp`) → no access.
- `expires_at == None` → active indefinitely.
- Identical duplicate grant → idempotent no-op.
- Changed role or expiry → updates record and emits `role_updated`.
- Re-grant after revocation → clears revocation, emits `role_granted`.

## Building

```bash
rustup target add wasm32v1-none
cargo test
cargo build --release --target wasm32v1-none
```

## Deploying

```bash
stellar contract deploy \
  --wasm target/wasm32v1-none/release/stellarpass_contract.wasm \
  --source-account <source> \
  --network testnet \
  -- \
  --upgrade_admin <upgrade-admin-address>
```

Record the deployed contract ID and WASM hash in `deployments/testnet.json`.

## Decision Records

| Decision | Value |
|---|---|
| Protocol authority | `upgrade_admin` can upgrade code only |
| Community admin | Creator wallet is sole admin in v1 |
| Roles per member | Exactly one per community |
| Duplicate grants | Idempotent no-op |
| Re-grant after revoke | Allowed |
| Expiry unit | Unix seconds, `<=` ledger timestamp |
| Upgrade | Supported, upgrade-admin only |

## License

MIT
