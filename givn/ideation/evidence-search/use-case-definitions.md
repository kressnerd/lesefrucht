# Use Case Definition — evidence-search

Status: revised 2026-10-20 after the phase review; all eight rows settled.

## Cut

| ID | Level | Goal — the primary actor's observable outcome | Primary actor | Mutation |
|---|---|---|---|---|
| answer-question | ! | The reader holds a cited answer to a question about the library. | Reader | observing |
| find-passages | ! | The agent holds candidate passages for a claim, each with a citation. | Agent for the Reader | observing |
| find-works | ! | The reader holds a ranked list of the works that treat a topic. | Reader | observing |
| compare-positions | ! | The reader holds a cited comparison of how up to five named works treat one question. | Reader | observing |
| summarize-work | ! | The reader holds a cited summary of one work or one chapter. | Reader | observing |
| classify-works | ! | The custodian determines the genre of each indexed work, directly or by threshold rule. | Custodian | changing |
| review-metadata-proposals | ! | The custodian decides a pending metadata proposal. | Custodian | changing |
| maintain-glossary | ! | The custodian keeps one accepted term per concept and language. | Custodian | changing |

`observing` means the case changes neither the library nor its metadata. The
Question record and the answer cache are session state, not library state.
Totals: five observing, three changing.

## Boundaries

Every excluded actor goal carries its use-case ID.

| ID | Does not |
|---|---|
| answer-question | return passages without prose (find-passages); list works (find-works); compare named works (compare-positions); summarize one work (summarize-work); decide proposals (review-metadata-proposals) |
| find-passages | answer the question (answer-question); rank works (find-works); compare named works (compare-positions) |
| find-works | answer the question (answer-question); compare named works (compare-positions) |
| compare-positions | choose the works; answer the question (answer-question); summarize one work (summarize-work) |
| summarize-work | compare works (compare-positions); answer the question (answer-question); list works (find-works) |
| classify-works | decide other metadata fields (review-metadata-proposals); accept glossary terms (maintain-glossary) |
| review-metadata-proposals | classify works (classify-works); accept glossary terms (maintain-glossary) |
| maintain-glossary | classify works (classify-works); decide metadata proposals (review-metadata-proposals) |

Caps and output details — five works, chapter hint, genre field — are rules of
the elaborated case, not boundaries.

## Subfunctions (-)

- index-work — extract, chunk, embed, track index state; runs inside the
  observing flows, no actor goal of its own.
- retrieve-evidence — expand, retrieve and rank passages; inside every observing flow.
- read-around-passage — widen context around a cited passage; inside
  answer-question, find-passages and compare-positions.
- resolve-citation — open the viewer at the exact position; inside every
  observing flow.
- report-status — index freshness and failure list; read by the Agent for the
  Reader.
- purge-derived-passages — remove passages of retired works from the index;
  triggered by the library watcher, no actor goal.

## Personas

| ID | Personas |
|---|---|
| answer-question | reader--researcher, agent--for-reader, rights--holder |
| find-passages | agent--for-reader, reader--researcher |
| find-works | reader--researcher, agent--for-reader |
| compare-positions | reader--researcher, agent--for-reader, rights--holder |
| summarize-work | reader--researcher, agent--for-reader, rights--holder |
| classify-works | custodian--metadata, upstream--calibre |
| review-metadata-proposals | custodian--metadata, upstream--calibre |
| maintain-glossary | custodian--glossary, agent--for-reader, reader--researcher |

`FieldLocked` (chain row 21) is resolved in review-metadata-proposals: a manual
edit that differs from the provenance record locks the field and opens a review
item.

## Open

None — every question is closed.

