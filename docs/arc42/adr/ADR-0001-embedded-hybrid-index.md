---
status: accepted
date: 2026-10-27
decision-makers: project operator (owner of the library and this repository)
consulted: none
informed: none
---

# Embedded hybrid index built from Tantivy, usearch and SQLite

## Qualification

ADR-QUALIFICATION: PASS

### Qualification verdict

qualification: QUALIFIED
alternatives: PASS
architectural_impact: PASS
durable_consequence: PASS
lower_level_artifact: PASS
existing_adr_check: PASS
must_be_shared: SUPPORTING
routing: NEW_ADR
canonical_artifact: null
target_adr: null
replacement_adr: null

Evidence:

- alternatives: three materially different, implementable options were
  considered — a single SQLite store (FTS5 plus brute-force cosine), an embedded
  multi-modal store (lancedb), and an external search/vector service; none is a
  strawman, and each was rejected on a stated property.
- architectural_impact: the choice fixes the derived-data boundary (one facade
  owns three stores), the dependency direction between `retrieval`, `indexing`
  and `index`, the deployment boundary (no external service, because the frame
  forbids a mandatory container), and the rebuildability invariant.
- durable_consequence: reversing the choice requires re-indexing the whole
  library and rewriting both retrieval paths and the store facade; the on-disk
  layout is a durable artefact that later changes (glossary expansion,
  classification, cited answers) read.
- lower_level_artifact: `design.md` carries the layout and the query mechanics,
  but it cannot own the deployment and dependency boundary or the three-store
  consistency rule, which is a cross-cutting concept future changes build on.
- existing_adr_check: `docs/arc42/09-architecture-decisions.md` had no register
  rows and `docs/arc42/adr/` contained only the template, so no active or
  archived record covers this boundary; the route is `NEW_ADR`.
- must_be_shared: every contributor touching retrieval, indexing or the store must
  know that only the facade writes and that an interrupted commit is repaired at
  startup, or the three stores silently diverge.

### MADR checklist evidence

- [x] **Hard to reverse** - Reverting later would cost significant effort
      because the lexical schema, the vector key mapping and the SQLite state
      machine all encode the choice, and the library would have to be re-indexed
      through a rewritten store.
- [x] **Shapes quality attributes** - It materially affects retrieval quality and
      operability because it sets the score scales for the evidence threshold and
      removes any service to operate, at the price of three stores to reconcile.
- [x] **Structural** - It defines a boundary, contract, or dependency direction
      because `index::store` becomes the only writer and both retrieval paths
      depend on it instead of on Tantivy or usearch directly.
- [x] **Must be shared** - Developers unfamiliar with it would systematically
      violate the architecture because they would write to Tantivy or usearch
      directly and break the commit ordering and the repair invariant.

The four checklist items are supporting evidence. The five mandatory dimensions
above are the qualification; all five pass with the evidence recorded here.

## Context and Problem Statement

The product needs to retrieve passages lexically and semantically over a
personal calibre library of a few thousand books, on one host, without a
mandatory container and without sending book text anywhere the owner did not
configure. The derived index must be rebuildable from the library because the
library, not the index, is the authority. The choice covers `index::store`, the
`retrieval` paths that read it, and the `indexing` pipeline that writes it.

## Decision Drivers

- Evidence quality: a calibrated lexical score and a cosine score are both
  needed for the evidence threshold.
- No external service: the frame forbids a mandatory container in production.
- Rebuildability: the index is derived, discardable and reproducible.
- One writer: an interrupted commit must never leave a citation pointing at a
  passage the index does not hold.
- One-person operability: no cluster, no managed service, no on-call.

## Considered Options

- Tantivy (lexical) + usearch (vectors) + SQLite (metadata and state) behind one
  store facade
- Single SQLite: FTS5 for lexical, a vector table with brute-force cosine
- lancedb as one embedded store for passages and vectors
- An external search/vector service (Qdrant, pgvector or a hosted index)

## Decision Outcome

Chosen option: "Tantivy + usearch + SQLite behind one store facade", because it
is the only option that keeps per-language BM25, an approximate-nearest-neighbour
vector index and transactional metadata in-process on one host, with no service
to operate and no dependency that reshapes the repository.

### Consequences

- Good, because the evidence threshold can be expressed on two calibrated
  scales, each tuned independently per path.
- Good, because the derived index is three plain files under `data/`, rebuildable
  by deleting the directory.
- Bad, because three stores have no shared transaction, so a commit can be
  partially applied and the product must carry a startup repair pass.
- Bad, because a rebuild re-embeds every passage, which costs endpoint calls
  proportional to the library.
- Bad, because usearch is a native dependency with its own persistence rules,
  and deleted passages require periodic compaction to reclaim space.
- Bad, because the two score scales must be tuned together, and a threshold that
  fits one corpus may not fit another.

### Confirmation

`tests/steps/passage_retrieval.rs` asserts the deterministic order and the
`short`/cap behaviour through the store facade; the setup task adds a golden test
that indexes the fixture library twice and asserts identical passage ids,
offsets and texts (idempotent re-indexing), and a repair test that interrupts a
commit and asserts the edition is marked `invalidated` and re-indexed. A review
checkpoint confirms that no module other than `index::store` constructs a
Tantivy writer or a usearch save.

## Pros and Cons of the Options

### Tantivy + usearch + SQLite behind one store facade

- Good, because each store does what it is best at: BM25, ANN, transactions.
- Good, because all three are embedded, licence-compatible for a copyleft-allowed
  project, and rebuildable.
- Neutral, because the facade adds a layer that must be respected.
- Bad, because three stores need reconciliation and a repair pass.

### Single SQLite: FTS5 plus brute-force cosine

- Good, because one file, one transaction, the simplest possible operation.
- Good, because no native dependency beyond SQLite itself.
- Bad, because FTS5's BM25 cannot apply per-language stemming to German
  compounds, which is where the library's evidence lives.
- Bad, because a brute-force cosine scan over millions of passages costs a full
  scan per query and does not fit the interactive expectation.

### lancedb as one embedded store

- Good, because it stores passages, metadata and vectors in one place.
- Bad, because it brings Arrow/Lance and a second query dialect for a need that
  one HNSW file satisfies.
- Bad, because its own format versioning becomes a migration burden for a
  one-person project.

### An external search/vector service

- Good, because operations and scaling are someone else's problem.
- Bad, because it violates "no mandatory container" and adds a service the owner
  must run for a single-user library.
- Bad, because book text would flow to a component that is not the configured
  endpoint, weakening the rights-holder boundary.

## More Information

Design detail — layout, schema, commit ordering, repair algorithm, thresholds —
is in `givn/changes/passage-retrieval/design.md` (Technology Decisions, Data Model, Retrieval). The three-store
consistency cost is tracked in arc42 chapter 11.
