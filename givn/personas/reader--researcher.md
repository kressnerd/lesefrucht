# Persona: Reader who must substantiate a claim

## Stable core

- Role: The library owner reading to support an argument, a text or a decision.
- Authority boundary: Decides what counts as evidence and what gets quoted;
  decides nothing about metadata or the library's catalog.
- Evaluation capability: Can judge whether a passage carries the claim, and
  whether the opened position actually shows that passage.
- Own vocabulary: citation, passage, work, edition. Refuses "source" when it
  means a book rather than a position.
- Success criterion: Finds the passage, opens it in the viewer, and the quote
  holds up when checked.

## Running record

- 2026-10-20 — proposed in the ideation topic "evidence-search"; confirmed by
  the user.
- 2026-10-27 — design review of `passage-retrieval`, question Q3: the operator
  chose that the citation's character offsets are the authoritative position and
  `viewer_url` opens the edition, with the position-aware viewer opening
  (`calibre://view-book/...?open_at=...`) deferred as technical debt TD-009.
  Recorded in `givn/changes/passage-retrieval/design-review.md`.

## Topic stances

- 2026-10-20 — evidence-search: the citation must open the exact position, not
  merely the book; a passage that cannot be opened is not evidence.
- 2026-10-27 — evidence-search, qualifier for the agent-facing slice: "exact
  position" is satisfied by the character span the citation carries and
  `read_passage` proves; the human viewer opens the edition now, and one-click
  position-aware opening is deferred with TD-009's resolution condition. The
  2026-10-20 stance stands as the target for a reader-facing surface.
