# Event Storming — evidence-search

Status: revised 2026-10-20 after the use-case cut review. Terms confirmed per
`domain-terms.md`. No statement is qualified by an open question.

## Actors

| Actor | Issues | Reads |
|---|---|---|
| Reader | asks questions | answers, passages |
| Agent for the Reader | asks questions and reads passages on the reader's behalf | answers, passages, index status |
| Custodian | confirms or rejects proposals, maintains the glossary | review inbox, work detail, glossary editor |

Not actors — policies: library watcher, select-edition policy, indexing policy,
retry policy, chunk-and-embed policy, expansion policy, retrieval policy,
ranking policy, composition policy, classification policy, apply-or-propose
policy, provenance guard.

## Aggregates

| Aggregate | Invariant | Commands | Events |
|---|---|---|---|
| Work | at most one edition is selected; at most one open proposal per field | RegisterWork, SelectEdition, ClassifyWork, ProposeMetadata, ApplyProposal, RejectProposal, DetectManualEdit, RetireWork | WorkDiscovered, EditionSelected, GenreProposed, MetadataProposed, MetadataApplied, MetadataRejected, FieldLocked, WorkRetired |
| Edition | exactly one index state; passages never cross a chapter boundary; an unchanged content hash is never re-embedded | IndexEdition, InvalidateIndex | TextExtracted, PassagesIndexed, IndexingFailed, IndexInvalidated |
| Question | exactly one response of the requested shape — answer, comparison, summary or works — one abstention with its reason, or one unavailability report, never more than one; every prose response sentence cites a retrieved passage of an indexed edition; no response below the evidence threshold; a works listing ranks only indexed works of the library | AskQuestion, ExpandQuery, RetrievePassages, ComposeAnswer, RankWorks, Abstain, ReportUnavailable | QuestionAsked, QueryExpanded, PassagesRetrieved, AnswerComposed, ComparisonComposed, SummaryComposed, WorksRanked, AbstentionDeclared, UnavailabilityReported |
| Glossary | at most one accepted term per concept and language; a proposed term has no effect until accepted | ProposeTerm, AcceptTerm, RejectTerm | TermProposed, TermAccepted, TermRejected |

Guards that live in policies, not in the aggregates:

- The format preference for the selected edition (EPUB > AZW3/MOBI > PDF) is a
  configuration of the select-edition policy. Retrieval joins the selected
  edition with its index state.
- The field-write guard belongs to the apply-or-propose policy: it reads the
  calibre field fresh and writes only if the field is empty or equals the
  provenance value; on a mismatch the field is locked. Every write is enqueued and retried on conflict, so a
  desktop change between read and write is detected rather than overwritten.

## External systems

The calibre content server (catalogue and file source), the OpenAI-compatible
endpoint (expansion, composition, embeddings), and the viewer that opens a
`calibre://` citation. None of them is an aggregate; the product calls
documented contracts only.

## Event chain

