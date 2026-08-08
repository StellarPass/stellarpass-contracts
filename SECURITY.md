# Security

Report vulnerabilities privately. See the [organization security policy](https://github.com/StellarPass/.github/blob/main/SECURITY.md).

## Contract Security

- The `upgrade_admin` can upgrade contract code but cannot create communities or administer memberships.
- Only community-specific admins can grant or revoke roles, enforced by Soroban `require_auth()`.
- Contract upgrades should use a Stellar multisig account on mainnet.
- Never commit deployment keys, deployer seed phrases, or upgrade-admin credentials.

## Audit

Contract has not been audited. Request an independent security review before mainnet deployment.
