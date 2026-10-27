# 8. Cross-cutting Concepts

## Configuration and the outbound boundary

One TOML file plus `LESEFRUCHT__<SECTION>__<KEY>` environment overrides. Every
value has a documented default except the library target and the embedding
endpoint: `library.url` (the `calibredb --with-library` value: a content-server
URL or a library path), `embedding.base_url`, `embedding.model` and a positive
`embedding.dimension` are required, and startup fails with a named error when one
is absent or when `library.calibredb_binary` cannot be executed. There is no
default cloud endpoint, no fallback host and no telemetry path, so the
rights-holder boundary ("book text leaves the host only to configured
endpoints") is enforced by construction rather than by review.

## Subprocess boundary and temporary files

calibre is reached by spawning `calibredb`, never by a library call or an HTTP
route. The boundary has four rules:

- Every invocation passes `--with-library <library.url>` explicitly and is
  bounded by `library.timeout_ms`; a timeout, a non-zero exit or unparsable
  output is the typed failure `source_unreachable`, never a partial listing.
- Listing output is parsed as one JSON array of per-book objects. Field
  expectations are fixed (`id`, `title`, `authors`, `languages`, `tags`,
  `pubdate`, `formats`, `last_modified`); an unexpected shape fails loudly.
- `calibredb export` writes into a per-run temporary directory created under
  `TMPDIR`; the directory is removed on success and on failure, and the product
  copies only the normalized text it needs into `data/`.
- No shell is involved: the binary and its argument vector are passed directly,
  so a library path containing spaces or shell metacharacters is safe.

The known limitation is recorded rather than hidden: `calibredb` joins multiple
authors into one ` & `-separated string even in machine output, so the author
list is split on that documented separator.

## Result taxonomy versus errors

The agent surface has exactly two error channels and they do not overlap:

| Channel | Carries | Consumer action |
|---|---|---|
| Tagged result (`structuredContent.kind`) | `candidate_passages`, `no_evidence` (reason `no_evidence` or `unknown_work`, optionally with the `indexing` summary of failed editions), `unavailable` (reason `index_unavailable`, `stale_index`, `library_unreachable`), `passage_context` | Continue composing, or report the abstention to the reader |
| JSON-RPC error (`isError: true`) | Malformed arguments (`invalid_arguments`), a citation that contradicts the index (`invalid_citation`) | Fix the request |

An abstention is never an error and never an empty success; the `kind` field is
the single discriminator, so a client cannot confuse the two. This is the
cross-cutting application of R6.

## Determinism

Every stage is a pure function of its inputs:

- Indexing: content hash decides whether work happens; chunk offsets derive from
  the normalized text; passage identity derives from content hash, chapter,
  ordinal and offsets; embedding is deterministic given the configured endpoint;
  the listing parse and the generation hash are pure functions of the
  `calibredb list` JSON (no clock, no ordering freedom).
- Expansion: accepted glossary terms plus a configured synonym map, applied in a
  fixed order and de-duplicated.
- Ranking: per-path scores, a fixed threshold, RRF with a fixed `k`, and a total
  order on `(score, work id, passage index)`.

Floating-point score values are not contract; membership, order, counts and the
`short` flag are.

## Persistence and rebuildability

The library is the authority; the index is derived. `data/` can be deleted and
rebuilt from the library through `calibredb`, so it is not backed up and not
migrated between releases beyond the SQLite schema migrations it carries.
Exported edition bytes live only in the per-run temp directory and never in
`data/`; the derived text files are keyed by content hash. Metadata is never
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
passage body, never an embedding vector. `/healthz` is the single **operational**
read: process liveness, indexed and failed edition counts, last successful index
time, library generation and store state. It is not an agent surface: the agent
learns about failed editions only through the optional `indexing` summary of an
abstention and cannot enumerate failures on demand.

## Testability

The two external dependencies are handled differently, on purpose. calibre is
not faked at all: tests run the real `calibredb` against a fixture calibre
library that `xtask fixtures` builds with `calibredb add` at fixed mtimes, so
the CLI contract cannot drift from a twin. The embedding endpoint is an HTTP
process boundary and is represented by a behavioural twin whose vector is a pure
function of the input text; the same twin router is mounted in-process for the
non-e2e scenarios. Fixtures are generated from reviewable text, so the repository
holds text and a generator rather than opaque binaries.

## Failure handling

Indexing failures are typed (`unsupported_format`, `extraction_failed`,
`embedding_failed`, `source_unreachable`, `empty_text`) and persisted with the
edition state; embedding calls are retried a bounded number of times with
backoff before failing. A failing, timing-out or unparsable `calibredb`
invocation is `source_unreachable` during indexing and
`unavailable { reason: library_unreachable }` during a request. Retrieval
failures that are environmental become unavailability reports, because the
library could not be checked. A library inconsistency (a citation that no longer
resolves) is a protocol error with `invalid_citation`, because the request
contradicts the index and retrying is pointless.
