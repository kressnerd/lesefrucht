# 12. Glossary

Domain and technical terms used in this project. Entries are alphabetically
ordered. Use the same terms consistently throughout all documentation.

## Ubiquitous language

| Term | Definition | Anti-terms | Code symbol |
|---|---|---|---|
| Abstention | The declared outcome that the library does not support a question, carrying its reason. | refusal, empty result | `Abstention` |
| Answer | The product's prose response to a question, in which every sentence carries at least one citation. | summary, reply | `Answer` |
| Citation | The exact position of a passage inside an edition, which the viewer opens. | quotation, reference | `Citation` |
| Edition | One file of a work in exactly one format, from which the product extracts text and cites positions. | format, file | `Edition` |
| Genre | The classification of a work into exactly one of `fachbuch`, `sachbuch`, `belletristik`, `sonstiges`. | Domäne, category | `Genre` |
| Glossary | The product's collection of accepted terms per concept and language, used to expand a question deterministically. | the arc42 glossary, lexicon | `Glossary` |
| Index | The derived search structure over passages, rebuildable from editions. | library, search store | `Index` |
| Library | The calibre collection whose metadata calibre holds and whose files the product reads. | index | `Library` |
| Passage | A contiguous stretch of an edition's text that the product indexes, retrieves and cites as one unit. | chapter, excerpt | `Passage` |
| Proposal | A metadata change prepared by the product and awaiting the custodian's decision. | write, suggestion | `Proposal` |
| Provenance record | The note of the value the product last wrote to one work field, guarding against silent overwrite. | history, audit log | `ProvenanceRecord` |
| Question | The reader's information need in one language with an optional scope, resolving to exactly one outcome: an answer, a comparison, a summary, a ranked-work result, a candidate-passage result, one abstention with its reason, or one unavailability report. | query, inquiry | `Question` |

## Aggregates and boundaries

| Aggregate | Root | Invariant | Commands | Events |
|---|---|---|---|---|
| Work | `Work` | At most one edition is selected; at most one open proposal per field. | RegisterWork, SelectEdition, ClassifyWork, ProposeMetadata, ApplyProposal, RejectProposal, DetectManualEdit, RetireWork | WorkDiscovered, EditionSelected, GenreProposed, MetadataProposed, MetadataApplied, MetadataRejected, FieldLocked, WorkRetired |
| Edition | `Edition` | Exactly one index state; passages never cross a chapter boundary; an unchanged content hash is never re-embedded. | IndexEdition, InvalidateIndex | TextExtracted, PassagesIndexed, IndexingFailed, IndexInvalidated |
| Question | `Question` | Exactly one outcome — a response of the requested shape (answer, comparison, summary, ranked works or candidate passages), one abstention with its reason, or one unavailability report; every prose sentence cites a retrieved passage; no result below the evidence threshold; a ranked-work result lists only indexed works. | AskQuestion, ExpandQuery, RetrievePassages, ComposeAnswer, RankWorks, Abstain, ReportUnavailable | QuestionAsked, QueryExpanded, PassagesRetrieved, AnswerComposed, ComparisonComposed, SummaryComposed, WorksRanked, AbstentionDeclared, UnavailabilityReported |
| Glossary | `Glossary` | At most one accepted term per concept and language; no term belongs to two concepts; a proposed term has no effect until accepted. | ProposeTerm, AcceptTerm, RejectTerm | TermProposed, TermAccepted, TermRejected |

## Invariants

| Invariant | Enforcement site |
|---|---|
| At most one selected edition per work | Work aggregate |
| At most one open proposal per field | Work aggregate |
| Passages never cross a chapter boundary | Edition indexing |
| An unchanged content hash is never re-embedded | Edition indexing |
| Every prose response sentence carries a citation | Question composition |
| No response below the evidence threshold | Question retrieval and composition |
| A Question yields exactly one outcome, distinguishable by the consumer | Question aggregate and the agent result contract |
| A field is written only while empty or equal to the provenance value | apply-or-propose policy and the persistent write queue |
| No accepted term belongs to two concepts in one language | Glossary aggregate |
| Book text leaves the host only to configured endpoints | outbound composition |

## Lifecycle events

- Work: discovered, edition selected, genre proposed, metadata proposed,
  applied, rejected, field locked, retired.
- Edition: text extracted, passages indexed, indexing failed, index invalidated.
- Question: asked, expanded, passages retrieved, answer composed, comparison
  composed, summary composed, works ranked, abstention declared, unavailability
  reported.
- Glossary: term proposed, accepted, rejected.

## Refinements

- 2026-10-27 — `Question`: the exactly-one outcome now names the
  candidate-passage result explicitly, because `find-passages` returns candidate
  passages rather than an answer, and the ranked-work result is restated from the
  confirmed event chain. The refinement keeps the same aggregate and the same
  term; no second term (such as "Query") is introduced. Source:
  `givn/ideation/evidence-search/changes/passage-retrieval.md`, recorded in
  `givn/changes/passage-retrieval/proposal.md`.
