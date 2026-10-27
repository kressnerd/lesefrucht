# Evidence search in one's own library

Topic `evidence-search`, ideation complete 2026-10-20. Handoff confirmed.

This document is the readable record of the ideation. The working artifacts live
in `givn/ideation/evidence-search/`: the frame, the event storming with its
canvas, the eight use cases, the question log and the change seeds.

Input to the ideation was two research documents supplied in conversation: an
architecture concept for a cited passage search over a calibre library, and a
language decision favouring Rust for the product. Neither is a project record;
the constraints they carry are reproduced here as given.

## The problem

A person with a mixed calibre library of a few thousand books — PDF, EPUB and
MOBI, German and English — cannot substantiate a claim from their own books.
They cannot find the passage across book and language boundaries, and a passage
they do find cannot be traced back to an exact position in a specific edition.

## The vision

The library becomes a citable source. A conceptual question in one language is
answered by a response whose every sentence carries a passage from the books,
with a citation the viewer opens. The reader stops trusting the model's memory
and starts checking the book.

## What the product owns

| Owned | Not owned |
|---|---|
| Finding, re-reading and citing passages | The reader's insight; leading the conversation |
| Answering a question from the library, every sentence cited | Assertions from model knowledge without evidence |
| Bounded synthesis over supplied text only | Free text production without a library basis |
| Classifying a work and proposing metadata, with human confirmation | Catalog authority; calibre remains binding |
| Abstaining when the library does not support the question | Guesses as a result |

The line is not "search versus answer" but "cited versus asserted text". An
answer the library does not support is refused, not smoothed.

## Non-goals of the first version

Reading progress, notes and highlights (assumption A2 is unconfirmed); a
mandatory container; the calibre plugin as the core; a web surface in the first
milestone; a replacement for calibre FTS.

## People

| Persona | Role | Authority | Success |
|---|---|---|---|
| reader--researcher | The owner reading to support an argument | Decides what counts as evidence | The citation opens the exact position and the quote holds |
| agent--for-reader | An LLM client acting for the reader | Decides tool choice and token budget | Composes a cited answer without guessing |
| custodian--metadata | The owner as catalog authority | Decides every metadata change | No manual decision is lost; the inbox stays small |
| custodian--glossary | The owner as vocabulary authority | Accepts terms per concept and language | The German question reaches the English passage |
| upstream--calibre | Maintainer of the calibre interface | Decides which interfaces stay supported | An upgrade does not break the product |
| rights--holder | Owner of the book text | Decides nothing; tolerates private use | Text leaves the host only to configured endpoints |

## Vocabulary

| Term | Meaning | Not |
|---|---|---|
| Work | A title of the library, identified by its calibre book id | Edition |
| Edition | One file of a work in exactly one format | Work |
| Passage | A contiguous text stretch the product indexes, retrieves and cites as one unit | Chapter |
| Citation | The exact position of a passage in an edition, which the viewer opens | Quotation |
| Genre | The classification of a work into `fachbuch`, `sachbuch`, `belletristik`, `sonstiges` | Domain |
| Question | The reader's need in one language, resolving to one response, one abstention or one unavailability report | Query |
| Answer | A prose response whose every sentence carries a citation | Summary |
| Abstention | The declared "the library does not support this", with a reason | Empty result |
| Glossary | Accepted terms per concept and language | The arc42 glossary |
| Index | The derived search structure over passages, rebuildable from editions | Library |
| Proposal | A metadata change awaiting the custodian's decision | Write |
| Provenance record | The value the product last wrote to one field | Change history |

## Aggregates and invariants

| Aggregate | Invariant |
|---|---|
| Work | At most one edition is selected; at most one open proposal per field |
| Edition | Exactly one index state; passages never cross chapter boundaries; an unchanged content hash is never re-embedded |
| Question | Exactly one response, one abstention or one unavailability report; every prose sentence cites a retrieved passage; no response below the evidence threshold |
| Glossary | At most one accepted term per concept and language; a proposed term has no effect until accepted |

Two guards live in policies, not aggregates: the format preference for the
selected edition, and the field-write guard that re-reads calibre before a
write and locks a field on a mismatch.

## The chain

A work appears in the library, one edition is selected and indexed into
passages. A question expands through accepted terms, retrieves and ranks
passages, and is answered, compared, summarized or listed as works — or it is
refused. Separately, indexed works are classified into a genre and their
metadata changes wait for the custodian's decision.

The full chain with its canvas is in
`givn/ideation/evidence-search/eventstorming.md`: 24 events, four aggregates,
twelve policies, thirteen read models.

## The use cases

| ID | Primary actor | Outcome |
|---|---|---|
| answer-question | Reader | A cited answer to a question |
| find-passages | Agent for the Reader | Candidate passages for a claim, each cited |
| find-works | Reader | Ranked works for a topic |
| compare-positions | Reader | A cited comparison over up to five named works |
| summarize-work | Reader | A cited summary of one work or chapter |
| classify-works | Custodian | The genre of each indexed work |
| review-metadata-proposals | Custodian | A decided metadata proposal |
| maintain-glossary | Custodian | One accepted term per concept and language |

Full contracts, rules, examples and guarantees are in
`givn/ideation/evidence-search/use-cases/`.

## Decisions

| ID | Decision | Chosen | Rejected |
|---|---|---|---|
| Q1 | Product boundary | The answer is a product goal | Evidence with bounded synthesis |
| Q2 | Name of the classification | Genre | WorkKind, DomainClass |
| Q3 | Acceptance of a cited answer | Per-sentence coverage check by a judge over recorded answers | Structural citation only, human sample only |
| Q4 | Classification write rule | Autonomous above a confidence threshold, proposals below it | Proposal only, autonomous first run |
| Q5 | Write safety with the desktop open | One persistent write queue with retry | Immediate re-read only, refuse while a desktop is detected |
| Q6 | Language of answer and quotations | Answer in the question's language, quotations in the source language | Translated quotations with original, translated only |
| Q7 | Glossary authority | Separate persona segment `custodian--glossary` | Extending the metadata custodian, the reader |
| Q8 | Classification | A custodian goal, restated | A subfunction of review |

The full records, with every option's gain and cost, are in
`givn/ideation/evidence-search/questions.md`.

## Open questions

None — every question is closed.

## What ships first

| Order | Seed | Guarantee |
|---|---|---|
| 1 | passage-retrieval | The agent gets cited candidate passages from the indexed library |
| 2 | cited-answer | The reader gets a cited answer, an abstention or an unavailability report |
| 3 | work-discovery | The reader gets ranked works for a topic |
| 4 | glossary-expansion | Accepted terms change retrieval deterministically |
| 5 | genre-classification | Every indexed work carries the custodian's genre |
| 6 | metadata-review | The custodian decides proposals without silent overwrite |
| 7 | comparison | A cited comparison over up to five named works |
| 8 | summary | A cited summary of one work or chapter |

Each seed is a change-sized slice with its own guarantee; the files are in
`givn/ideation/evidence-search/changes/`. Handoff does not start a change.
