# Design review — passage-retrieval

Change: `passage-retrieval` (use case `find-passages`, capability
`passage-retrieval`, `NEW in find-passages`). Date: 2026-10-27. Operator: the
project operator (owner of the library and this repository).

Process: the plan (proposal, feature, use case, `design.md`, arc42 chapters,
glossary, README, commands) was interrogated by a fresh-context grilling
subagent that did not write it. Three questions could not be settled from the
repository and were put to the operator one at a time. Every other branch was
settled by exploring the repository or the upstream sources and is recorded
below as a finding with its disposition.

## Operator questions and decisions

### Q1 — Must the agent be able to see per-edition indexing failures through the MCP contract?

**Context.** The proposal promises that an edition in a format the product
cannot read is "reported as failed indexing with a reason … so the agent can
tell a missing passage from a missing capability" (`proposal.md`,
§Proposed Solution). The design put the failures only behind `/healthz` — an
operational probe, not an agent surface — while the MCP contract exposed two
tools only, and arc42 chapter 6 claimed the agent could see them. Nothing in the
feature, the use case or the interaction inventory made the promise observable,
so the promise was unimplementable as written. It blocked the scenario set: a
missing capability is exactly the case where an abstention must not look like
"the library has nothing".

**Options.**

- **A — add a third MCP tool `library_status`.**
  *Means:* a new tool returns `indexed_editions`, `failed_editions` with
  reasons, `last_success_at` and `library_generation`.
  *Gain:* the promise is kept explicitly and the agent can inspect status on
  demand at any time.
  *Cost:* a third consumer action, a third inventory entry, a third `@e2e`
  scenario, and an agent-visible surface that must be kept honest and cheap.
  *Example:* the agent calls `library_status`, sees
  `work-5 mobi: failed(unsupported_format)`, then knows the gap before it asks.
- **B — carry an optional `indexing` summary in the `no_evidence` result.**
  *Means:* an abstention carries the editions that failed indexing and their
  reasons when they are relevant to the request.
  *Gain:* no new consumer action, no new tool, no new `@e2e` scenario; the
  explanation arrives exactly when it matters — with the abstention.
  *Cost:* a larger outcome payload, and the agent cannot enumerate failures
  outside a retrieval.
  *Example:* `{"kind":"no_evidence","reason":"no_evidence","indexing":{
  "failed":[{"work_id":"5","reason":"unsupported_format"}]}}`.
- **C — amend the proposal and use case to make indexing status
  operational-only (`/healthz`).**
  *Means:* the promise is withdrawn; the agent never learns that an edition
  failed.
  *Gain:* the smallest contract: two tools, no payload growth.
  *Cost:* a skipped MOBI edition is indistinguishable from a genuine
  no-evidence result, which is the exact confusion the proposal wanted to end.
  *Example:* the agent receives `no_evidence` and cannot know that work-5 was
  never readable.

**Resolution (2026-10-27, project operator): option B.** The `no_evidence`
outcome carries an optional `indexing` summary naming the editions that failed
indexing and their reasons, so an abstention distinguishes a missing passage
from a missing capability. No `library_status` tool is added; the MCP contract
stays at two tools.

**Rejected.** A — rejected because it adds a consumer action and an `@e2e`
scenario for information the retrieval already knows at the moment it abstains.
C — rejected because it withdraws the proposal's sixth promise and leaves the
reader's agent unable to explain an empty answer.

**Applied:** `design.md` (Outcome types, MCP Contract examples, Indexing
Subfunction step 7, decisions ledger T9); feature scenario "An abstention names
the editions that failed indexing"; `usecase.md` (Result shapes); `proposal.md`
clarification; arc42 chapters 4, 6, 8, 10 (QS-012), 11.

### Q2 — Is the internal `/ajax/books` route an acceptable "documented contract" for library discovery?

