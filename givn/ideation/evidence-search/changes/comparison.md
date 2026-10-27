# Change seed: comparison

Status: ready

- Use-case IDs: compare-positions
- Topic: evidence-search
- Personas: reader--researcher, agent--for-reader, rights--holder
- Suggested start: 7 — depends on passage-retrieval and cited-answer
- Handoff: handoff confirmed 2026-10-20; ready for `/givn-propose`

## Accepted decisions

### The product owns the cited answer (Q1)

- Context: A comparison is prose and must be governed by the same boundary as an
  answer.
- Options: **evidence with bounded synthesis**, **the answer as a product goal**.
- Chosen: **Option 2**, with the comparison limited to the named works.
  Rejected: Option 1 because the agent would have to compose it.

### Answer in the question's language, quotations in the source language (Q6)

- Context: A comparison cites works in both languages.
- Options:
  1. **Quotations in their source language.** Gain: exact text. Cost: bilingual.
  2. **Translated with the original alongside.** Gain: readable. Cost: translation
     per quotation.
  3. **Translated only.** Gain: short. Cost: uncheckable.
- Chosen: **Option 1** because the quotation must be checkable against the book.
  Rejected: Option 2 for unmeasured accuracy cost; Option 3 because a
  translation error becomes an invented position.

### The classification is called Genre (Q2)

- Context: The comparison may be scoped by genre.
- Options: **WorkKind**, **DomainClass**, **Genre**.
- Chosen: **Genre**. Rejected: the alternatives as recorded in Q2.

## Open questions

None.
