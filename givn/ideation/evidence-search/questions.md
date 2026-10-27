# Open questions — evidence-search

## Q1 — Product boundary: evidence or answer

- Status: closed 2026-10-20 (user) — Option B
- Blocks: the Frame (goal, scope, stakeholders) and therefore the cut of every
  use case.
- Context: The supplied documents describe both cited passage search and
  synthesis tools (`summarize`, `compare_positions`). The opening of this
  session spoke of "insights from the library". That left open whether an
  answer to a question is a product goal with its own acceptance or the agent's
  task over supplied evidence. The decision determines whether the product must
  guarantee answer quality and hallucination containment at answer level, or
  only evidence fidelity.
- Question: Where is the product boundary — does lesefrucht deliver evidence and
  bounded synthesis over supplied evidence only, or does it deliver the answer
  itself?
- Options:
  1. **Evidence with bounded synthesis (frame draft).**
     In practice: the agent calls `search_passages` for "how does trust form in
     organisations", receives three passages with a `calibre://` link, reads one
     with `read_passage`, and writes the answer itself; `summarize` returns only
     text drawn from the supplied passages, with a citation per sentence.
     Gain: sharp boundary; every use case stays small; abstention is the normal
     case; acceptance measures evidence fidelity, not prose quality.
     Cost: without an agent the product returns passages, not prose; the web
     surface answers nothing.
  2. **The answer as a product goal.**
     In practice: the reader types the question into the web surface and gets a
     cited answer without an MCP client; the product owns completeness and
     wording.
     Gain: usable without an agent; one place for the answer.
     Cost: answer quality and hallucination containment become product
     responsibility; acceptance needs answer metrics in addition to AK-1/2/3;
     the web surface moves from M3 into the core; the use-case cut grows a
     synthesis aggregate.
- Resolution: **The answer as a product goal (Option 2)** because the reader
  must be able to use the library without an MCP client, and the cited answer is
  where the benefit becomes visible; the boundary then runs between cited and
  asserted text, not between search and answer. Rejected: evidence with bounded
  synthesis (Option 1) because the reader would have to assemble the answer
  from passages and would always need an agent for it; the web surface would
  stay a passage list.

## Q2 — Name of the library classification

- Status: closed 2026-10-20 (user) — Option 3, Genre
- Blocks: the Event Storming canvas and every use case that names how a work is
  classified.
- Context: The supplied research calls the classification of a work — value set
  `fachbuch`, `sachbuch`, `belletristik`, `sonstiges` — "Domäne" and stores it
  as the calibre tag `domain.<class>`. In this project "domain" already means
  the problem area (the DDD cut, the arc42 glossary). One word with two meanings
  breaks the single vocabulary. The tag namespace is set nowhere yet, because no
  code exists, so a rename costs no migration.
- Question: What is the classification of a work into `fachbuch` / `sachbuch` /
  `belletristik` / `sonstiges` called?
- Options:
  1. **WorkKind** — a work is classified by the kind of its content.
     In practice: the agent calls `search_passages` with `workkind=fachbuch`;
     the calibre tag is `workkind.fachbuch`; the glossary has one entry;
     "domain" stays reserved for the problem area.
     Gain: one unambiguous term in tag, tool parameter and use case.
     Cost: deviates from the supplied document; its `domain.` prefix must be
     replaced everywhere.
  2. **DomainClass** — keep the supplied name; the product uses "domain" for both.
     In practice: `domain=fachbuch` in the tool and `domain.fachbuch` in the
     tag; the glossary carries two entries, "domain (problem area)" and "domain
     (work)".
     Gain: no deviation from the supplied document, no rename.
     Cost: every conversation needs a disambiguation; use cases and glossary
     carry the ambiguity permanently.
  3. **Genre** — the literary-studies term for the kind of work.
     In practice: `genre=fachbuch`, tag `genre.fachbuch`.
     Gain: unambiguous against "domain"; familiar in English.
     Cost: genre conventionally means epic/lyric/drama, not textbook/fiction; the
     name promises a finer split than the value set delivers.