**Context.** The design discovered the library with `GET /ajax/books` on the
content server and read bytes with `GET /get/<FMT>/<id>`, and arc42 chapter 2
called this "the documented content-server HTTP interface". `/ajax/books` is
implemented in calibre's `srv/ajax.py`; it is an internal AJAX route, not a
documented API, and the e2e calibration used a twin that would keep passing if
real calibre changed the route. The frame and the confirmed Persona
`upstream--calibre` both require documented interfaces only
(`givn/ideations/evidence-search/frame.md`; `givn/personas/upstream--calibre.md`),
so discovery had to move or the claim had to be withdrawn.

**Options.**

- **A — use the documented OPDS catalogue.**
  *Means:* fetch `/opds`/`/opds/books` and map Atom entries to works and
  editions.
  *Gain:* a documented, versioned contract.
  *Cost:* a new Atom parser; the catalogue lists tags only inside a
  human-readable XHTML block, has no `dc:language`, paginates, and exposes no
  per-format modification time that the generation needs.
  *Example:* parse `/opds/books` entries into `Work` records and infer formats
  from acquisition links.
- **B — keep `/ajax/books` and add a recorded real-calibre contract test.**
  *Means:* capture a real server's JSON and replay it in a contract test.
  *Gain:* rich metadata and one parser.
  *Cost:* the contract is still internal, so a calibre release can break it,
  and the test pins one capture rather than a promise.
  *Example:* a recorded `books.json` fixture turns green while the live server
  renamed a field.
- **C — use the documented calibre CLI: `calibredb list --with-library
  <url-or-path> --for-machine --fields all` for discovery and `calibredb
  export …` for edition bytes.**
  *Means:* shell out to the installed `calibredb`, which works against a
  content-server URL or a local library path, and parse the machine output.
  *Gain:* a documented interface with the metadata the design needs
  (`languages`, `tags`, `pubdate`, `formats`, `last_modified`), one mechanism for
  listing and export, and a test suite that runs the real CLI instead of a
  twin.
  *Cost:* a subprocess boundary (timeout, temp directories, output parsing), and
  the author list arrives as one ` & `-joined string.
  *Example:* `calibredb list --with-library http://host:8080#calibre
  --for-machine --fields all` prints one JSON array of per-book objects;
  `calibredb export --with-library … --to-dir /tmp/run --single-dir --formats
  EPUB <id>` writes the bytes locally.

**Resolution (2026-10-27, project operator, decision D2).** The operator's
decision list labels the calibre-CLI option "A"; in this question's ordering it
is option C. Discovery uses
`calibredb list --with-library <url-or-path> --for-machine --fields all` and
edition bytes come from `calibredb export --with-library <url-or-path> --to-dir
<tmp> --single-dir --formats <FMT> --dont-save-cover --dont-write-opf
--dont-save-extra-files <id>` (`--single-dir` is the design refinement that
makes the exported filename discoverable). Every `/ajax/books` and `/get/...`
claim is removed from the design and arc42; no HTTP calibre client remains.

**Rejected.** A — rejected because OPDS exposes tags only inside a
human-readable XHTML block and has no structured language field, so the scope
and generation cannot rely on it. B — rejected because it keeps an internal
route and replaces a promise with a recording.

**Verified shape (calibre documentation and `cmd_list.py`/`cmd_export.py`,
2026-10-27):** `--for-machine` prints one JSON **array** of per-book objects,
not a `{book_ids, data}` object; `authors` is stringified with
`authors_to_string` (joined by calibre's ` & `); `formats`, `languages` and
`tags` stay arrays; `pubdate` and `last_modified` are ISO-8601 strings; `id` is
numeric; `--fields all` is required for the listed fields; remote `list` rejects
templates only, and remote `export` returns the bytes over RPC and writes them
locally. The generation input is `(work_id, format, last_modified)` taken from
that array.

**Applied:** `design.md` (Technology Decisions › Stack, Module structure,
Runtime flows, Data Model › Index state, Indexing Subfunction steps 1 and 3,
Configuration, E2E Infrastructure, Test infrastructure, Risk, Version
Freshness, decisions ledger T8); arc42 chapters 2, 3, 4, 5, 6, 7, 8, 9, 11, 12;
feature scenario "An unreachable library yields an unavailability report";
`usecase.md` extension 3c; `proposal.md` clarification; README prerequisites.

### Q3 — Does "the citation opens the exact position" include the human viewer, or only the agent's offsets?

**Context.** The confirmed Persona `reader--researcher` says "the citation must
open the exact position, not merely the book" (`givn/personas/reader--researcher.md`,
2026-10-20 stance), the use case promised it, and arc42 chapter 3 said the viewer
opens the exact position. The design's `viewer_url` was
`calibre://show-book/<library>/<work>`, which opens the edition and carries no
position. Either the promise or the design had to move.

