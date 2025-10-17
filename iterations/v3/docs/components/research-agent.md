# Research Agent

Purpose: Gather, vet, and synthesize context for tasks to reduce worker/model token use and improve decisions.

Inputs:
- Task spec and hints from Router.
- On-demand evidence requests from Council during debate.

Outputs:
- Context bundle: sources, embeddings, summaries, citations.
- Evidence packets tailored to judge/worker needs.

Key Requirements:
- Safe retrieval (no secrets). Respect scope.out.
- De-duplication, source quality scoring, freshness heuristics.
- Vector search integration; cache for reuse.

Interactions:
- Serves both Worker Pool and Council.
- Logs sources for provenance.

