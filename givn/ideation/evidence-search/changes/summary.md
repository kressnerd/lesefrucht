# Change seed: summary

Status: ready

- Use-case IDs: summarize-work
- Topic: evidence-search
- Personas: reader--researcher, agent--for-reader, rights--holder
- Suggested start: 8 — depends on passage-retrieval and cited-answer
- Handoff: handoff confirmed 2026-10-20; ready for `/givn-propose`

## Accepted decisions

### The product owns the cited answer (Q1)

- Context: A summary is prose and must be governed by the same boundary as an
  answer, limited to one work.
- Options: **evidence with bounded synthesis**, **the answer as a product goal**.
- Chosen: **Option 2**, with the summary limited to the named work or chapter.
  Rejected: Option 1 because the agent would have to compose it.

### Answer in the question's language, quotations in the source language (Q6)

- Context: A summary of an English work may be requested in German.
- Options:
  1. **Quotations in their source language.** Gain: exact text. Cost: bilingual.
  2. **Translated with the original alongside.** Gain: readable. Cost: a
     translation per quotation.
  3. **Translated only.** Gain: short. Cost: uncheckable.
- Chosen: **Option 1** because the quotation must be checkable against the book.
  Rejected: Option 2 for unmeasured accuracy cost; Option 3 because a
  translation error becomes an invented citation.

### The classification is called Genre (Q2)

- Context: Works may be selected by genre before summarizing.
- Options: **WorkKind**, **DomainClass**, **Genre**.
- Chosen: **Genre**. Rejected: the alternatives as recorded in Q2.

## Open questions

None.