**Options.**

- **A — scope this slice to the agent.** The citation's character offsets are
  authoritative and `read_passage` proves the span; the viewer link opens the
  edition.
  *Means:* the agent can always locate the exact span; the reader navigates to
  it by hand.
  *Gain:* honest, no viewer integration, no unverified URL scheme.
  *Cost:* the reader's one-click exact-position experience is not delivered in
  this slice.
  *Example:* the agent cites work 12, chapter 1, characters 1400–1900; the
  reader opens work 12 at the start of the chapter and scrolls.
- **B — build a position-aware deep link or viewer route now.**
  *Means:* e.g. `calibre://show-book/.../12#char=1400-1900` or a small viewer
  route.
  *Gain:* the Persona promise is met in one click.
  *Cost:* a new surface and format, calibration against calibre's viewer, and a
  scope increase beyond retrieval.
  *Example:* a citation click opens the edition scrolled to the cited span.
- **C — A now, with the position-aware opening tracked as technical debt.**
  *Means:* option A, plus a recorded TD with a concrete resolution condition.
  *Gain:* honest scope with a durable reminder.
  *Cost:* the same as A, plus documentation upkeep.
  *Example:* the design states offsets are authoritative and TD-009 names the
  condition under which `open_at` is added.

**Resolution (2026-10-27, project operator, decision D3).** The operator
decision list labels this choice "B"; its content is options A and C as
presented here: character offsets stay authoritative, `read_passage` proves the
span, `viewer_url` remains `calibre://show-book/<library>/<work>` and opens the
edition, and the position-aware viewer opening
(`calibre://view-book/...?open_at=...`) is recorded as tracked technical debt
with a concrete resolution condition.

**Rejected.** A position-aware deep link now — the scheme itself is documented
(`calibre://view-book/<library>/<id>/<FORMAT>?open_at=<location>`), but it needs
a second coordinate system from the product's normalized character offsets to
calibre's opaque viewer locator, and the desktop viewer cannot be driven
headlessly to verify that the span opens; that is a new surface outside this
slice. Withdrawing the Persona stance — rejected because the stance still
describes the reader-facing target, which the TD preserves.

**Applied:** `design.md` (Data Model › Domain types, MCP Contract ›
`read_passage`, decisions ledger T10); `proposal.md` release note (the citation
carries its exact position and opens its edition; the outcome is an abstention,
not a refusal); `docs/arc42/03-context-and-scope.md`
(viewer row and interface table); `docs/arc42/06-runtime-view.md`; glossary
`Citation` row plus refinement note; `usecase.md` Persona line; arc42 chapter 11
TD-009; `givn/personas/reader--researcher.md` (2026-10-27 running record and a
qualified 2026-10-27 stance; the 2026-10-20 stance is not deleted).

## Branch findings and dispositions