- Resolution: **Genre (Option 3)** because the term is unambiguous against the
  project's use of "domain" and the user accepted the narrower literary
  connotation over a permanent disambiguation. Rejected: WorkKind because the
  user preferred the established English word to a coined compound despite its
  deviation from the supplied document; DomainClass because it keeps two
  meanings of "domain" in the glossary forever.

## Q3 — Acceptance criterion for a cited answer

- Status: closed 2026-10-20 (user) — Option 1, per-sentence judge
- Blocks: the success guarantee of the answer use case and the M1 acceptance run.
- Context: AK-1 to AK-12 measure recall, citation accuracy, classification,
  latency, abstention and operation. With Q1 Option B the product delivers
  answers itself; no supplied criterion checks whether an answer is supported by
  the passages it carries. Without a check, the central new guarantee is
  unverified.
- Question: How is it measured that an answer is cited rather than asserted?
- Options:
  1. **Per-sentence coverage check by a judge.**
     In practice: 50 answers are checked sentence by sentence against the
     supplied passages by a judge (human or model); the target is a covered-
     sentence share of at least 0.95 and no uncovered sentence without a
     citation mark in the sample.
     Gain: measures exactly the new guarantee, including wrongly attached
     citations.
     Cost: judge calibration, running cost, subjectivity at the margins.
  2. **Structural citation requirement.**
     In practice: an answer is delivered only if every sentence carries a
     citation identifier; five sentences with five identifiers pass, one
     sentence without an identifier fails the answer.
     Gain: deterministic, cheap, checkable in every test run.
     Cost: a wrongly attached citation passes; substantive coverage stays
     unmeasured.
  3. **Human sample only.**
     In practice: 20 answers are read manually; the target is zero invented
     statements.
     Gain: no judge setup, competent at the margins.
     Cost: no regression capability, not in CI, does not scale with prompt or
     model changes.
- Resolution: **Per-sentence coverage check by a judge (Option 1)** because it
  measures the guarantee itself — including citations that are attached but
  wrong — and extends to new response shapes later. The judge sees only the
  question, the supplied passages and the answer, never the library; trust comes
  from a one-time human-labelled sample of about 100 sentences. Runs are offline
  on recorded artifacts: fixture recordings in CI, real-library recordings on
  the host. Rejected: the structural citation requirement because R1 already
  enforces it at composition and it cannot detect a wrongly attached citation;
  the human sample alone because it is not regression-capable and does not scale
  with prompt or model changes.

## Q4 — Autonomous first run into empty metadata fields

- Status: closed 2026-10-20 (user) — Option 3, autonomous above a confidence threshold
- Blocks: the classification policy in the event chain and the success
  guarantee of the classification use case.
