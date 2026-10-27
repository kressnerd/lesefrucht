# arc42 assessment — passage-retrieval

Change: `passage-retrieval` (use case `find-passages`, capability
`passage-retrieval`). Assessment date: 2026-10-27.

## Chapter-by-chapter assessment

| # | Chapter | Affected | Reason / what changed |
|---|---|---|---|
| 1 | Introduction and Goals | Yes | Requirements overview for the first slice (evidence retrieval plus the enabling index), five quality goals (evidence fidelity, honest outcome, determinism, privacy, container-free operability) and the stakeholder table were filled. |
| 2 | Architecture Constraints | Yes | Technical, organisational and convention constraints recorded: Rust, calibre content server as the only interface, required OpenAI-compatible endpoints with no default cloud, no mandatory production container, reproducible offsets, MCP as the agent contract. |
| 3 | Context and Scope | Yes | Business and technical context diagrams and tables now show the new agent surface, the library, the embedding endpoint, the viewer and the boundary of this change. |
| 4 | Solution Strategy | Yes | Key decisions, technology table and the approach to each quality goal were written, referencing ADR-0001 and ADR-0002. |
| 5 | Building Block View | Yes | Level 1 decomposition and level 2 views of `retrieval`, `indexing` and `index` with responsibilities and dependency direction. |
| 6 | Runtime View | Yes | Three runtime scenarios: `search_passages` with abstention and unavailability branches, `read_passage` widening, and edition indexing; plus startup/readiness notes. |
| 7 | Deployment View | Yes | Production systemd topology on the NixOS host and the isolated compose test stack with its two twins. |
| 8 | Cross-cutting Concepts | Yes | Configuration and the outbound boundary, result taxonomy versus errors, determinism, persistence/rebuildability, concurrency and write serialisation, observability, testability, failure handling. |
| 9 | Architecture Decisions | Yes | Register created with ADR-0001 (embedded hybrid index) and ADR-0002 (MCP over Streamable HTTP), both `NEW_ADR`; non-qualifying candidates recorded once with their canonical destinations. |
| 10 | Quality Requirements | Yes | Quality tree and eleven measurable quality scenarios (QS-001 to QS-011) covering citation exactness, determinism, abstention, short/cap, staleness, non-mutation, local runnability, performance envelope and privacy. |
| 11 | Risks and Technical Debt | Yes | R-001 to R-011 and TD-001 to TD-008, including every "Bad, because..." consequence of both ADRs. |
| 12 | Glossary | Yes | The `Question` term and the `Question` aggregate invariant were refined to include the candidate-passage outcome and restate the ranked-work outcome from the confirmed event chain; the invariants table gained the distinguishable-outcome row; a refinement note records the source. No new term was introduced. |

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

### ADR-0002 — MCP over Streamable HTTP with the official Rust SDK

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
contract and the process/deployment boundary. Written to
`docs/arc42/adr/ADR-0002-mcp-streamable-http-transport.md`.

## Candidates routed to canonical artifacts

| Candidate | Gate that failed | Canonical destination |
|---|---|---|
| EPUB/PDF extraction libraries, MOBI/AZW3 explicit failure | Lower-level artifact (single component, reversible in one change) and single-change reversibility | `design.md` (Technology Decisions, Indexing Subfunction) |
| Chapter-aware chunking and passage identity | Lower-level artifact (algorithm and data shape) | `design.md` (Data Model, Indexing Subfunction) |
| Unicode scalar offsets as the citation unit | Lower-level artifact (data-model decision with a spec-visible effect) | `design.md` (Data Model, MCP Contract) |
| Required outbound endpoints, no default cloud | No real alternatives considered by this change: fixed by the frame and the `rights--holder` Persona | chapter 2, `design.md` (Configuration) |
| calibre content server as the only calibre interface | Given constraint, not a decision of this change | chapter 2 |

## Status

`STATUS: DONE`
