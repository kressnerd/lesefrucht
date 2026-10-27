# 12. Glossary

Domain and technical terms used in this project. Entries are alphabetically
ordered. Use the same terms consistently throughout all documentation.

## Ubiquitous language

| Term | Definition | Anti-terms | Code symbol |
|---|---|---|---|
| Abstention | The declared outcome that the library does not support a question, carrying its reason — the observable form of `no evidence`, either `no evidence` or `unknown work`. Abstention, not unavailability: the library was checked and has nothing. | empty result; refusal (the proposal's prose synonym for this outcome, never a result kind) | `NoEvidence`, `AbstentionReason` |
| Answer | The product's prose response to a question, in which every sentence carries at least one citation. | summary, reply | `Answer` |
| Candidate-passage result | The outcome that carries the passages retrieved for a question — each with citation, score, expansion and text — together with the requested and returned counts and the `short` flag. A candidate-passage result, not "candidate passages": the passages are its contents, the result is the outcome. | candidate passages (as an outcome noun), hits, results list | `CandidatePassages`; the JSON kind `candidate_passages` is a wire name only |
| Chapter | A contiguous structural section of an edition's text: the parent within which passages are chunked, widened and cited. A passage never crosses a chapter; a chapter is structure, not a citable unit. | section, part (as an equivalent of chapter) | `Chapter`, `ChapterRef` |
| Claim | The text the agent submits in one language — a claim or a search phrase — from which the expansion and the retrieval start. A claim, not a question: the question is the whole information need, including its scope and its outcome. | query, prompt | `claim` (request field) |
| Citation | The exact position of a passage inside an edition: work, edition, chapter, passage index and the character start and end offsets. The offsets are authoritative and the viewer link opens the edition; the link carries no position. | quotation, reference, page number | `Citation` |
| Edition | One file of a work in exactly one format, from which the product extracts text and cites positions. Edition, not work: several editions can carry the same text. | format, file | `Edition`, `EditionId` |
| Expansion | The deterministic widening of a claim into retrieval terms: the claim tokens plus accepted glossary terms and configured synonyms. Expansion, not translation: it adds retrieval terms and never rewrites the claim. | translation, query rewriting | `Expansion`, `expand` |
| Genre | The classification of a work into exactly one of `fachbuch`, `sachbuch`, `belletristik`, `sonstiges`. | Domäne, category | `Genre` |
| Glossary | The product's collection of accepted terms per concept and language, used to expand a question deterministically. | the arc42 glossary, lexicon | `Glossary` |
| Index | The derived search structure over passages, rebuildable from editions. Index, not library: the library is the authority and the index is derived. | library, search store, chunk index | `IndexStore` |
| Library | The calibre collection whose metadata calibre holds and whose files the product reads. | index | `Library` |
| Passage | A contiguous stretch of an edition's text that the product indexes, retrieves and cites as one unit, never crossing a chapter boundary. | excerpt; chapter (a chapter is the structural parent, not a passage) | `Passage` |
| Passage index | The ordinal of a passage within its edition, stable because it derives from chapter order and chunk order. Passage index, not character position: the offsets locate the text, the index orders passages. | chunk index | `passage_index` |
| Proposal | A metadata change prepared by the product and awaiting the custodian's decision. | write, suggestion | `Proposal` |
| Provenance record | The note of the value the product last wrote to one work field, guarding against silent overwrite. | history, audit log | `ProvenanceRecord` |
| Question | The reader's information need in one language with an optional scope, resolving to exactly one outcome: an answer, a comparison, a summary, a ranked-work result, a candidate-passage result, one abstention with its reason, or one unavailability report. | query, inquiry | `Question` |
| Short result | The declared state of a candidate-passage result whose returned number is below the requested number, with nothing padded to close the gap. Short, not empty: an empty result is an abstention. | truncated, incomplete | `short` (result field) |
| Unavailability report | The declared outcome that the system could not check the library or its index, carrying the reason `index unavailable`, `stale index` or `library unreachable`. An unavailability report, not an abstention: it reports a capability the system lacks right now, not a missing passage. | error, failure, empty result | `Unavailable`, `UnavailabilityReason` |
| Work | One book in the library, independent of format: a work owns one or more editions and at most one selected edition. Work, not edition: an edition is one file of the work in one format. | book (when it means the format-bearing file), title | `Work` |
| Work id | The numeric calibre book id that identifies a work, carried as a string and ordered numerically — `"9"` sorts before `"10"`, never byte-wise. Work id, not title: titles change and are not unique. | book number, slug | `WorkId` |

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
| Only the selected edition's passages are retrievable; a de-selected edition's passages are removed in the same commit | Edition indexing and `IndexStore::commit_edition` |
| An unchanged content hash is never re-embedded | Edition indexing |
| A citation's character offsets locate the passage exactly; the viewer link carries no position | citation data and `read_passage` |
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
- 2026-10-27 — `Citation`: the character offsets are the authoritative position
  and `viewer_url` opens the edition without a position; the position-aware
  viewer opening (`calibre://view-book/...?open_at=...`) is deferred as TD-009.
  Source: design-review decision D3, question Q3, recorded in
  `givn/changes/passage-retrieval/design-review.md` and
  `givn/changes/passage-retrieval/design.md` (Data Model › Domain types).
