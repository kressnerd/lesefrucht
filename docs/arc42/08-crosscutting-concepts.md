# 8. Cross-cutting Concepts

## Configuration and the outbound boundary

One TOML file plus `LESEFRUCHT__<SECTION>__<KEY>` environment overrides. Every
value has a documented default except the outbound endpoints: `library.base_url`,
`embedding.base_url`, `embedding.model` and a positive `embedding.dimension`
are required, and startup fails with a named error when one is absent. There is
no default cloud endpoint, no fallback host and no telemetry path, so the
rights-holder boundary ("book text leaves the host only to configured
endpoints") is enforced by construction rather than by review.

## Result taxonomy versus errors

The agent surface has exactly two error channels and they do not overlap:

| Channel | Carries | Consumer action |
|---|---|---|
| Tagged result (`structuredContent.kind`) | `candidate_passages`, `no_evidence` (reason `no_evidence` or `unknown_work`), `unavailable` (reason `index_unavailable`, `stale_index`, `library_unreachable`), `passage_context` | Continue composing, or report the refusal to the reader |
| JSON-RPC error (`isError: true`) | Malformed arguments (`invalid_arguments`), a citation that contradicts the index (`invalid_citation`) | Fix the request |

An abstention is never an error and never an empty success; the `kind` field is
the single discriminator, so a client cannot confuse the two. This is the
cross-cutting application of R6.

## Determinism

Every stage is a pure function of its inputs:

- Indexing: content hash decides whether work happens; chunk offsets derive from
  the normalized text; passage identity derives from content hash, chapter,
  ordinal and offsets; embedding is deterministic given the configured endpoint.
- Expansion: accepted glossary terms plus a configured synonym map, applied in a
  fixed order and de-duplicated.
- Ranking: per-path scores, a fixed threshold, RRF with a fixed `k`, and a total
  order on `(score, work id, passage index)`.

Floating-point score values are not contract; membership, order, counts and the
`short` flag are.

## Persistence and rebuildability

The library is the authority; the index is derived. `data/` can be deleted and
rebuilt from the content server, so it is not backed up and not migrated between
releases beyond the SQLite schema migrations it carries. Metadata is never
written back to calibre in this change, so a retrieval cannot damage the source
of truth.

## Concurrency and write serialisation

One indexing consumer serialises extraction and embedding; one store facade
serialises writes to the three stores; retrieval reads through the store
facade with SQLite WAL and a reloaded Tantivy reader. Startup repair reconciles
the three stores after an interrupted commit, so a crash degrades to re-indexing
one edition, never to a wrong citation.

## Observability

Structured tracing with a request id per tool call. Log records carry ids,
counts, timings, outcome kinds and failure reasons — never book text, never a
passage body, never an embedding vector. `/healthz` is the single operational
read: process liveness, indexed and failed edition counts, last successful index
time, library generation and store state.

## Testability

The two external dependencies are behavioural twins in the same workspace, so
the e2e stack needs no network and no live service. The same twin routers are
mounted in-process for the non-e2e scenarios, which keeps one implementation of
each fake and lets the fast scenarios drive the real production paths. Fixtures
are generated from reviewable text by an `xtask`, so the repository holds text
and a generator rather than opaque binaries.

## Failure handling

Indexing failures are typed (`unsupported_format`, `extraction_failed`,
`embedding_failed`, `source_unreachable`, `empty_text`) and persisted with the
edition state; embedding calls are retried a bounded number of times with
backoff before failing. Retrieval failures that are environmental become
unavailability reports, because the library could not be checked. A library
inconsistency (a citation that no longer resolves) is a protocol error with
`invalid_citation`, because the request contradicts the index and retrying is
pointless.