| Branch | Finding | Disposition |
|---|---|---|
| Scope | Proposal promise 6 (missing passage vs missing capability) had no capability, interaction, scenario or MCP surface. | Fixed by Q1 / decision D1: optional `indexing` summary on an abstention, scenario "An abstention names the editions that failed indexing", use-case Result shapes, proposal clarification. |
| Scope | Proposal promise 1 was covered only for genre and named works; language, authors and the year range had no scenario. | Fixed: scenarios "Scope filters passages by language" and "Scope filters passages by author and year". |
| Scope | Repeat determinism ("the same request … always yields the same sequence") had no scenario. | Fixed: scenario "Identical requests return the same sequence"; QS-002 aligned. |
| Scope | Citation exactness of a search result (a fabricated span inside an existing edition could pass) was untested. | Fixed: scenario "A returned citation resolves to the passage it cites"; the e2e scenario now asserts the cited slice equality. |
| Scope | `window_chars` clamping to the chapter and to `read.max_window_chars` had no scenario. | Fixed: scenario "A widened context is clamped to the chapter and the maximum window". |
| Scope | Malformed requests and unknown citations were design-only arms. | Fixed: scenarios "A malformed request is rejected" (`invalid_arguments`) and "An unknown citation is rejected as an invalid request" (`invalid_citation`). |
| Scope | `library_unreachable` and the empty-store `index_unavailable` arm existed only in the design. | Fixed: use-case extension 3c and scenarios "An unreachable library yields an unavailability report" and "An unavailable index yields an unavailability report"; precondition wording aligned. |
| Scope | Scenario 9 asserted internal "index state" and omitted the glossary the use case calls a minimal guarantee. | Fixed: scenario 9 now compares the library metadata listing, the accepted glossary, and the index generation plus `last_success_at` read through the readiness probe. |
| Tech choices | `/ajax/books` was not a documented contract. | Fixed by Q2 / decision D2 (calibredb). |
| Tech choices | Licence contradiction: arc42 chapter 2 said "unchanged/open" while README says MPL-2.0. | Fixed: chapter 2 states MPL-2.0 with the allowlist as the dependency policy. |
| Tech choices | README said "host without a container runtime" while local/e2e need podman. | Fixed: README qualifies podman for local/e2e tests and keeps production container-free. |
| Tech choices | The rustup/`libz.so.1` story was stale and offered two contradictory pin mechanisms. | Fixed: observed host state recorded, Nix devShell as the single pin, `rust-toolchain.toml` removed from the plan, R-010 rewritten. |
| Tech choices | Single-SQLite, sqlite-vec and other alternatives. | Not an issue: the rejected-alternatives table already rejects brute-force SQLite/FTS5 and the single-store option with reasons; no change. |
| Missing scenarios | Indexing failure visibility, repeat determinism, citation exactness, language/author/year scope, cross-language expansion, malformed request, invalid citation, window clamp, unreachable library. | Fixed (see Scope rows). Address the gap set explicitly rather than leaving it to tasks. |
| Missing scenarios | R2 (per-request isolation) and R5 (no support verdict) have no scenario. | Recorded: `design.md` §Retained non-e2e scenarios explains why neither is separately observable (no cross-request state; absence of a schema field, already implied by every passage assertion). No scenario invented. |
| Testability | `cucumber::cli::Cli`/`Cli::from_args` does not exist in cucumber 0.23; `run_and_exit` takes one `AsRef<Path>`; the design passed an array of globs. | Fixed: `Opts::<_,_,_>::parsed()` and `run_and_exit("givn")`, with the crate source locations recorded. |
| Testability | `--name` and `--tags` conflict in `src/cli.rs`, so both exact single-scenario commands were rejected by clap. | Fixed: both commands use `--name` only; the conflict is stated. |
| Testability | The strictness proof pointed at a feature outside the runner roots without naming `--input`. | Fixed: `--input tests/fixtures/strictness.feature`, the Basic parser's glob option, and a non-zero exit asserted. |
| Testability | "clippy fails an empty body" was false. | Fixed: `unimplemented!()`/`todo!()` plus a CI grep for empty step bodies, named in the strictness proof. |
| Testability | E2E scenario 1 could pass with zero passages; scenario 3 was vacuously true; scenario 2 asserted no concrete bounds; scenario 6 joined stale and unavailable. | Fixed in the feature file (non-empty/kind/`short` consistency, positive fachbuch assertion, explicit bounds and chapter equality, scenario split with concrete reasons). |
| Testability | Scenario 4 used the slug `"unbekanntes-werk"` although `WorkId` is a calibre book id. | Fixed: `"999"`, a well-formed id absent from the fixture library; non-numeric ids are defined as `invalid_arguments`. |
| E2E fidelity | Inventory normalization, real driver (`rmcp` Streamable HTTP client) and `structuredContent` primary assertions. | Not an issue: two normalized actions, two matching `@e2e` scenarios, no duplicate action coverage, readiness-only `/healthz`. |
| E2E fidelity | The e2e scenario's Given read like a pre-existing citation instead of a retrieved one. | Fixed: "the agent has retrieved a citation to an indexed edition"; the driving mechanism takes the citation from `search_passages`. |
| E2E fidelity | The cross-language claim was not asserted, and the expansion configuration was unnamed. | Fixed: the e2e scenario asserts an English citation and `expanded_terms` containing `trust`; the fixture's synonym map is the named `expansion.synonyms_file`. |
| E2E fidelity | The calibre twin was the only evidence for the calibre contract. | Fixed by decision D2: no calibre twin; the real `calibredb` runs against a fixture calibre library in both non-e2e and e2e. |
| Interaction coverage | Two inventory entries, two matrix rows, two `@e2e` titles. | Not an issue: verified against `usecase.md` §Interactions; `givn lint` reports no mapping finding. |
| Risk | The named risk was citation/identity drift (concrete, mitigated), but the most likely immediate failure — the runner — was unmitigated. | Fixed: runner risk R-014 added and the mitigation applied as design edits. |
| Risk | calibredb availability and version drift were unrecorded after the transport change. | Fixed: R-012 (CLI drift) and R-013 (binary absent) with mitigations. |
| Use case | Inconsistent preconditions (library reachable, one indexed edition) versus runtime arms. | Fixed: preconditions reworded and extensions 3b/3c added. |
| Use case | The success guarantee covered only candidate passages, not the widening of main-flow step 5. | Fixed: the guarantee now includes widening with exact bounds. |
| Use case | R3 said "chunk index". | Fixed: "passage index" in `usecase.md` and in the ideation use-case draft. |
| Use case | `Includes:` named three undefined subfunctions. | Fixed: `Includes: none.` |
| Use case | Persona line claimed a viewer opening at the exact position. | Fixed by decision D3 wording with TD-009. |
| Decision ledger | Proposal `D<n>` and seed Q1–Q8 resolution. | Not an issue: the proposal records no unresolved `D<n>` and every seed question keeps its seed reference. Ledger extended with T8–T10 for the three decisions. |
| Visual design contract | `Interface: none` for an MCP JSON endpoint. | Not an issue: the section exists with viewports and the opposite classification explicitly rejected; no screen is introduced. |
| Interface classification | MCP JSON-RPC over Streamable HTTP versus browser/terminal. | Not an issue: tested the opposite; the optional `content` text block is a protocol payload, not a rendered view. |
| ADR qualification | ADR-0002 bundled the transport decision with the SDK implementation decision; the "Required outbound endpoints" row named two destinations. | Fixed: ADR-0002 now records one decision (transport), the SDK choice is routed once to `design.md` with a cheaper-home sentence, and the routed table names exactly one destination per row. |
| ADR qualification | ADR-0001 gates and consequences. | Not an issue: all five mandatory gates `PASS`, MADR complete, consequences in chapter 11. |
| arc42 | Independent 12-row derivation. | Verified: this review derived Yes for chapters 1–12 before opening `arc42.md`; the marker says Yes for all twelve, so there is no omission. Both ADRs have MADR files; routed candidates have one home. |
| arc42 | Chapter 6 claimed the agent could read indexing failures; chapter 10 overclaimed which QS the delta asserts; chapter 2 licence/open; chapter 3 exact-position viewer; chapter 11 stale R-010. | Fixed in the chapter files (each listed in the hardening edits). |
| arc42 | Chapter file integrity, template placeholders, ASCII art. | Not an issue: all 12 files exist with substantive content; no box-drawing characters and no plus-sign/pipeline pseudo-diagrams were found. |
| Glossary | `Work`, `Chapter`, `Expansion`, `Short result`, `Unavailability report`, `Claim`, `Work id`, `Passage index` missing; `Chapter` wrongly an anti-term of `Passage`; `Abstention`/`Index` code symbols stale; `candidate passages` versus `candidate-passage result`; `refusal` anti-term versus the proposal heading. | Fixed: rows added, anti-terms corrected and qualified, code symbols corrected, numeric ordering and the `Citation` refinement recorded. |
| Deferrals | No mandatory check was deferred. Position-aware viewer opening is a scope decision with a tracked TD, not a deferred check; model expansion and AZW3/MOBI are product scope, already recorded as TD-008/TD-007. `@e2e` tags are intact on both scenarios. | Recorded. |
| Persona disposition | Only the two direct Personas were dispositioned. | Fixed: `design.md` §Persona dispositions records `rights--holder`, `custodian--metadata`, `custodian--glossary` and `upstream--calibre` with relevance and evidence. |