- Context: The supplied research allows the first classification run to write a
  genre into empty metadata fields without confirmation ("Erstlauf autonom nur
  in leere Felder; danach Vorschlag + Bestätigung"). The confirmed Frame says
  metadata is proposed with human confirmation. Both cannot hold. The decision
  determines whether a work can enter the library with a genre the custodian
  never saw.
- Question: May the product write a genre into an empty field without the
  custodian's confirmation?
- Options:
  1. **Proposal only — every write waits for confirmation.**
     In practice: a new work arrives, the product proposes `fachbuch`, the
     review inbox shows it, and nothing changes in calibre until the custodian
     confirms.
     Gain: one rule, no invisible writes, matches the confirmed Frame.
     Cost: the inbox starts with one item per work; a three-thousand-book
     library starts with three thousand items.
  2. **Autonomous first run into empty fields, proposals afterwards.**
     In practice: the first classification run writes `fachbuch` into an empty
     tag field and records provenance; later runs propose when the value
     differs.
     Gain: no inbox flood; the library is classified after the first run.
     Cost: the custodian inherits thousands of unreviewed decisions; a wrong
     genre stays invisible until a search misleads.
  3. **Autonomous only above a confidence threshold, proposals below it.**
     In practice: a work with a confident `fachbuch` result is written; a work
     whose result sits near the threshold appears in the inbox.
     Gain: bounded exposure; the inbox holds the uncertain cases only.
     Cost: the threshold needs calibration against a labelled sample; two write
     paths instead of one.
- Resolution: **Autonomous above a confidence threshold (Option 3)** because it
  bounds the product's unreviewed writes to the cases where classification is
  reliable and keeps the uncertain cases with the custodian; the threshold is
  calibrated on a labelled sample and configured by the custodian. Rejected:
  proposal only because a three-thousand-book library would start with three
  thousand inbox items and no usable genre filter; the autonomous first run
  because the custodian would inherit thousands of decisions nobody made.

## Q5 — Write safety while the calibre desktop is open

- Status: closed 2026-10-20 (user) — Option 2, persistent write queue
- Blocks: the application precondition in the event chain and the
  "no silent overwrite" guarantee of the two changing use cases.
- Context: The product writes metadata through the content server while the
  desktop may hold the same library. AK-7 demands no corruption and no error;
  the supplied document proposes a persistent queue with retry. Between the
  fresh read that checks the current calibre value and the write that changes
  it, another writer can change the field; the product would then overwrite a
  value it never saw. The window cannot be closed without cooperation from
  calibre.
- Question: How is a metadata write protected against a concurrent change by
  the calibre desktop?
- Options:
  1. **Re-read immediately before the write, abort on mismatch.**
     In practice: the product reads the tag field, sees `fachbuch`, prepares
     `sachbuch`; if the desktop changed it to `roman` in between, the write is
     refused and the field is locked for review.
     Gain: no lock and no queue ownership; catches the common cases.
     Cost: the read-to-write window stays open; a change inside it is lost.
  2. **Serialize every product write through one persistent queue.**
     In practice: each proposal application is queued, retried on conflict, and
     the queue survives a restart; the desktop's writes remain unobserved.
     Gain: deterministic order inside the product; matches AK-7 and the
     supplied document.
     Cost: the desktop can still write concurrently, so this orders only the
     product's own writes.
  3. **Refuse writes while a desktop session is detected.**
     In practice: the apply command returns "library busy" and the proposal
     stays pending until the desktop closes.
     Gain: the strongest protection.
     Cost: needs a reliable detection signal that calibre may not offer; the
     owner cannot apply a proposal while reading in the desktop.
- Resolution: **Serialize every product write through one persistent queue
  (Option 2)** because AK-7 requires surviving a concurrent desktop without
  corruption or error, and a queue with retry is the only option that also
  survives a restart; the expected-value check before the write still catches a
  desktop change, and a conflict re-bases the proposal instead of overwriting.
  Rejected: the immediate re-read alone because the read-to-write window stays
  open and a failure has no retry; refusing while a desktop is detected because
  it needs a signal calibre may not offer and blocks writes while reading.

## Q6 — Language of an answer and its quotations

- Status: closed 2026-10-20 (user) — Option 1
- Blocks: rule R8 of `answer-question` and the corresponding rules of the
  synthesis use cases.
- Context: The library is German and English and a question is asked in one
  language. The Frame promises cross-language search but does not say how the
  response reads: a German question may be answered from an English passage.
  The decision determines what the reader sees for every quotation and whether
  a translation becomes part of the evidence.
- Question: In which language does the answer appear, and in which language do
  its quotations appear?
- Options:
  1. **Answer in the question's language, quotations in their source language.**
     In practice: a German question about trust produces a German answer; the
     English passage appears in English with its citation beside it.
     Gain: the quotation is exactly the text the reader finds in the book; no
     translation error can enter the evidence.
     Cost: a bilingual answer; the reader must read the source language.
  2. **Answer and quotations in the question's language, the source alongside.**
     In practice: the English passage appears translated, the original directly
     below it, both under one citation.
     Gain: readable end to end, the original stays checkable.
     Cost: every quotation is translated at composition time; translation cost
     and a second failure mode.
  3. **Answer in the question's language, quotations translated only.**
     In practice: the English passage appears only in German.
     Gain: the shortest answer.
     Cost: the reader cannot check the quotation against the book; a
     translation error becomes an invented citation.
- Resolution: **Quotations in their source language, answer in the question's
  language (Option 1)** because a translation would put a second failure mode
  between the reader and the evidence, and AK-3 measures citation accuracy
  against the edition. Rejected: Option 2 because every quotation would need a
  translation at composition time with an accuracy cost nothing measures;
  Option 3 because the reader could not check the quotation against the book,
  making a translation error indistinguishable from an invented citation.

## Q7 — Who owns the glossary

- Status: closed 2026-10-20 (user) — Option 2, separate persona segment
- Blocks: the persona of `maintain-glossary` and the authority to accept a term.
- Context: the confirmed personas give the custodian authority over calibre
  metadata; the glossary is a separate product store of concepts and terms per
  language. No persona's authority boundary covers it, yet the glossary use case
  needs one. Extending an authority boundary changes a persona the user already
  confirmed.
- Question: Whose authority is it to accept or reject a glossary term?
- Options:
  1. **Extend the custodian's authority to the glossary.**
     In practice: the custodian accepts "Vertrauen → trust" and rejects
     "confidence"; one person owns the descriptive apparatus.
     Gain: no sixth persona; one owner for metadata and vocabulary.
     Cost: the custodian's stable core changes from "calibre metadata" to
     "metadata and glossary", which needs a confirmed persona edit.
  2. **A separate persona segment, `custodian--glossary`.**
     In practice: the same person reviews glossary terms through a different
     lens than metadata proposals.
     Gain: a precise authority boundary and a separate review lens.
     Cost: a sixth persona for the same human; more review passes per artifact.
  3. **The reader owns the glossary.**
     In practice: the reader adds terms while reading, and the agent proposes
     them.
     Gain: vocabulary grows where it is used.
     Cost: the reader persona currently decides nothing about the library's
     apparatus; expanding it changes a confirmed persona.
- Resolution: **A separate persona segment, `custodian--glossary` (Option 2)**
  because the glossary is a different authority object than the calibre
  catalog, and the distinguishing criterion is the authority boundary; the
  segment keeps the review lens precise without editing a confirmed stable
  core. Rejected: extending the custodian's authority because it would blur two
  decision objects into one lens; the reader as owner because it would change a
  confirmed persona and move catalog-adjacent authority to a role that decides
  nothing about the library's apparatus.

## Q8 — Is classification a goal of the custodian or a subfunction

- Status: closed 2026-10-20 (user) — Option 1, keep it restated
- Blocks: the final shape of the cut — eight goals or seven — and the contract
  that owns the classification acceptance (AK-4).
- Context: the chain runs classification from the classification policy after
  `PassagesIndexed`; the custodian's only commands are `ApplyProposal` and
  `RejectProposal`. The cut lists `classify-works` as a custodian goal, but the
  goal joins the product's classification with the custodian's acceptance, and
  under Q4 Option 2 the product writes without that acceptance. The cut review
  therefore calls the row a policy flow presented as a goal.
- Question: Should `classify-works` stay a use case, or become a subfunction of
  `review-metadata-proposals`?
- Options:
  1. **Keep it, restated.**
     In practice: a new work arrives, the product proposes `fachbuch`, the
     custodian confirms it in the review inbox; the goal is that every indexed
     work carries the custodian's genre.
     Gain: classification keeps its own contract and its AK-4 acceptance; the
     cut stays at eight.
     Cost: it describes the same write path as `review-metadata-proposals` for
     the genre field; two contracts must not contradict each other.
  2. **Demote it to a subfunction.**
     In practice: a new work arrives, the product proposes `fachbuch`, and the
     custodian decides in the review case; there is no separate classification
     goal.
     Gain: one write path, one contract; Q4 no longer changes the cut's shape.
     Cost: classification rules and AK-4 live inside the review use case; a
     change to classification reopens that contract.
- Resolution: **Keep it as a goal, restated (Option 1)** because classification
  owns the AK-4 accuracy acceptance and a change to it should reopen one
  contract, not the review contract; the overlap with review is bounded by the
  Boundaries table. Rejected: demoting to a subfunction because AK-4 and the
  classification rules would live inside an unrelated review contract, and
  every classification change would reopen it.
