# arc42 assessment — passage-retrieval

Change: `passage-retrieval` (use case `find-passages`, capability
`passage-retrieval`). Assessment date: 2026-10-27.

## Chapter-by-chapter assessment

| # | Chapter | Affected | Reason / what changed |
|---|---|---|---|
| 1 | Introduction and Goals | Yes | Requirements overview for the first slice (evidence retrieval plus the enabling index), five quality goals (evidence fidelity, honest outcome, determinism, privacy, container-free operability) and the stakeholder table were filled. |
| 2 | Architecture Constraints | Yes | Technical, organisational and convention constraints recorded: Rust, calibre reached only through documented interfaces (`calibredb list`/`export`), required library target and OpenAI-compatible endpoint with no default cloud, no mandatory production container, reproducible offsets, MCP as the agent contract, MPL-2.0 product licence. |
| 3 | Context and Scope | Yes | Business and technical context diagrams and tables show the new agent surface, the calibre library reached through the `calibredb` process boundary, the embedding endpoint, the viewer (edition link; offsets authoritative) and the boundary of this change. |
| 4 | Solution Strategy | Yes | Key decisions, technology table (including the `calibredb` library interface and the SDK choice owned by `design.md`) and the approach to each quality goal were written, referencing ADR-0001 and ADR-0002. |
| 5 | Building Block View | Yes | Level 1 decomposition and level 2 views of `retrieval`, `library` (calibredb client), `indexing` and `index` with responsibilities and dependency direction. |
| 6 | Runtime View | Yes | Three runtime scenarios: `search_passages` with abstention (including the `indexing` summary), unavailability and unreachable-library branches, `read_passage` widening, and edition indexing through `calibredb export`; plus startup/readiness notes. |
| 7 | Deployment View | Yes | Production systemd topology on the NixOS host (calibre CLI on the host, writable `TMPDIR`) and the isolated compose test stack with a fixture calibre library and one embedding twin; no calibre twin. |
| 8 | Cross-cutting Concepts | Yes | Configuration and the outbound boundary, the `calibredb` subprocess boundary and temporary files, result taxonomy versus errors (`indexing` summary), determinism, persistence/rebuildability, concurrency and write serialisation, observability, testability, failure handling. |
| 9 | Architecture Decisions | Yes | Register created with ADR-0001 (embedded hybrid index) and ADR-0002 (MCP over Streamable HTTP only), both `NEW_ADR`; the SDK implementation, the calibredb call shape and the outbound-endpoint boundary are routed once to `design.md`; each routed row names exactly one destination. |
| 10 | Quality Requirements | Yes | Quality tree and twelve measurable quality scenarios (QS-001 to QS-012) covering citation exactness, repeat determinism, abstention, short/cap, staleness and unavailability, non-mutation, local runnability, performance envelope, privacy and the indexing-failure abstention. |
| 11 | Risks and Technical Debt | Yes | R-001 to R-014 and TD-001 to TD-010, including every "Bad, because..." consequence of both ADRs, the `calibredb` availability/version-drift risks and the deferred position-aware viewer opening (TD-009). |
| 12 | Glossary | Yes | The `Question` term and aggregate were refined for the candidate-passage outcome; rows were added for `Work`, `Work id`, `Chapter`, `Claim`, `Expansion`, `Passage index`, `Short result`, `Unavailability report` and `Candidate-passage result`; `Abstention` is mapped to `no evidence`/`NoEvidence`; the numeric work-id ordering and the `Citation` refinement (offsets authoritative, viewer deferred, TD-009) are recorded with their sources. |

## ADR qualification verdicts

Both candidates were tested against the five mandatory dimensions and the three
falsification tests.

### ADR-0001 — Embedded hybrid index (Tantivy + usearch + SQLite)

```json
{
  "qualification": "QUALIFIED",
  "alternatives": "PASS",
  "architectural_impact": "PASS",
  "durable_consequence": "PASS",
  "lower_level_artifact": "PASS",
  "existing_adr_check": "PASS",
  "must_be_shared": "SUPPORTING",
  "routing": "NEW_ADR",
  "canonical_artifact": null,
  "target_adr": null,
  "replacement_adr": null
}
```

Falsification: not reversible within one change (re-index plus two retrieval
paths plus the store), constrains at least two future changes (glossary
expansion and cited answers both read this index), and it is not product
workflow behaviour — it moves the data and deployment boundary. Written to
`docs/arc42/adr/ADR-0001-embedded-hybrid-index.md`.

### ADR-0002 — MCP over Streamable HTTP

```json
{
  "qualification": "QUALIFIED",
  "alternatives": "PASS",
  "architectural_impact": "PASS",
  "durable_consequence": "PASS",
  "lower_level_artifact": "PASS",
  "existing_adr_check": "PASS",
  "must_be_shared": "SUPPORTING",
  "routing": "NEW_ADR",
  "canonical_artifact": null,
  "target_adr": null,
  "replacement_adr": null
}
```

Falsification: not reversible within one change (server, client and e2e driver
are all written against it), constrains the future web surface and any later
tool, and it is not observable workflow behaviour — it fixes the external
contract and the process/deployment boundary. One decision only: the SDK
implementation (official `rmcp` versus hand-rolled JSON-RPC) is routed to
`design.md` and is not part of this ADR. Written to
`docs/arc42/adr/ADR-0002-mcp-streamable-http-transport.md`.

## Candidates routed to canonical artifacts

| Candidate | Gate that failed | Canonical destination |
|---|---|---|
| EPUB/PDF extraction libraries, MOBI/AZW3 explicit failure | Lower-level artifact (single component, reversible in one change) and single-change reversibility | `design.md` (Technology Decisions, Indexing Subfunction) |
| Chapter-aware chunking and passage identity | Lower-level artifact (algorithm and data shape) | `design.md` (Data Model, Indexing Subfunction) |
| Unicode scalar offsets as the citation unit | Lower-level artifact (data-model decision with a spec-visible effect) | `design.md` (Data Model, MCP Contract) |
| Required outbound endpoints, no default cloud | No real alternatives considered by this change: fixed by the frame and the `rights--holder` Persona | `design.md` (Configuration) |
| calibre reached only through documented interfaces | Given constraint, not a decision of this change | chapter 2 |
| Library discovery and edition bytes through `calibredb list`/`calibredb export` | Lower-level artifact (the constraint is given; the command shape is a component detail) | `design.md` (Technology Decisions › Stack, Indexing Subfunction) |
| MCP implementation: official Rust SDK versus hand-rolled JSON-RPC | Lower-level artifact (dependency choice inside `mcp` and the e2e driver; ADR-0002 owns the transport boundary) | `design.md` (Technology Decisions › MCP SDK implementation) |

## Status

`STATUS: DONE`
