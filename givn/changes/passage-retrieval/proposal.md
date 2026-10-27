# Proposal: passage-retrieval

## Release note

An agent can submit a claim or phrase to the indexed library and receive up to
twenty candidate passages, each with the exact citation it opens, the score that
ranked it and the expansion that produced it, plus wider context on request and
an explicit refusal or unavailability report when the library cannot supply
evidence.

## Use-Case Context

- Use-case ID: `find-passages`
- Ideation topic: `evidence-search`
- Seed path: `givn/ideation/evidence-search/changes/passage-retrieval.md`
- Confirmed Personas: `agent--for-reader`, `reader--researcher`
  - `agent--for-reader` is the caller and checks that `no evidence` is
    distinguishable from failure and that the order is deterministic.
  - `reader--researcher` is the reader on whose behalf the agent calls, and
    checks that a citation opens the exact position.
  - `rights--holder` is a boundary lens on the outbound path (book text may
    leave the host only to endpoints the owner configured); it does not own this
    goal and is not listed as a Persona of `find-passages` in the use-case cut.

## Problem / Opportunity

A reader who wants to substantiate a claim from their own library cannot do it.
The library is a few thousand calibre books in German and English across EPUB,
PDF and MOBI, and the evidence for one claim is scattered: the same idea appears
in several works, the decisive sentence may sit in an English book while the
claim is German, and a passage that is found cannot be traced back to an exact
position in a specific edition. Today the reader or their agent has to remember
where something was said, or trust a model's memory, because there is no way to
retrieve the passage itself.

The opportunity is the first slice of the library as a citable source: the
agent receives candidate passages with citations it can check, instead of prose
it cannot. Nothing of the product exists yet — no specification, no index, no
agent-facing interface — so this change also has to create the conditions under
which a passage can be found at all: the library's editions have to be indexed.

## Proposed Solution

The product indexes the editions of the library it can read, so that a result
can exist, and then answers the agent's request from that index.

What the agent can observe:

- **Submit a claim or phrase.** The agent sends a claim or search phrase in one
  language together with an optional scope — genre, language, authors, a year
  range, or named works — and a requested number of passages. It receives up to
  that number of candidate passages: never more than it asked for, and never
  more than twenty even when it asks for more. Each passage carries a citation
  to an edition the library holds, a score, and the expansion that was used to
  find it. The order is deterministic: score first, then work, then position
  within the edition, so the same request against the same index always yields
  the same sequence.
- **Widen the context.** The agent can ask for the text surrounding a cited
  passage and receives it with exact bounds, so the cited passage appears
  verbatim inside a larger stretch the reader can read. The cited position
  itself is never approximated.
- **Receive a refusal rather than an empty success.** When no passage reaches
  the evidence threshold the agent receives `no evidence` with its reason. When
  the scope names a work the library does not hold it receives `no evidence`
  with reason `unknown work`. When the index is unavailable or older than the
  freshness bound it receives an unavailability report, distinct from
  `no evidence`. None of these three is reported as a failure, and none is
  reported as a successful empty list.
- **Receive a short result honestly.** When fewer passages than requested exist
  above the threshold, the agent receives what exists and the result is marked
  as short. Nothing is padded to reach the requested number.
- **Change nothing.** A retrieval and a context widening leave the library, its
  metadata and the index state unchanged.
- **See indexing that does not lie.** The index is built from the library's
  editions. An edition in a format the product cannot read in this slice is
  reported as failed indexing with a reason, never silently skipped, so the
  agent can tell a missing passage from a missing capability.

The scope is applied to the request before results are combined, so a scoped
request can never return a passage from outside the scope.

## Capability Routing

`givn spec route --change passage-retrieval` reported no signal (no command,
token, or lexical overlap), so no recommendation is recorded. The decision is
made here, with its rationale.

| Proposed capability | Decision | Rationale |
|---|---|---|
| `passage-retrieval` | `NEW in find-passages` | No existing capability can be extended: `givn/specs/` is empty and this is the first slice of the product. The capability advances the `find-passages` goal — the agent holds candidate passages for a claim, each with a citation — and contains both consumer actions of that goal: submitting a claim with an optional scope, and widening context around a cited passage. The goal is retrieval of evidence, so the capability is named for that goal and not for a channel, a component or the index that enables it. |

## Domain refinement

The confirmed term `Question` says a Question resolves to exactly one answer,
one abstention with its reason, or one unavailability report. The seed builds
the `Question` aggregate on the retrieval side, where the outcome is candidate
passages. The passages outcome is therefore made explicit as a refinement of the
same term, not as a new term: a `Question` in `find-passages` resolves to
exactly one **candidate-passage result**, one abstention with its reason, or one
unavailability report. The request stays a `Question`; no second term such as
"Query" is introduced. `docs/arc42/12-glossary.md` carries the refinement, and
`design.md` records it in the decisions ledger.

## Out of Scope

- `answer-question` — composing cited prose. This slice returns passages; it
  makes no claim about whether a passage supports the claim.
- `find-works`, `compare-positions`, `summarize-work` — ranking works,
  comparing named works, summarizing a work.
- `classify-works`, `review-metadata-proposals`, `maintain-glossary` — the
  changing cases. This slice reads an existing genre tag and writes nothing.
- The web surface and any reader-facing CLI for this slice. The surface here is
  the agent's; the reader is a supporting actor and a Persona lens, not the
  caller.
- The calibre plugin.
- Reading progress, notes and highlights.
- A general search UI over the library, and a replacement for calibre's own
  full-text search.

## Open Questions

None unresolved. The seed records no open question, and every question of the
topic that could intersect this change is dispositioned below. Implementation
questions are decided in `design.md`, not here.

### Seed question dispositions

- **Q1 — Product boundary: evidence or answer — answered (seed).** The seed
  records the cited answer as a product goal. `find-passages` is the evidence
  side of that boundary: it returns passages, and the cited answer is
  `answer-question`.
- **Q2 — Name of the library classification — answered (seed).** `Genre` is the
  scope term and the calibre tag namespace this slice reads as
  `genre.<class>`. Classifying a work is out of scope.
- **Q3 — Acceptance criterion for a cited answer — out of scope.** It measures
  prose answers, which `answer-question` composes; this slice returns passages.
- **Q4 — Autonomous first run into empty metadata fields — out of scope.** It
  governs classification writes; this slice never writes to calibre.
- **Q5 — Write safety while the calibre desktop is open — out of scope.** It
  concerns product writes to calibre metadata; this slice performs none.
- **Q6 — Language of an answer and its quotations — out of scope.** It decides
  answer language. This slice returns each passage in its source language
  verbatim, which decides no answer-language rule.
- **Q7 — Who owns the glossary — out of scope.** Glossary authority belongs to
  `maintain-glossary`; this slice only reads accepted terms, and the glossary is
  empty.
- **Q8 — Is classification a goal of the custodian or a subfunction — out of
  scope.** It concerns `classify-works`; this slice only reads a genre that
  already exists.