## Note on the missing use-case instruction

`givn instructions usecase` is not a known artifact in this binary (it exits
with "unknown artifact 'usecase'"). The use-case rules were therefore taken from
`givn instructions review`'s scope-rejection criteria, `givn instructions specs`
(the interaction inventory) and `givn instructions domain-modeling`. This is a
documented substitute, not a skipped check.

## Hardening edits applied

**Feature** (`givn/changes/passage-retrieval/specs/find-passages/passage-retrieval.feature`)
— e2e scenario 1 asserts the `candidate_passages` kind, a non-empty result with
at most five passages, `short == (returned < requested)`, at least one English
citation, `expanded_terms` containing `trust` and the deterministic order; e2e
scenario 2 starts from a retrieved citation and asserts the cited slice equality,
the bound inequalities and the bounds chapter; scenario 3 asserts at least one
`fachbuch` passage; scenario 6 is split into stale (`stale_index`) and
unavailable (`index_unavailable`); scenario 4 uses `"999"`; scenario 9 compares
the library listing, the accepted glossary and the generation/last-success probe;
nine new `@givn.added @wip` scenarios were added (repeat determinism, citation
resolution, window clamp, invalid citation, malformed request, language scope,
author/year scope, indexing-failure abstention, unreachable library). Every
scenario stays `@wip`; both `@e2e` tags remain on their original titles.

