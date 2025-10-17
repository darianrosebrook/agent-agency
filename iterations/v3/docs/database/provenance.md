# V3 Provenance and Verdict Persistence

Goals:
- Tamper-evident storage of council verdicts and waivers.
- Searchable by task, time, and constitutional references.

Tables (initial sketch):
- verdicts
  - id (uuid, pk)
  - task_id (text)
  - decision (text: accept/reject/modify)
  - votes (jsonb) — array of {judge_id, weight, verdict}
  - dissent (text)
  - remediation (jsonb array of text)
  - constitutional_refs (text[])
  - signature (bytea) — JWS detached payload
  - hash_chain (bytea)
  - created_at (timestamptz default now())

- waivers
  - id (text, pk)
  - reason (text)
  - scope (text)
  - task_id (text)
  - verdict_id (uuid, fk verdicts.id)
  - created_at (timestamptz default now())

Indexes:
- verdicts(created_at desc)
- verdicts(task_id)
- verdicts using gin (constitutional_refs)
- waivers(task_id)

Notes:
- Aligns with ADR-003 for JWS signing (Ed25519).
- Hash chain computed across verdicts per task to detect tampering.
- See migrations: v3/database/migrations/001_create_verdicts.sql
 - Orchestration provides a VerdictWriter abstraction; see Postgres implementation in v3/orchestration/src/persistence_postgres.rs
 - Database connectivity via env var: `DATABASE_URL=postgres://user:pass@localhost:5432/agent_agency`
