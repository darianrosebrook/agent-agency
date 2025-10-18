-- Add contract payload column to verdicts for interoperability tracking
ALTER TABLE verdicts
    ADD COLUMN IF NOT EXISTS contract JSONB NOT NULL DEFAULT '{}'::jsonb;

-- Ensure existing rows (if any) have deterministic default values
UPDATE verdicts
SET contract = json_build_object(
        'decision', decision,
        'votes', votes,
        'dissent', dissent,
        'remediation', remediation,
        'constitutional_refs', constitutional_refs
    )
WHERE contract = '{}'::jsonb;