**Use case** (`givn/specs/find-passages/usecase.md`) — R3 says passage index;
preconditions aligned with extensions 3b/3c; extension 3c added; success
guarantee includes widening; Result shapes record the `indexing` summary;
Persona line uses the D3 wording with TD-009; `Includes: none.`; interactions
unchanged (two actions).

**Design** (`givn/changes/passage-retrieval/design.md`) — toolchain pin reduced
to one Nix devShell mechanism with the observed host state; the calibre HTTP
client removed and replaced by the `calibredb` library interface row, module
`library/calibredb.rs`, process-boundary edges and the export step; test
infrastructure switched to the real `calibredb` against an `xtask fixtures`
calibre library with fixed mtimes, no calibre twin, and the exact stale-index
controls; outcome types gained `NoEvidence.indexing`; `requested` defined as the
clamped effective limit with the `50 → 20` triple; numeric work-id ordering in
the sort key; uniqueness stated with the same-commit de-selection purge;
viewer semantics per D3 with TD-009; the cucumber runner rewritten against the
0.23 source (`Opts::parsed()`, single `"givn"` root, `--input`, `--name`/`--tags`
conflict, no clippy claim, CI grep); the SDK decision added as its own design
subsection; T8–T10 added to the ledger; every new scenario added to the
traceability table and to the retained-scenario justification with the R2/R5
non-observability note; Persona dispositions added; calibredb and runner risks
added; version freshness corrected (calibre resolved at setup, host state
recorded, no invented numbers).

