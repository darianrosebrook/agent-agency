# ADR-003: Verdict Schema and Provenance Signing

Status: Proposed

Decision (Proposed):
- Use JSON per schemas: judge-verdict, final-verdict.
- Sign final verdict with JWS (Ed25519 keypair stored in secure enclave when available).
- Store verdict, signatures, and evidence refs in provenance log.

Rationale:
- Tamper-evident audit trail aligned with CAWS.

Open Questions:
- Rotation policy for keys; verification in CI.

