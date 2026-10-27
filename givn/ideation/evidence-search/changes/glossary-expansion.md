# Change seed: glossary-expansion

Status: ready

- Use-case IDs: maintain-glossary
- Topic: evidence-search
- Personas: custodian--glossary, agent--for-reader, reader--researcher
- Suggested start: 4 — depends on passage-retrieval for an observable effect
- Handoff: handoff confirmed 2026-10-20; ready for `/givn-propose`

## Accepted decisions

### The glossary has its own custodian segment (Q7)

- Context: The confirmed personas give the metadata custodian authority over the
  calibre catalog; the glossary is a separate store of concepts and terms per
  language, and no persona covered it.
- Options:
  1. **Extend the metadata custodian's authority.**
     Gain: no sixth persona. Cost: two decision objects in one lens.
  2. **A separate persona segment, `custodian--glossary`.**
     Gain: a precise authority boundary. Cost: one more lens per artifact.
  3. **The reader owns it.**
     Gain: vocabulary grows where it is used. Cost: changes a confirmed persona.
- Chosen: **Option 2** because the glossary is a different authority object and
  the segment keeps the review lens precise. Rejected: Option 1 because it
  blurs two decision objects; Option 3 because it moves catalog-adjacent
  authority to a role that decides nothing about the library's apparatus.

### The product owns the cited answer (Q1)

- Context: Deterministic expansion is part of retrieval; it must work without
  composition.
- Options: **evidence with bounded synthesis**, **the answer as a product goal**.
- Chosen: **Option 2** for the product; within it, accepted terms expand a
  question before any composition runs. Rejected: Option 1 as the boundary.

## Open questions

None.