**arc42** (`docs/arc42/*.md`, `givn/changes/passage-retrieval/arc42.md`) —
chapter 2 (calibredb constraint, MPL-2.0, glossary terms); chapter 3 (calibredb
process boundary in both context diagrams, viewer row per D3); chapter 4
(library interface row, SDK home, explicit-failure wording, testability/privacy);
chapter 5 (`library` block, level-2 elements, ordering); chapter 6 (calibredb
participants, export step, agent-visible indexing failures on abstention, viewer
note); chapter 7 (production calibre CLI prerequisites, no content-server twin,
fixture library, two published ports); chapter 8 (subprocess boundary and temp
files, result taxonomy, determinism, persistence, observability, testability,
failure handling); chapter 9 (ADR-0002 title, SDK/calibredb routing, one
destination per routed row); chapter 10 (QS-002, QS-004, QS-005, QS-007, new
QS-012, supporting notes); chapter 11 (R-010 rewritten, R-012/R-013/R-014 added,
TD-009/TD-010 added); chapter 12 (term rows, invariants, refinement note); the
change marker `arc42.md` reasons and ADR verdict wording (all 12 rows stay
`Yes`, `STATUS: DONE`). ADR-0002 was reduced to one decision with a cheaper-home
sentence; the ADR register and the ADR index were updated to match.

**Glossary** (`docs/arc42/12-glossary.md`) — new rows for `Work`, `Work id`,
`Chapter`, `Claim`, `Expansion`, `Passage index`, `Short result`,
`Unavailability report` and `Candidate-passage result`; `Abstention` mapped to
`no evidence`/`NoEvidence`; `chapter` removed as a `Passage` anti-term and
`refusal` qualified as prose only; `Index` → `IndexStore`, `Abstention` →
`NoEvidence`/`AbstentionReason`; numeric work-id ordering; `Citation` refinement
note for D3 with date and source.

**Proposal** (`proposal.md`) — the indexing promise now says the failed-edition
report accompanies an abstention and that discovery and export use the
documented `calibredb` interface; the release note says the citation carries its
exact position and opens its edition and calls the outcome an abstention. The
seven promises are unchanged.

**README / commands** — prerequisites name calibre (`calibredb`), the library
target, the embedding endpoint, Nix, and podman only for local/e2e; usage names
the two MCP tools `search_passages`/`read_passage` under the
`passage-retrieval` capability; licence stays MPL-2.0. `givn/commands.yaml` `run`
block is `./dev-up.sh`, `product_type: server`, `port: 8787`,
`ready_path: /healthz`, `ready_timeout_s: 120`; the verify commands are
unchanged.

**Persona** (`givn/personas/reader--researcher.md`) — 2026-10-27 running-record
entry for D3 and a qualified 2026-10-27 stance; the 2026-10-20 stance is kept.

## Verification

| Command | Exit | Result |
|---|---|---|
| `givn lint --change passage-retrieval` | 2 | 19 findings, all `[WIP]` "scenario … is tagged @wip", 0 other findings |
| `givn status --change passage-retrieval` | 0 | before this file existed: stage `arc42-docs-done`, next artifact design-review; after it was written: stage `design-review-done`, next artifact tasks (not written by this review) |

Grep audit (repository, excluding `givn/archive`):

| Pattern | Hits | Resolution |
|---|---|---|
| `/ajax/books` | 3 | All are explicit negations: the design's library-interface row ("no `/ajax/books` call"), decision T8, and chapter 2 ("never calls an internal HTTP route such as `/ajax/books`"). No plan text relies on the route. |
| `Cli::from_args` | 1 | The design's correction sentence stating the API does not exist. No command uses it. |
| `--tags` combined with `--name` | 0 invocations (4 prose mentions) | Every `--name` invocation in the design uses `--name` alone; the four prose mentions state the clap conflict and the corrected commands. |
| `chunk index` | 2 | Only as an anti-term in the glossary (`Index`, `Passage index`). The use-case R3 and the ideation draft now say passage index. |
| `rust-toolchain.toml` | 2 | Only as an explicit negation in the design toolchain row and in R-010. |
| `libz.so.1` | 0 | The stale claim is gone; the observed host state replaces it. |

## Tracked items carried into tasks

- TD-009 (deferred position-aware viewer opening) with its resolution condition —
  a scope decision, not a deferred mandatory check.
- R-014's strictness proof (`./run-tests.sh --prove-strictness` must exit
  non-zero) is a setup-task deliverable; `verify.e2e_command` stays distinct from
  `verify.command`.
- `@wip` stays on every scenario until step definitions exist; no task list was
  written by this review.

DESIGN-REVIEW: PASS
