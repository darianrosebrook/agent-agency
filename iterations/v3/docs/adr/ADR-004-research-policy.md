# ADR-004: Research Agent Source Policy and Caching

Status: Proposed

Decision (Proposed):
- Allowed sources: repo-local files within scope.in, approved docs, configured web domains.
- Disallow secrets and .env; enforce allowlist validator.
- Cache: vector store with TTL and freshness heuristics; offline mode falls back to local corpus.

Rationale:
- Balances safety, performance, and reproducibility.

Risks:
- Staleness; mitigated by freshness checks and TTL.

