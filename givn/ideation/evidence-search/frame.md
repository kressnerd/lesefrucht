# Frame — Evidence search in one's own library

Topic: `evidence-search`
As of: 2026-10-20. Confirmed (Q1, Option B: the answer is a product goal).

## Problem (one sentence)

A person with a mixed calibre library of a few thousand books (PDF/EPUB/MOBI,
German and English) cannot substantiate a claim from their own books: they
cannot find the passage across book and language boundaries, and a passage they
do find cannot be traced back to an exact position in a specific edition.

## Vision

The owner's library becomes a citable source. A conceptual question in one
language is answered by an answer whose every sentence carries a passage from
the owner's books — with an exact citation that the viewer opens, without
trusting the model's memory. Human and agent receive the same answer; the agent
can additionally retrieve the passages one by one and continue on its own.

## Product boundary

| Owned by the product | Not owned |
|---|---|
| Finding, re-reading and citing passages | The reader's insight; leading the conversation |
| Answering a question from the library, every sentence carrying a citation | Assertions from model knowledge without evidence |
| Bounded synthesis (`compare_positions`, `summarize`) over supplied text only | Free text production without a library basis |
| Classifying a work — writing an empty field above the confidence threshold, proposing otherwise — and proposing other metadata | Catalog authority; calibre remains binding for metadata |
| Abstaining when the library does not support the question | Guesses as a result |

The line is not "search vs. answer" but "cited vs. asserted text": the product
may answer, summarize and compare, but only over text it has taken from the
library, and every sentence carries a citation. An answer the library does not
support is refused, not smoothed.

## First-version non-goals

- Reading progress, notes, highlights — assumption A2 is explicitly unconfirmed.
- A mandatory container; the service runs directly on the shared NixOS host.
- The calibre plugin as the core; only an optional thin client after the API is
  stable (M5).
- A web surface in the first milestone (M1). The cited answer is reachable via
  MCP and CLI from M1; the web surface (M3) is a second surface for the same
  answer, not a goal of its own.
- A replacement for calibre FTS or a general search UI over the library.

## Stakeholders

| Stakeholder | Interest | Touches the system |
|---|---|---|
| Reader and library owner | A cited answer they can check; no plausible-sounding assertion | yes, primary |
| Agent for the reader | Machine-readable, token-budgeted results; passages retrievable one by one; explicit abstention instead of an empty list | yes, primary |
| Library custodian — the same person in another role | Metadata stays their decision; no silent overwrite | yes, writing |
| calibre and its maintainers | Documented contracts only; no access to internals | no |
| Rights holders of the books | No redistribution of text; book text leaves the host only to configured endpoints | no |

## Recorded as given (from the supplied documents)

These are input, not decided here; later phases check them against the domain cut.

- Operation: server-side; shared NixOS host, 8 cores, 64 GB RAM, GPU; the
  calibre desktop may run in parallel.
- calibre: the content server is the only interface; no exclusive access to
  `metadata.db`; writes via `calibredb` against the server URL.
- Model access: OpenAI-compatible endpoints.
- Capacity: one person, LLM-agentic, part-time, no deadline.
- Language: product code in Rust (decision from the second document); Python
  only as a throwaway tool in spike M0.
- License: copyleft is permitted; the concrete license is open and decided via a
  dependency allowlist.
- Acceptance: AK-1 to AK-12, milestones M0 to M5 (from the first document).
  With Q1 Option B, acceptance adds a per-sentence coverage check by a judge
  over recorded answers and the passages that composed them (Q3, closed).

## Research protocol

Read: `README.md` (title only), all twelve `docs/arc42/` chapters (unmodified
templates, empty glossary), `givn/config.yaml`, `givn/commands.yaml`,
`givn/specs/` (empty), `git log` (initial commit).

What the project already owns: nothing domain-specific. No glossary entry, no
specification, no persona, no architecture decision.

What the supplied documents already own: the architecture cut, interface
contracts, technology choice, acceptance criteria, milestones and a set of
constraints. They are treated as given constraints and are not renegotiated in
later phases unless the domain cut contradicts them.

Gaps this ideation must fill: a confirmed frame (done), personas (done), the
event chain with aggregates and invariants, the use-case cut, terminology, and
the open questions (the exact `open_at` syntax and the BGE-M3 license remain;
concurrency is settled as a persistent write queue).
