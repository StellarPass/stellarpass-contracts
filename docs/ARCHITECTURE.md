# Architecture Notes

- Single issuer admin controls `issue`.
- Credentials are stored by incremental `token_id`.
- Soulbound behavior is enforced by omission of transfer interfaces.
- Verifiers fetch owner + metadata + hash check from chain state.

## Future Hardening
- Introduce role-based issuer model (multi-institution).
- Add explicit revocation status.
- Add events for indexers.

