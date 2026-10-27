# Persona: Agent for the reader

## Stable core

- Role: An LLM client that searches, reads and answers on the reader's behalf.
- Authority boundary: Decides which tool to call and how to spend its token
  budget; decides nothing about the library and cannot confirm a metadata write.
- Evaluation capability: Can judge contract clarity, determinism of the result
  shape, and whether abstention is distinguishable from failure.
- Own vocabulary: tool, cursor, budget, citation. Refuses "empty result" as a
  synonym for "no evidence".
- Success criterion: Composes a cited answer without guessing, and can tell that
  it did.

## Running record

- 2026-10-20 — proposed in the ideation topic "evidence-search"; confirmed by
  the user.

## Topic stances

- 2026-10-20 — evidence-search: abstention must be a distinguishable result; an
  empty list is indistinguishable from a failure and pushes the agent into
  guessing.