| # | Event | Issuer | Command | Aggregate | Reacting policy | Read model |
|---|---|---|---|---|---|---|
| 1 | WorkDiscovered | library watcher | RegisterWork | Work | select-edition policy | library listing |
| 2 | EditionSelected | select-edition policy | SelectEdition | Work | indexing policy | work detail |
| 3 | TextExtracted | indexing policy | IndexEdition | Edition | chunk-and-embed policy | index status |
| 4 | PassagesIndexed | indexing policy | IndexEdition | Edition | classification policy | index status |
| 5 | IndexingFailed | indexing policy | IndexEdition | Edition | retry policy | index status |
| 6 | IndexInvalidated | library watcher | InvalidateIndex | Edition | indexing policy | index status |
| 7 | WorkRetired | library watcher | RetireWork | Work | purge derived passages (projection maintenance) | library listing |
| 8 | QuestionAsked | Reader, Agent for the Reader | AskQuestion | Question | expansion policy | — |
| 9 | QueryExpanded | expansion policy | ExpandQuery | Question | retrieval policy | expansion note |
| 10 | PassagesRetrieved | retrieval policy | RetrievePassages | Question | composition policy, ranking policy | ranked passages |
| 11 | AnswerComposed | composition policy | ComposeAnswer | Question | — | answer with citations |
| 12 | ComparisonComposed | composition policy | ComposeAnswer | Question | — | comparison with citations |
| 13 | SummaryComposed | composition policy | ComposeAnswer | Question | — | summary with citations |
| 14 | WorksRanked | ranking policy | RankWorks | Question | — | ranked works |
| 15 | AbstentionDeclared | composition policy | Abstain | Question | — | no-evidence result |
| 16 | UnavailabilityReported | retrieval policy, composition policy | ReportUnavailable | Question | — | unavailability report |
| 17 | GenreProposed | classification policy | ClassifyWork | Work | apply-or-propose policy | review inbox |
| 18 | MetadataProposed | classification policy | ProposeMetadata | Work | — | review inbox |
| 19 | MetadataApplied | Custodian | ApplyProposal | Work | provenance guard | work detail |
| 20 | MetadataRejected | Custodian | RejectProposal | Work | — | work detail |
| 21 | FieldLocked | provenance guard | DetectManualEdit | Work | — | review inbox |
| 22 | TermProposed | Custodian | ProposeTerm | Glossary | — | glossary editor |
| 23 | TermAccepted | Custodian | AcceptTerm | Glossary | expansion policy | glossary editor |
| 24 | TermRejected | Custodian | RejectTerm | Glossary | — | glossary editor |

The four response shapes share one aggregate and one composition command; the
shape is decided at `AskQuestion`. `ComparisonComposed` and `SummaryComposed`
are answers in the confirmed term's sense — cited prose responses — with a
different shape.

## Reverse narrative

Which events must have happened for `AnswerComposed` to be possible?
`PassagesRetrieved` ← `QueryExpanded` ← `QuestionAsked`, and
`PassagesIndexed` ← `TextExtracted` ← `EditionSelected` ← `WorkDiscovered`.
`TermAccepted` is not a prerequisite: deterministic expansion uses accepted
terms where they exist, and the expansion policy may call the model endpoint
beyond them.

## Canvas

Kinds are distinguishable by shape: circle = actor, parallelogram = command,
cylinder = aggregate, hexagon = event, flag = policy, rectangle = read model,
subroutine = record, stadium = external system.

```mermaid
flowchart LR
  A(("Actor"))
  C[/"Command"/]
  G[("Aggregate")]
  E{{"Event"}}
  P>"Policy"]
  R["Read model"]
  REC[["Record"]]
  X(["External system"])
  A --> C
  C --> G
  G --> E
  E --> P
  E --> R
  E --> REC
  X --> P
```

### Library intake and indexing

```mermaid
flowchart LR
  CS(["External: calibre content server"])
  W>"library watcher"]
  C1[/"RegisterWork"/]
  A1[("Work")]
  E1{{"WorkDiscovered"}}
  P1>"select-edition policy"]
  C2[/"SelectEdition"/]
  E2{{"EditionSelected"}}
  P2>"indexing policy"]
  C3[/"IndexEdition"/]
  P4>"retry policy"]
  C4[/"InvalidateIndex"/]
  A2[("Edition")]
  E3{{"TextExtracted"}}
  P3>"chunk-and-embed policy"]
  E4{{"PassagesIndexed"}}
  E5{{"IndexingFailed"}}
  E6{{"IndexInvalidated"}}
  R1["RM: library listing"]
  R2["RM: work detail"]
  R3["RM: index status"]

  CS --> W
  W --> C1 --> A1
  A1 --> E1
  E1 --> P1 --> C2 --> A1
  A1 --> E2
  E2 --> P2 --> C3 --> A2
  A2 --> E3 --> P3 --> E4
  A2 --> E5
  E5 --> P4 --> C3
  W --> C4 --> A2
  A2 --> E6 --> P2
  E1 --> R1
  E2 --> R2
  E3 --> R3
  E4 --> R3
  E5 --> R3
  E6 --> R3
```

### Question answering

