# 4. Solution Strategy

## Key decisions

- **One derived index, three cooperating stores.** A SQLite database holds
  metadata, index state, passages and citations; a Tantivy directory holds the
  lexical index; a usearch file holds the vectors. One store facade is the only
  writer, so the three cannot be updated by different components. Rationale:
  ADR-0001.
- **The agent talks MCP over Streamable HTTP.** One endpoint serves `initialize`
  and both tools with SDK-managed sessions, so an agent connects remotely and
  the same surface carries the readiness probe. Rationale: ADR-0002.
- **Retrieval is hybrid and scope-first.** Lexical and vector candidates are
  produced independently, the scope is applied to both before fusion, the
  evidence threshold is applied per path, and Reciprocal Rank Fusion produces a
  score whose ties are broken by work id and passage index. This is what makes
  "no evidence" a decision rather than an accident, and the order deterministic.
- **Indexing is a subfunction, not a goal.** Discovery, edition selection,
  extraction, chapter-aware chunking, embedding and state tracking run inside
  the observing flow; the library is the authority and the index is derived and
  rebuildable.
- **Deterministic expansion before model expansion.** Expansion draws on
  accepted glossary terms and a configured synonym map. The glossary is empty in
  this slice, so the configured map is the only source; a model-based expansion
  would make the deterministic order and the evidence threshold depend on a
  model call.
- **Explicit failure over silent omission.** A format the product cannot read is
  `IndexingFailed` with a reason, so the agent can tell a missing passage from a
  missing capability.

## Technology choices

| Concern | Choice | Rationale (see ADR) |
|---|---|---|
| Language | Rust (stable, edition 2024) | Given constraint; one language for product, twins and fixtures. |
| Async runtime / HTTP | tokio with axum | I/O-bound server with one concurrency model. |
| Agent protocol | MCP over Streamable HTTP via the official Rust SDK (`rmcp`) | ADR-0002. |
| Lexical index | Tantivy, BM25, per-language stemming | Calibrated, per-language scores for the evidence threshold. |
| Vector index | usearch (HNSW, cosine), embedded single file | ADR-0001. |
| Metadata and index state | SQLite (WAL) via rusqlite | Transactional state machine, no service to run. |
| Embeddings | Configured OpenAI-compatible endpoint | Given constraint; no default cloud endpoint. |
| Extraction | rbook (EPUB 2/3), pdf-extract (PDF); AZW3/MOBI fail explicitly | Chapter structure for EPUB; explicit failure for the rest. |
| Chunking | text-splitter, character capacity, chapter-bounded | Reproducible offsets across re-indexing. |
| Gherkin runner | cucumber-rs with `fail_on_skipped()` | Rust, strict by configuration. |
| Local infrastructure | rootless podman-compose with two behavioural twins | Docker is unavailable on this host; behavioural twins make the stale-index case expressible. |

## Approach to quality goals

| Quality attribute | Approach |
|---|---|
| Evidence fidelity | Offsets are Unicode scalar indices into stored normalized text; `read_passage` verifies the cited span's hash against the passage record; a citation is only produced from an indexed edition. |
| Honest outcome | Three mutually exclusive tagged outcomes; `short` flags a result below the requested number; the freshness check runs before retrieval so unavailability cannot masquerade as `no evidence`. |
| Determinism | Content-hash-keyed idempotent indexing; deterministic keyed expansion; per-path thresholds; RRF plus a total order on `(score, work id, passage index)`. |
| Privacy of the library | Both outbound endpoints are required configuration, validated at startup; no telemetry path exists; the test twins are the only endpoints used in tests. |
| Operability without a container | Single binary plus a data directory; the index is rebuildable; production runs under systemd on the owner's host. |
| Testability | One command brings up the product and its twins; both twins are also mounted in-process for the non-e2e scenarios, so the same production paths run at two speeds. |
