# 1. Introduction and Goals

## Requirements overview

The product turns a private calibre library into a citable source: a reader, or
the agent acting for them, asks a question about the books and receives a
response in which every sentence carries a passage from the library, with a
citation that opens the exact position.

The first change (`passage-retrieval`, use case `find-passages`) delivers the
evidence side and the conditions for it:

1. The library's editions are indexed — extracted, chunked chapter by chapter,
   embedded and written into a derived index — so that a passage can be found.
2. An agent submits a claim or phrase in one language with an optional scope and
   receives up to the requested number of candidate passages, at most twenty,
   each with a citation, a score and the expansion used, in a deterministic
   order.
3. The agent widens the context around a cited passage and receives the
   surrounding text with exact bounds.
4. When the library cannot support the request, the agent receives `no evidence`
   with its reason or an unavailability report — never an empty success and
   never a failure disguised as a result.

The authoritative behaviour specification is the Gherkin under `givn/specs/`
once this change is archived, and
`givn/changes/passage-retrieval/specs/find-passages/passage-retrieval.feature`
until then. The use-case contract is `givn/specs/find-passages/usecase.md`.

## Quality goals

| Priority | Quality attribute | Motivation |
|---|---|---|
| 1 | Evidence fidelity | A returned passage must exist at its citation, character for character; a passage that cannot be opened is not evidence (`reader--researcher`). |
| 2 | Honest outcome | `no evidence`, `unknown work` and an unavailability report are distinguishable results, never an empty list and never an error (`agent--for-reader`, R6). |
| 3 | Determinism | The same request against the same index yields the same sequence of passages, so an agent can reason about and cite the result. |
| 4 | Privacy of the library | Book text leaves the host only to endpoints the owner configured; there is no default cloud endpoint and no telemetry carrying text (`rights--holder`). |
| 5 | Operability without a container | The service runs directly on the owner's NixOS host; the index is derived and rebuildable, so a lost index is an inconvenience, not a data loss. |

## Stakeholders

| Role | Expectation |
|---|---|
| Reader and library owner | Findings they can check: a cited passage opens at the exact position, and the product says so when it has nothing. |
| Agent for the reader | A machine-readable, token-bounded result with a distinguishable abstention and a deterministic order. |
| Library custodian (the same person in another role) | No silent change to calibre metadata; this change writes nothing to the library. |
| calibre and its maintainers | Only documented interfaces are used: the content server, never `metadata.db` internals. |
| Rights holders of the books | No redistribution; every outbound path is configured by the owner. |