```mermaid
flowchart LR
  Rd(("Reader"))
  Ag(("Agent for the Reader"))
  LLM(["External: OpenAI-compatible endpoint"])
  C1[/"AskQuestion"/]
  A1[("Question")]
  E1{{"QuestionAsked"}}
  P1>"expansion policy"]
  C2[/"ExpandQuery"/]
  E2{{"QueryExpanded"}}
  P2>"retrieval policy"]
  C3[/"RetrievePassages"/]
  E3{{"PassagesRetrieved"}}
  P3>"composition policy"]
  P4>"ranking policy"]
  C4[/"ComposeAnswer"/]
  C5[/"Abstain"/]
  C6[/"RankWorks"/]
  C7[/"ReportUnavailable"/]
  E4{{"AnswerComposed"}}
  E5{{"AbstentionDeclared"}}
  E6{{"ComparisonComposed"}}
  E7{{"SummaryComposed"}}
  E8{{"WorksRanked"}}
  E9{{"UnavailabilityReported"}}
  R1["RM: expansion note"]
  R2["RM: ranked passages"]
  R3["RM: answer with citations"]
  R4["RM: comparison with citations"]
  R5["RM: summary with citations"]
  R6["RM: no-evidence result"]
  R7["RM: ranked works"]
  R8["RM: unavailability report"]

  Rd --> C1
  Ag --> C1
  C1 --> A1
  A1 --> E1
  E1 --> P1
  LLM --> P1
  P1 --> C2 --> A1
  A1 --> E2
  E2 --> P2
  LLM --> P2
  P2 --> C3 --> A1
  A1 --> E3
  E3 --> P3
  E3 --> P4
  LLM --> P3
  P3 --> C4 --> A1
  P3 --> C5 --> A1
  P2 --> C7 --> A1
  P3 --> C7
  P4 --> C6 --> A1
  A1 --> E4 --> R3
  A1 --> E5 --> R6
  A1 --> E6 --> R4
  A1 --> E7 --> R5
  A1 --> E8 --> R7
  A1 --> E9 --> R8
  E2 --> R1
  E3 --> R2
```

### Classification, metadata and retirement

```mermaid
flowchart LR
  Cu(("Custodian"))
  W2>"library watcher"]
  E0{{"PassagesIndexed"}}
  P1>"classification policy"]
  C1[/"ClassifyWork"/]
  A1[("Work")]
  E1{{"GenreProposed"}}
  P2>"apply-or-propose policy"]
  C2[/"ProposeMetadata"/]
  C3[/"ApplyProposal"/]
  C4[/"RejectProposal"/]
  E2{{"MetadataProposed"}}
  E3{{"MetadataApplied"}}
  E4{{"MetadataRejected"}}
  P3>"provenance guard"]
  C5[/"DetectManualEdit"/]
  E5{{"FieldLocked"}}
  C6[/"RetireWork"/]
  E6{{"WorkRetired"}}
  PR[["Record: provenance"]]
  R1["RM: review inbox"]
  R2["RM: work detail"]
  R3["RM: library listing"]

  E0 --> P1
  Cu --> P1
  P1 --> C1 --> A1
  A1 --> E1
  E1 --> P2
  E1 --> R1
  P2 --> C2 --> A1
  P2 --> C3 --> A1
  Cu --> C3
  Cu --> C4 --> A1
  A1 --> E2 --> R1
  A1 --> E3 --> PR
  E3 --> R2
  A1 --> E4 --> R2
  W2 --> P3 --> C5 --> A1
  A1 --> E5 --> R1
  W2 --> C6 --> A1
  A1 --> E6 --> R3
```

### Glossary

```mermaid
flowchart LR
  Cu(("Custodian"))
  C1[/"ProposeTerm"/]
  A1[("Glossary")]
  E1{{"TermProposed"}}
  C2[/"AcceptTerm"/]
  C3[/"RejectTerm"/]
  E2{{"TermAccepted"}}
  E3{{"TermRejected"}}
  R1["RM: glossary editor"]

  Cu --> C1 --> A1
  A1 --> E1 --> R1
  Cu --> C2 --> A1
  Cu --> C3 --> A1
  A1 --> E2 --> R1
  A1 --> E3 --> R1
```

## Read models

Library listing, work detail, index status, review inbox, glossary editor,
ranked passages, answer with citations, comparison with citations, summary with
citations, ranked works, no-evidence result, unavailability report, expansion
note.

## Open

None — every question is closed.
