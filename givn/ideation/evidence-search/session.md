# Session state — evidence-search

## Position

- Phase: Handoff complete
- Date: 2026-10-20

## Progress

- Frame confirmed (Q1, Option B).
- Six personas confirmed and promoted to `givn/personas/`.
- Twelve domain terms confirmed and promoted to `docs/arc42/12-glossary.md`,
  together with aggregates, invariants and lifecycle events.
- Q2 Genre, Q3 per-sentence judge, Q4 autonomous above threshold, Q5 persistent
  write queue, Q6 language rule, Q7 glossary persona, Q8 classification goal.
- Event Storming complete: 4 aggregates, 24 events, 12 policies, 13 read models.
- Eight use cases elaborated and reviewed.
- Topic document published at `docs/ideation/evidence-search.md`.
- Eight change seeds written under `changes/`, all `Status: ready`.

## Next

- The user runs `/givn-propose` per seed, starting with `passage-retrieval`.
- Handoff started no change.

## Open questions

None — every question is closed.

## Recorded user decisions in this topic

- 2026-10-20 — Q1: the product owns the cited answer (Option B).
- 2026-10-20 — six personas confirmed as review lenses (Q7 added the glossary
  segment).
- 2026-10-20 — output language English; topic renamed `belegsuche` →
  `evidence-search`.
- 2026-10-20 — Q2: the classification is called `Genre`.
- 2026-10-20 — twelve domain terms confirmed.
- 2026-10-20 — the eight-goal cut confirmed; Q8 kept classification as a goal.
- 2026-10-20 — Q6: answer in the question's language, quotations in their
  source language.
- 2026-10-20 — handoff records confirmed.
- 2026-10-20 — Q3: the acceptance run judges each answer sentence against the
  passages that composed it.
- 2026-10-20 — Q4: classification writes an empty field above the confidence
  threshold and proposes below it.
- 2026-10-20 — Q5: every product write is serialized through one persistent
  queue with retry.
