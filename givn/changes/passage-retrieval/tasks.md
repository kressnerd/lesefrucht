# Tasks — passage-retrieval

Change: `passage-retrieval` · Use case: `find-passages` · Capability:
`passage-retrieval` (`NEW in find-passages`) · Design:
`givn/changes/passage-retrieval/design.md` · Spec:
`givn/changes/passage-retrieval/specs/find-passages/passage-retrieval.feature`.

**Release note (exact commit subject, one line):**

```text
An agent can submit a claim or phrase to the indexed library and receive up to twenty candidate passages, each with a citation that carries its exact position and opens its edition, the score that ranked it and the expansion that produced it, plus wider context on request and an explicit abstention or unavailability report when the library cannot supply evidence.
```

```sh
RELEASE_NOTE='An agent can submit a claim or phrase to the indexed library and receive up to twenty candidate passages, each with a citation that carries its exact position and opens its edition, the score that ranked it and the expansion that produced it, plus wider context on request and an explicit abstention or unavailability report when the library cannot supply evidence.'
```

## How this list is worked

- Order: setup → every non-`@e2e` scenario in `.feature` order → e2e setup →
  every `@e2e` scenario in `.feature` order → closeout. Never start an `@e2e`
  scenario before the e2e setup task is done.
- One scenario = one RED → GREEN → REFACTOR → COMMIT loop. Remove `@wip` from
  **this scenario only** at RED. At GREEN the steps assert real values and the
  production code exists. At REFACTOR behaviour is unchanged.
- Every single-scenario run is the exact command in the scenario's
  **Design constraints** block. Never infer a result from the whole-suite
  output. Paste command, exit code and output in the evidence field.
- A step body is never empty: RED uses `unimplemented!()` / `todo!()`; GREEN
  replaces it with a real assertion or a real reused setup action.
- Commit once per scenario with `givn commit`; the subject is `RELEASE_NOTE`,
  the body is the scenario title. Record the hash. A revision that touches only
  `tasks.md` or the `.feature` text does not count.
- Never remove an `@e2e` tag. Never silently deviate from `design.md`: if the
  design's command, file or approach is wrong, stop, correct `design.md`,
  re-run design-review, then resume.
- `verify.command` is `./run-tests.sh`; `verify.e2e_command` is
  `./run-tests.sh --e2e`. They are not equal.

## Shared design constraints (apply to every task below)

### Domain and use-case contract

- Terms are the arc42 glossary in `docs/arc42/12-glossary.md`: `Work`,
  `Edition`, `Chapter`, `Passage`, `Passage index`, `Citation`, `Claim`,
  `Expansion`, `Question`, `Genre`, `IndexStore`, `NoEvidence`,
  `Unavailable`. `Chapter` is structural; a passage never crosses it.
- Use-case rules: R1 every passage cites an indexed edition; R2 only passages
  retrieved for this request; R3 order is score, then numeric work id, then
  passage index; R4 capped at the requested number, at most 20; R5 no support
  judgement; R6 `no evidence` is a result, never a failure or empty success;
  R7 scope filters both paths before fusion.
- Minimal guarantee: no passage is returned that does not exist at its
  citation; a failure is a failure, never an empty success; the library,
  metadata and glossary are unchanged.
- Success guarantee: candidate passages with citations in deterministic order,
  and widening with exact bounds.

### Stack and files

- Rust edition 2024; one workspace: `lesefrucht`, `twins/openai-compatible`,
  `xtask`. Versions from design.md Technology Decisions; `Cargo.lock` committed.
  Toolchain: the Nix devShell (`nixpkgs#rustc`, `#cargo`, `#calibre`,
  `#grcov`, `#llvm-tools`); there is no `rust-toolchain.toml`. Calibre is not on
  the host by default — the devShell and product image install it.
- Modules exactly as design.md Module and Component Structure: `config`,
  `domain/{mod,ids,outcome}`, `library/{mod,calibredb,discovery,selection}`,
  `indexing/{pipeline,queue,extract/{mod,epub,pdf,unsupported},chunk,embed,state}`,
  `retrieval/{expand,lexical,vector,scope,fuse,service,context}`,
  `index/{store,sqlite,lexical_index,vector_index,text_store}`,
  `mcp/{server,tools,dto,errors}`, `telemetry`, `main`, `lib`.
- Dependencies: tokio, axum 0.8, rmcp 3.8 (`server`,
  `transport-streamable-http-server`; client features in the e2e driver),
  tantivy 0.27, usearch 2.26, rusqlite 0.41 (WAL), reqwest 0.13, rbook 0.7,
  pdf-extract 0.12, text-splitter 0.33, cucumber 0.23, serde, uuid, thiserror 2,
  anyhow 1 in `main` only.

### Library interface (D2)

- Discovery: `calibredb list --with-library <library.url> --for-machine
  --fields all`. Output is one JSON array of per-book objects:
  `id` numeric, `title`, `authors` (one ` & `-joined string; split on the
  convention), `languages` array (map `deu`→`de`, `eng`→`en`), `tags` array
  (`genre.<class>`), `pubdate` ISO-8601, `formats` array, `last_modified`
  ISO-8601.
- Bytes: `calibredb export --with-library <library.url> --to-dir <tmp>
  --single-dir --formats <FMT> --dont-save-cover --dont-write-opf
  <id>`; take the single file whose extension matches
  the format; remove the temp directory on success and failure.
- `library.url` is required configuration (content-server URL or local path);
  `library.calibredb_binary` defaults to `calibredb`; `library.timeout_ms`
  bounds every invocation. A non-zero exit, timeout or unparsable output is
  `source_unreachable`.
- No HTTP client to calibre, no `/ajax/books`, no `/get`, no `metadata.db`
  access, no write command.

### Index and indexing

- Layout: `data/index/index.sqlite` (metadata, state, passages, citations,
  generation), `data/index/lexical/` (tantivy), `data/index/vectors.usearch`,
  `data/index/vectors.map.json`, `text/<content_hash>.txt`. Offsets are Unicode
  scalar indices into the stored normalized text.
- `IndexState = not_indexed | indexing | indexed | failed(reason) |
  invalidated`; `FailureReason = unsupported_format | extraction_failed |
  embedding_failed | source_unreachable | empty_text`; `StoreState = ready |
  indexing | empty | unavailable`.
- Selection: first available format in `edition.format_preference` (default
  epub, azw3, mobi, pdf); exactly one selected edition per work; a de-selected
  edition's passages are removed in the same commit, so `(work_id,
  passage_index)` is unique.
- Extraction: EPUB via `rbook` (spine/TOC chapters); PDF via `pdf-extract`,
  text split on form feeds; AZW3/MOBI/other →
  `IndexingFailed(unsupported_format)` without export.
- Chunking: chapter-bounded, sentence-aware `text-splitter`, `chunk.max_chars`
  1800, `chunk.overlap_chars` 200; `passage_index` is the ordinal within the
  edition.
- Embedding: batches of `embedding.batch_size`; response length must equal
  `embedding.dimension`; mismatch → `IndexingFailed(embedding_failed)`.
- Commit order: passages + state in one SQLite transaction, then tantivy
  commit, then usearch save; state becomes `indexed` only after all three.
  Startup `repair` invalidates editions whose passage, lexical and vector
  counts disagree.
- Skip: unchanged `content_hash` + `indexed` → no extraction, no embedding.
- Status: `/healthz` exposes `indexed_editions`, `failed_editions`,
  `last_success_at`, `library_generation`, `index_state`; the agent does not
  read it (D1).
- Generation: sha256 over sorted `(work_id, format, last_modified)` triples
  from the listing; read through `index.generation_cache_ms`.
- Freshness: `now - last_success_at <= index.freshness_bound_hours` **and**
  current generation equals recorded.

### MCP contract

- Streamable HTTP at `http://127.0.0.1:8787/mcp`, JSON-RPC 2.0, rmcp session
  handling; server capabilities `tools` only; `/healthz` on the same listener.
- `search_passages`: `claim` 1..2000 chars, `language` BCP-47 required, optional
  `scope {genre, language, authors[], year_from, year_to, work_ids[]}`, optional
  `limit`. `scope` and `limit` optional; every scope field optional.
- `read_passage`: `edition_id`, `passage_index`, optional `window_chars`
  (default 0, clamped to `read.max_window_chars` default 4000 and to the
  chapter).
- Outcomes (`structuredContent`, exactly one arm):
  `candidate_passages {requested, returned, short, expansion, passages[]}`;
  `no_evidence {reason: no_evidence|unknown_work, indexing?: {failed: [{
  work_id, edition_id, format, reason }]}}`; `unavailable {reason:
  index_unavailable|stale_index|library_unreachable}`.
- `requested` = clamped effective limit; `short = returned < requested`;
  `limit 50` → `requested 20`, `returned ≤ 20`, `short = returned < 20`.
- Errors: missing/invalid argument → JSON-RPC `-32602`, `isError: true`,
  `data.reason = invalid_arguments`; unknown edition/passage → `-32602`,
  `data.reason = invalid_citation`. A limit above 20 is clamped, never
  rejected; a limit below 1 or non-integer is invalid.
- Scope resolution: genre matches `genre.<class>` (fachbuch, sachbuch,
  belletristik, sonstiges); language maps calibre codes; authors and years
  filter `works`; AND across fields, OR within authors; a non-numeric work id
  is `invalid_arguments`; any valid id absent from the library →
  `unknown_work` before either path.
- Retrieval: expansion = NFKC/lowercase/Unicode alphabetic tokens + accepted
  glossary terms (empty) + configured synonyms; tantivy BM25 with the
  per-language stemmer and a `work_id` TermSet; usearch cosine with
  `retrieval.vector_overfetch`; candidate survives iff
  `bm25 >= retrieval.lexical_min_score` or
  `cosine >= retrieval.vector_min_cosine`; RRF `Σ 1/(rrf_k + rank)`, `rrf_k`
  60; sort `(fused_score desc, work_id numeric asc, passage_index asc)`.
- Citation: work id, edition id, format, title, authors, language, chapter
  index/title, passage index, `char_start`/`char_end`, `viewer_url =
  calibre://show-book/<library>/<work>` (opens the edition; no position).
  `read_passage` slices `[cited.start_char, cited.end_char)` and verifies its
  sha256 against the passage `text_hash`.

### Test infrastructure and evidence

- Runner: `tests/features_runner.rs`, `harness = false`; `World::cucumber()
  .fail_on_skipped().with_cli(cucumber::cli::Opts::<_, _, _>::parsed())
  .run_and_exit("givn").await;`. `--name` and `--tags` conflict; use `--name`
  alone for one scenario. The Basic parser walks `givn` recursively, so both
  durable and delta features are found.
- Step files: non-e2e `tests/steps/passage_retrieval.rs`; e2e
  `tests/steps/passage_retrieval_e2e.rs`; support `world.rs`, `twins.rs`,
  `fixtures.rs`, `calibredb.rs`, `mcp_client.rs`. No delta step copies.
- Tests run the real `calibredb` against a fresh fixture calibre library built
  by `xtask fixtures` with `calibredb add` (fixed mtimes); no calibre twin. The
  only twin is the deterministic `twins/openai-compatible` embedding endpoint
  (in-process for non-e2e, container for e2e).
- Fixtures: work-1 German EPUB fachbuch, Anna Beispiel 2001, 3 chapters,
  claim in chapter 2; work-2 English EPUB fachbuch, Clara Example 2005,
  English mirror "trust ... repeated interaction"; work-3 German EPUB
  sachbuch, Bernd Beispiel 2012, same claim; work-4 German PDF sachbuch;
  work-5 German MOBI only, fails `unsupported_format`; work-6 German EPUB,
  4 chapters, 4 passages above threshold; work-7 German EPUB, 30 chapters
  matching the claim; work-8 German EPUB, none of the probe terms.
- Expansion config: `LESEFRUCHT__EXPANSION__SYNONYMS_FILE=tests/fixtures/synonyms.toml`,
  entry `"Vertrauen" = ["trust"]`.
- Test knobs through the environment: `LESEFRUCHT__RETRIEVAL__LEXICAL_MIN_SCORE`,
  `...__VECTOR_MIN_COSINE`, `...__LIMIT_DEFAULT`, `...__LIMIT_MAX`,
  `LESEFRUCHT__INDEX__FRESHNESS_BOUND_HOURS`,
  `LESEFRUCHT__INDEX__GENERATION_CACHE_MS`,
  `LESEFRUCHT__READ__MAX_WINDOW_CHARS`.
- Strictness proof is run once in setup: `cargo test --test features_runner --
  --input tests/fixtures/strictness.feature` must exit non-zero. The probe is
  then removed/fixed; the real baseline excludes `@wip`.
- A single-scenario run never uses `--tags` together with `--name`.

---

## Phase 0 — Setup

### 0.1 Workspace, toolchain, configuration, health endpoint

- [ ] Create the Cargo workspace and skeleton: `Cargo.toml` (edition 2024,
  members `lesefrucht`, `twins/openai-compatible`, `xtask`), `src/main.rs`,
  `src/lib.rs`, module files from the design, `flake.nix` with the devShell
  (`rustc`, `cargo`, `calibre`, `grcov`, `llvm-tools`, podman), `.gitignore`
  for `data/`, `coverage/`, `target/`. No `rust-toolchain.toml`.
- [ ] `src/config.rs`: TOML + `LESEFRUCHT__<SECTION>__<KEY>` overrides; startup
  fails with a named error when `library.url`, `embedding.base_url`,
  `embedding.model` or a positive `embedding.dimension` is missing; defaults
  from design.md Configuration.
- [ ] `src/main.rs` + `mcp/server.rs`: axum listener on `server.bind`,
  `GET /healthz` reporting `indexed_editions`, `failed_editions`,
  `last_success_at`, `library_generation`, `index_state`; `GET /healthz` 200
  with an empty store and `index_state = empty`.
- [ ] Evidence: `nix develop --command cargo build`, `nix develop --command
  calibredb --version`, and a manual `curl -s http://127.0.0.1:8787/healthz`.
  Record the resolved Rust and calibre versions in design.md Version Freshness
  (the setup task owns this; no invented number).
  ```
  paste commands, exit codes, output
  ```

### 0.2 Gherkin runner, strict mode and proof

- [ ] `Cargo.toml`: `[[test]] name = "features_runner"; harness = false`.
- [ ] `tests/features_runner.rs` exactly as shared constraints; `tests/steps/
  passage_retrieval.rs`, `tests/steps/passage_retrieval_e2e.rs` and
  `tests/steps/support/{world,twins,fixtures,calibredb,mcp_client}.rs` created;
  no phrase bound yet; no empty step body.
- [ ] `tests/fixtures/strictness.feature`: one scenario with one step that is
  deliberately unbound.
- [ ] `./run-tests.sh`: non-e2e run under coverage instrumentation; `--e2e`
  brings `compose.e2e.yaml` up, runs `@e2e`, tears down with a trap;
  `--prove-strictness` runs the strictness feature and asserts a non-zero exit.
- [ ] Prove strictness: `cargo test --test features_runner -- --input
  tests/fixtures/strictness.feature` exits non-zero. Paste the output.
- [ ] Remove/fix the probe; baseline `cargo test --test features_runner --
  --tags 'not @e2e and not @wip'` exits 0 with zero selected scenarios.
  (`verify.command` intentionally fails while any `@wip` scenario remains; it
  becomes green only after the last non-e2e scenario is GREEN.)
- [ ] `givn/commands.yaml` already carries `verify.command: ./run-tests.sh`,
  `verify.e2e_command: ./run-tests.sh --e2e` and the `run:` block
  (`./dev-up.sh`, server, 8787, `/healthz`, 120 s); confirm without changing
  them.
  ```
  paste strictness non-zero output and baseline 0 output
  ```

### 0.3 Fixture library and test world with the real calibredb

- [ ] `xtask fixtures`: write reviewable text for work-1..work-8, add them with
  `calibredb add` at fixed mtimes, set `genre.<class>`, language, authors
  (Anna Beispiel/2001, Clara Example/2005, Bernd Beispiel/2012), pubdate and
  formats; work-5 only MOBI. Output a temporary calibre library directory.
- [ ] `tests/fixtures/synonyms.toml` with `"Vertrauen" = ["trust"]`.
- [ ] `tests/steps/support/{world,fixtures,calibredb}.rs`: per-scenario temp
  library; spawn the real `calibredb` for `list`/`export`; record every spawned
  command for the non-mutation assertion; expose the fixture work ids and the
  expected metadata.
- [ ] Evidence: a full `calibredb list --with-library <temp> --for-machine
  --fields all` shows the eight works with the named metadata; a full export of
  work-1 produces one EPUB file.
  ```
  paste commands, exit codes, representative JSON
  ```

### 0.4 Index store and indexing pipeline

- [ ] `index/{sqlite,lexical_index,vector_index,text_store,store}.rs` with the
  schema/layout, `commit_edition` in the design order, `repair`, and read paths.
- [ ] `library/{calibredb,discovery,selection}.rs`; `indexing/{pipeline,queue,
  extract/*,chunk,embed,state}.rs`; `twins/openai-compatible` deterministic
  `/v1/models` and `/v1/embeddings` (pure function of text into `dimension`).
- [ ] Index the fixture library once; `/healthz` reports
  `indexed_editions = 7`, `failed_editions` contains work-5 with
  `unsupported_format`, `index_state = ready`.
- [ ] Evidence: the health JSON and a second indexing pass with no embedding
  calls (skip by content hash).
  ```
  paste commands, exit codes, health JSON
  ```

### 0.5 MCP server, tools and session

- [ ] `mcp/{tools,dto,errors}.rs` with the two tool schemas, tagged outcomes and
  `-32602` errors; rmcp server and session; `/mcp` mounted next to `/healthz`.
- [ ] Evidence: an `initialize` + `tools/list` against `127.0.0.1:8787/mcp`
  returns exactly `search_passages` and `read_passage`; a malformed call
  returns `-32602`.
  ```
  paste client transcript or test output
  ```

### 0.6 Compose, dev scripts and coverage pipeline

- [ ] `compose.dev.yaml`, `compose.e2e.yaml`, `dev-up.sh`, `dev-down.sh`,
  `run-tests.sh`, `measure-coverage.sh`, `merge-coverages.sh`,
  `scripts/merge_cobertura.py`; isolated networks `lesefrucht-dev` and
  `lesefrucht-e2e`; product image installs calibre and the fixture library; the
  embedding twin is the only other process; loopback ports 8787 (dev/e2e) and
  8789 (embedding twin).
- [ ] Coverage: non-e2e profiles under `coverage/profraw/non-e2e/`, e2e server
  and runner profiles under `coverage/profraw/e2e/`, two Cobertura files and
  the merged output; one known production path is non-zero per report.
- [ ] Evidence: `./run-tests.sh` produces both reports; `./merge-coverages.sh`
  writes `coverage/cobertura-coverage.xml`.
  ```
  paste commands, exit codes, file listing
  ```

---

## Phase 1 — Non-e2e scenarios

### S1 — Scope restricts passages to the named genre

`@givn.added` · non-e2e · first non-e2e scenario in the feature

**Design constraints**
- Fixture: work-1 (fachbuch) and work-3 (sachbuch) both contain the claim.
- Request: claim `"Vertrauen in Organisationen"`, language `de`, scope
  `genre=fachbuch`, limit 5.
- Then: at least one returned passage is from a fachbuch work; every citation
  belongs to a fachbuch work; no citation belongs to work-3 or any other genre.
- Production: `retrieval/scope.rs` (genre filter, AND across fields),
  `retrieval/service.rs`, `library/discovery.rs` (`genre.<class>` parse),
  `index/sqlite.rs` (`works.genre`).
- Suite evidence is on `structuredContent`; the vector and lexical paths both
  receive the scope before fusion.
- Single-scenario: `cargo test --test features_runner -- --name '^Scope restricts passages to the named genre$'`

- [ ] **RED** — remove `@wip` from this scenario only; bind its steps with
  `unimplemented!()`; run the single-scenario command; paste the output. Must
  exit non-zero.
  ```
  paste command, exit code, output
  ```
- [ ] **GREEN** — implement the scope resolution and the two paths' scope
  application; list every production file created/modified; compile; run the
  single-scenario command; paste the output. Must exit 0.
  ```
  paste production file list, command, exit code, output
  ```
- [ ] **REFACTOR** — clean up without behaviour change; re-run the
  single-scenario command; paste the output. Must exit 0.
  ```
  paste command, exit code, output
  ```
- [ ] **COMMIT** — `givn commit -m "$(printf '%s\n\n%s' "$RELEASE_NOTE" 'Scope restricts passages to the named genre')"`; record the hash.
  `COMMIT:`

### S2 — A scope naming an unknown work yields no evidence

**Design constraints**
- Request: any valid claim, scope `work_ids=["999"]`; `999` is a well-formed
  calibre id absent from the fixture library.
- Then: `no_evidence`, reason `unknown_work`, no passage; not an MCP error and
  not an empty `candidate_passages`.
- Production: `retrieval/scope.rs` (strict absent-id rule before either path),
  `retrieval/service.rs`, `mcp/dto.rs`, `mcp/tools.rs`.
- A non-numeric work id is `invalid_arguments`, not `unknown_work`.
- Single-scenario: `cargo test --test features_runner -- --name '^A scope naming an unknown work yields no evidence$'`

- [ ] **RED** — remove `@wip` from this scenario only; stubs; run targeted; paste
  non-zero output.
  ```
  paste command, exit code, output
  ```
- [ ] **GREEN** — implement the unknown-work arm; list production files; compile;
  run targeted; paste zero output.
  ```
  paste production file list, command, exit code, output
  ```
- [ ] **REFACTOR** — re-run targeted; paste zero output.
  ```
  paste command, exit code, output
  ```
- [ ] **COMMIT** — `givn commit -m "$(printf '%s\n\n%s' "$RELEASE_NOTE" 'A scope naming an unknown work yields no evidence')"`.
  `COMMIT:`

### S3 — No passage above the threshold yields no evidence

**Design constraints**
- Fixture: work-8 contains none of the probe terms; claim absent from the
  library above the threshold.
- Then: `no_evidence`, reason `no_evidence`; no `passages`; neither a JSON-RPC
  error nor a successful empty list.
- Production: `retrieval/fuse.rs` (evidence threshold),
  `retrieval/service.rs`, `retrieval/lexical.rs`, `retrieval/vector.rs`,
  `mcp/dto.rs`.
- Thresholds from `retrieval.lexical_min_score` / `vector_min_cosine`; tests may
  set them through the environment.
- Single-scenario: `cargo test --test features_runner -- --name '^No passage above the threshold yields no evidence$'`

- [ ] **RED** — un-`@wip` this scenario only; stubs; targeted run; paste non-zero.
  ```
  paste command, exit code, output
  ```
- [ ] **GREEN** — threshold and abstention arm; list production files; compile;
  targeted run; paste zero.
  ```
  paste production file list, command, exit code, output
  ```
- [ ] **REFACTOR** — targeted run; paste zero.
  ```
  paste command, exit code, output
  ```
- [ ] **COMMIT** — `givn commit -m "$(printf '%s\n\n%s' "$RELEASE_NOTE" 'No passage above the threshold yields no evidence')"`.
  `COMMIT:`

### S4 — A stale index yields an unavailability report

**Design constraints**
- Given: an index older than the freshness bound. Run with
  `LESEFRUCHT__INDEX__FRESHNESS_BOUND_HOURS=0` and
  `LESEFRUCHT__INDEX__GENERATION_CACHE_MS=0`; the recorded `last_success_at` is
  outside the bound. No clock is mocked.
- Then: `unavailable`, reason `stale_index`; not `no_evidence`.
- Production: `indexing/state.rs` (freshness), `retrieval/service.rs` (check
  before retrieval), `index/store.rs`, `mcp/dto.rs`.
- The check runs before expansion and retrieval, so unavailability can never be
  reported as abstention.
- Single-scenario: `cargo test --test features_runner -- --name '^A stale index yields an unavailability report$'`

- [ ] **RED** — un-`@wip`; stubs; targeted; paste non-zero.
  ```
  paste command, exit code, output
  ```
- [ ] **GREEN** — freshness check and arm; list production files; compile;
  targeted; paste zero.
  ```
  paste production file list, command, exit code, output
  ```
- [ ] **REFACTOR** — targeted; paste zero.
  ```
  paste command, exit code, output
  ```
- [ ] **COMMIT** — `givn commit -m "$(printf '%s\n\n%s' "$RELEASE_NOTE" 'A stale index yields an unavailability report')"`.
  `COMMIT:`

### S5 — An unavailable index yields an unavailability report

**Design constraints**
- Given: no edition is indexed (fresh store, `index_state = empty`).
- Then: `unavailable`, reason `index_unavailable`; not `no_evidence`.
- Production: `indexing/state.rs`, `index/store.rs` (empty/unavailable state),
  `retrieval/service.rs`, `mcp/dto.rs`.
- Distinct gate from the stale case; both precede retrieval.
- Single-scenario: `cargo test --test features_runner -- --name '^An unavailable index yields an unavailability report$'`

- [ ] **RED** — un-`@wip`; stubs; targeted; paste non-zero.
  ```
  paste command, exit code, output
  ```
- [ ] **GREEN** — empty-store arm; list production files; compile; targeted;
  paste zero.
  ```
  paste production file list, command, exit code, output
  ```
- [ ] **REFACTOR** — targeted; paste zero.
  ```
  paste command, exit code, output
  ```
- [ ] **COMMIT** — `givn commit -m "$(printf '%s\n\n%s' "$RELEASE_NOTE" 'An unavailable index yields an unavailability report')"`.
  `COMMIT:`

### S6 — Fewer passages than requested are returned short

**Design constraints**
- Fixture: work-6 has four passages above the threshold; request limit 20.
- Then: `requested` 20, `returned` 4, `short` true; no padding.
- Production: `retrieval/service.rs` (`effective_limit`, `short`),
  `retrieval/fuse.rs`, `mcp/dto.rs`.
- `requested` is the clamped effective limit; `short = returned < requested`.
- Single-scenario: `cargo test --test features_runner -- --name '^Fewer passages than requested are returned short$'`

- [ ] **RED** — un-`@wip`; stubs; targeted; paste non-zero.
  ```
  paste command, exit code, output
  ```
- [ ] **GREEN** — short accounting; list production files; compile; targeted;
  paste zero.
  ```
  paste production file list, command, exit code, output
  ```
- [ ] **REFACTOR** — targeted; paste zero.
  ```
  paste command, exit code, output
  ```
- [ ] **COMMIT** — `givn commit -m "$(printf '%s\n\n%s' "$RELEASE_NOTE" 'Fewer passages than requested are returned short')"`.
  `COMMIT:`

### S7 — The result is capped at twenty passages

**Design constraints**
- Fixture: work-7 has 30 chapters matching the claim; request limit 50.
- Then: `requested` 20, `returned` 20, `short` false.
- Production: `retrieval/service.rs` (`clamp(…, 1, limit_max)`, `requested =
  effective_limit`), `mcp/dto.rs`.
- A limit above 20 is clamped, never rejected; scenario 13 covers the invalid
  side (limit below 1).
- Single-scenario: `cargo test --test features_runner -- --name '^The result is capped at twenty passages$'`

- [ ] **RED** — un-`@wip`; stubs; targeted; paste non-zero.
  ```
  paste command, exit code, output
  ```
- [ ] **GREEN** — clamp and counts; list production files; compile; targeted;
  paste zero.
  ```
  paste production file list, command, exit code, output
  ```
- [ ] **REFACTOR** — targeted; paste zero.
  ```
  paste command, exit code, output
  ```
- [ ] **COMMIT** — `givn commit -m "$(printf '%s\n\n%s' "$RELEASE_NOTE" 'The result is capped at twenty passages')"`.
  `COMMIT:`

### S8 — A retrieval never changes the library

**Design constraints**
- Given: record `calibredb list --with-library <library.url> --for-machine
  --fields all` output, the accepted glossary rows, the `library_generation`
  and `last_success_at` from `/healthz`.
- When: `search_passages` and then `read_passage` on one citation.
- Then: the listing, glossary, generation and last-success time are identical;
  only `calibredb list` and `calibredb export` were spawned; no write command.
- Production: read-only `retrieval/service.rs`, `retrieval/context.rs`,
  `library/calibredb.rs`; the test support records spawned commands.
- Single-scenario: `cargo test --test features_runner -- --name '^A retrieval never changes the library$'`

- [ ] **RED** — un-`@wip`; stubs; targeted; paste non-zero.
  ```
  paste command, exit code, output
  ```
- [ ] **GREEN** — read-only guarantee and command recorder; list production
  files; compile; targeted; paste zero.
  ```
  paste production file list, command, exit code, output
  ```
- [ ] **REFACTOR** — targeted; paste zero.
  ```
  paste command, exit code, output
  ```
- [ ] **COMMIT** — `givn commit -m "$(printf '%s\n\n%s' "$RELEASE_NOTE" 'A retrieval never changes the library')"`.
  `COMMIT:`

### S9 — Identical requests return the same sequence

**Design constraints**
- Two identical `search_passages` calls (same claim, scope, limit) against an
  unchanged index and generation.
- Then: both `candidate_passages` results carry the same citations in the same
  order (work id, edition id, passage index, char offsets).
- Production: `retrieval/fuse.rs` (RRF and sort key; no randomness),
  `retrieval/service.rs` (no per-request state).
- Single-scenario: `cargo test --test features_runner -- --name '^Identical requests return the same sequence$'`

- [ ] **RED** — un-`@wip`; stubs; targeted; paste non-zero.
  ```
  paste command, exit code, output
  ```
- [ ] **GREEN** — determinism; list production files; compile; targeted; paste
  zero.
  ```
  paste production file list, command, exit code, output
  ```
- [ ] **REFACTOR** — targeted; paste zero.
  ```
  paste command, exit code, output
  ```
- [ ] **COMMIT** — `givn commit -m "$(printf '%s\n\n%s' "$RELEASE_NOTE" 'Identical requests return the same sequence')"`.
  `COMMIT:`

### S10 — A returned citation resolves to the passage it cites

**Design constraints**
- Search for a fixture claim above the threshold; take the first citation;
  call `read_passage` with `window_chars: 0`.
- Then: the returned text equals the passage text at `cited.start_char` /
  `cited.end_char` and equals the stored normalized slice; the sha256 matches
  the passage `text_hash`.
- Production: `retrieval/context.rs` (slice and hash verification),
  `index/text_store.rs`, `index/store.rs`, `mcp/tools.rs`.
- Offsets are Unicode scalar indices, not bytes.
- Single-scenario: `cargo test --test features_runner -- --name '^A returned citation resolves to the passage it cites$'`

- [ ] **RED** — un-`@wip`; stubs; targeted; paste non-zero.
  ```
  paste command, exit code, output
  ```
- [ ] **GREEN** — context slice and hash check; list production files; compile;
  targeted; paste zero.
  ```
  paste production file list, command, exit code, output
  ```
- [ ] **REFACTOR** — targeted; paste zero.
  ```
  paste command, exit code, output
  ```
- [ ] **COMMIT** — `givn commit -m "$(printf '%s\n\n%s' "$RELEASE_NOTE" 'A returned citation resolves to the passage it cites')"`.
  `COMMIT:`

### S11 — A widened context is clamped to the chapter and the maximum window

**Design constraints**
- Given: a cited passage that starts less than the maximum window after its
  chapter start. Set `LESEFRUCHT__READ__MAX_WINDOW_CHARS=120` (or use the
  default 4000 with an equivalent fixture passage).
- When: `read_passage` with `window_chars` wider than the maximum.
- Then: bounds do not extend beyond the chapter, the window is no wider than
  the configured maximum, and `truncated` is true.
- Production: `retrieval/context.rs` (expand, clamp to chapter and max),
  `config.rs`.
- `window_chars` default 0; both clamps apply; `truncated` reports a clamp.
- Single-scenario: `cargo test --test features_runner -- --name '^A widened context is clamped to the chapter and the maximum window$'`

- [ ] **RED** — un-`@wip`; stubs; targeted; paste non-zero.
  ```
  paste command, exit code, output
  ```
- [ ] **GREEN** — clamps; list production files; compile; targeted; paste zero.
  ```
  paste production file list, command, exit code, output
  ```
- [ ] **REFACTOR** — targeted; paste zero.
  ```
  paste command, exit code, output
  ```
- [ ] **COMMIT** — `givn commit -m "$(printf '%s\n\n%s' "$RELEASE_NOTE" 'A widened context is clamped to the chapter and the maximum window')"`.
  `COMMIT:`

### S12 — An unknown citation is rejected as an invalid request

**Design constraints**
- `read_passage` with an `edition_id` and/or `passage_index` the index does not
  hold (e.g. `calibre:999:epub`, `passage_index: 999999`).
- Then: JSON-RPC `-32602`, `isError: true`, `data.reason = invalid_citation`;
  not `no_evidence`.
- Production: `mcp/errors.rs`, `mcp/tools.rs`, `retrieval/context.rs`.
- Protocol errors are reserved for malformed requests and citations that
  contradict the index.
- Single-scenario: `cargo test --test features_runner -- --name '^An unknown citation is rejected as an invalid request$'`

- [ ] **RED** — un-`@wip`; stubs; targeted; paste non-zero.
  ```
  paste command, exit code, output
  ```
- [ ] **GREEN** — error mapping; list production files; compile; targeted; paste
  zero.
  ```
  paste production file list, command, exit code, output
  ```
- [ ] **REFACTOR** — targeted; paste zero.
  ```
  paste command, exit code, output
  ```
- [ ] **COMMIT** — `givn commit -m "$(printf '%s\n\n%s' "$RELEASE_NOTE" 'An unknown citation is rejected as an invalid request')"`.
  `COMMIT:`

### S13 — A malformed request is rejected

**Design constraints**
- Three validation variants, one rule: a request with no `claim`; a request with
  no `language`; a request with `limit` below 1 (use `0`). Cover each; a limit
  above 20 is clamped, never invalid.
- Then: JSON-RPC `-32602`, `isError: true`,
  `data.reason = invalid_arguments`; no result.
- Production: `mcp/dto.rs` (schema), `mcp/tools.rs` (validation),
  `mcp/errors.rs`.
- Single-scenario: `cargo test --test features_runner -- --name '^A malformed request is rejected$'`

- [ ] **RED** — un-`@wip`; stubs; targeted; paste non-zero.
  ```
  paste command, exit code, output
  ```
- [ ] **GREEN** — validation; list production files; compile; targeted; paste
  zero.
  ```
  paste production file list, command, exit code, output
  ```
- [ ] **REFACTOR** — targeted; paste zero.
  ```
  paste command, exit code, output
  ```
- [ ] **COMMIT** — `givn commit -m "$(printf '%s\n\n%s' "$RELEASE_NOTE" 'A malformed request is rejected')"`.
  `COMMIT:`

### S14 — Scope filters passages by language

**Design constraints**
- Fixture: work-1 (German) and work-2 (English) both contain the claim.
- Request: claim language `de`, scope `language=en`; the request language and
  the scope language are different fields.
- Then: at least one citation belongs to an English work; every citation does.
- Production: `retrieval/scope.rs`, `index/sqlite.rs` (works.language),
  `library/discovery.rs` (calibre `deu`→`de`, `eng`→`en`).
- AND across scope fields; OR within authors.
- Single-scenario: `cargo test --test features_runner -- --name '^Scope filters passages by language$'`

- [ ] **RED** — un-`@wip`; stubs; targeted; paste non-zero.
  ```
  paste command, exit code, output
  ```
- [ ] **GREEN** — language mapping and filter; list production files; compile;
  targeted; paste zero.
  ```
  paste production file list, command, exit code, output
  ```
- [ ] **REFACTOR** — targeted; paste zero.
  ```
  paste command, exit code, output
  ```
- [ ] **COMMIT** — `givn commit -m "$(printf '%s\n\n%s' "$RELEASE_NOTE" 'Scope filters passages by language')"`.
  `COMMIT:`

### S15 — Scope filters passages by author and year

**Design constraints**
- Fixture: work-1 (Anna Beispiel, 2001) and work-3 (Bernd Beispiel, 2012) both
  contain the claim.
- Request: scope `authors=["Anna Beispiel"]`, `year_from=2001`, `year_to=2001`.
- Then: at least one citation is from work-1; every citation is from that
  author and year.
- Production: `retrieval/scope.rs`, `index/sqlite.rs` (`works.authors`,
  `works.pubdate`), `library/discovery.rs` (split ` & `, parse ISO pubdate).
- AND across fields; OR within authors; a name containing ` & ` is the recorded
  TD-010 limitation.
- Single-scenario: `cargo test --test features_runner -- --name '^Scope filters passages by author and year$'`

- [ ] **RED** — un-`@wip`; stubs; targeted; paste non-zero.
  ```
  paste command, exit code, output
  ```
- [ ] **GREEN** — author/year filters; list production files; compile; targeted;
  paste zero.
  ```
  paste production file list, command, exit code, output
  ```
- [ ] **REFACTOR** — targeted; paste zero.
  ```
  paste command, exit code, output
  ```
- [ ] **COMMIT** — `givn commit -m "$(printf '%s\n\n%s' "$RELEASE_NOTE" 'Scope filters passages by author and year')"`.
  `COMMIT:`

### S16 — An abstention names the editions that failed indexing

**Design constraints**
- Fixture: work-5 holds only a MOBI edition and is recorded
  `failed(unsupported_format)`.
- Request: a claim with scope `work_ids=["5"]`; work 5 is in the library, so
  the result is not `unknown_work`.
- Then: `no_evidence`, reason `no_evidence`, and `indexing.failed` contains
  exactly `{work_id: "5", edition_id: "calibre:5:mobi", format: "mobi",
  reason: "unsupported_format"}`; no third tool.
- Production: `indexing/extract/unsupported.rs`, `indexing/state.rs`,
  `retrieval/service.rs` (relevant failures), `mcp/dto.rs`.
- `indexing` is omitted when no relevant edition failed and never appears on
  `candidate_passages` or `unavailable`.
- Single-scenario: `cargo test --test features_runner -- --name '^An abstention names the editions that failed indexing$'`

- [ ] **RED** — un-`@wip`; stubs; targeted; paste non-zero.
  ```
  paste command, exit code, output
  ```
- [ ] **GREEN** — failure summary; list production files; compile; targeted;
  paste zero.
  ```
  paste production file list, command, exit code, output
  ```
- [ ] **REFACTOR** — targeted; paste zero.
  ```
  paste command, exit code, output
  ```
- [ ] **COMMIT** — `givn commit -m "$(printf '%s\n\n%s' "$RELEASE_NOTE" 'An abstention names the editions that failed indexing')"`.
  `COMMIT:`

### S17 — An unreachable library yields an unavailability report

**Design constraints**
- Given: `library.url` names a target that cannot be listed (a missing path or
  a `calibredb` that exits non-zero); startup still succeeds because the
  configuration is present.
- When: `search_passages`; the generation read fails.
- Then: `unavailable`, reason `library_unreachable`; not `no_evidence`.
- Production: `library/calibredb.rs` (timeout, non-zero, unparsable →
  `source_unreachable`), `retrieval/service.rs`, `mcp/dto.rs`.
- Single-scenario: `cargo test --test features_runner -- --name '^An unreachable library yields an unavailability report$'`

- [ ] **RED** — un-`@wip`; stubs; targeted; paste non-zero.
  ```
  paste command, exit code, output
  ```
- [ ] **GREEN** — failure classification; list production files; compile;
  targeted; paste zero.
  ```
  paste production file list, command, exit code, output
  ```
- [ ] **REFACTOR** — targeted; paste zero.
  ```
  paste command, exit code, output
  ```
- [ ] **COMMIT** — `givn commit -m "$(printf '%s\n\n%s' "$RELEASE_NOTE" 'An unreachable library yields an unavailability report')"`.
  `COMMIT:`

---

## Phase 2 — E2E setup

### E2E.1 Compose stack, e2e driver and filter proof

**Design constraints**
- `compose.e2e.yaml`: product image (instrumented, calibre installed, fixture
  calibre library), deterministic embedding twin, isolated network
  `lesefrucht-e2e`. Loopback ports: 8787 product, 8789 embedding twin. No
  calibre twin.
- Product env: `LESEFRUCHT__LIBRARY__URL=<fixture path>`,
  `LESEFRUCHT__EMBEDDING__BASE_URL=http://<embedding twin>:8789`,
  `LESEFRUCHT__EMBEDDING__MODEL`, `LESEFRUCHT__EMBEDDING__DIMENSION`,
  `LESEFRUCHT__EXPANSION__SYNONYMS_FILE=/fixtures/synonyms.toml`,
  `LESEFRUCHT__INDEX__GENERATION_CACHE_MS=0` for the stale scenario only.
- Readiness: `/healthz` 200 with `indexed_editions >= 6`,
  `index_state = ready`, and `failed_editions` containing work-5 with
  `unsupported_format`.
- Driver: rmcp `client` with
  `transport-streamable-http-client-reqwest` against
  `http://127.0.0.1:8787/mcp`; one client per scenario, closed in teardown;
  responses parsed by rmcp, never a raw body.
- `verify.e2e_command` is `./run-tests.sh --e2e`, distinct from
  `./run-tests.sh`; the e2e run filters `@e2e` only.
- Step file `tests/steps/passage_retrieval_e2e.rs`; support
  `mcp_client.rs`.
- Strictness proof for the e2e runner: the same `--input
  tests/fixtures/strictness.feature` probe exits non-zero; remove/fix it
  afterwards.

- [ ] Bring the stack up with `podman-compose -f compose.e2e.yaml up -d
  --build`; confirm readiness and an `initialize` + `tools/list` smoke call;
  tear down with `down -v`. Paste the health JSON and the smoke transcript.
  ```
  paste commands, exit codes, health JSON, transcript
  ```
- [ ] Create the e2e step file and support; prove the runner is strict with the
  unbound probe; remove/fix the probe. Paste the non-zero proof.
  ```
  paste command, exit code, output
  ```
- [ ] Prove the filter is real: run `./run-tests.sh` (non-e2e) and record its
  scenario count; run `./run-tests.sh --e2e` and record its scenario count; the
  e2e count must be strictly smaller than the full count. At this point: 17
  non-e2e, 2 e2e, 19 total. Paste both counts.
  ```
  paste commands, exit codes, counts
  ```

---

## Phase 3 — E2E scenarios

### S18 — Agent retrieves cited passages for a claim

`@givn.added @e2e` · first `@e2e` scenario

**Design constraints**
- Driver: rmcp Streamable HTTP client against `127.0.0.1:8787/mcp`;
  `initialize`, then `tools/call` `search_passages` with claim
  `"Vertrauen in Organisationen"`, `language: de`, `scope.genre: fachbuch`,
  `limit: 5`.
- Primary assertions on `structuredContent`: `kind = candidate_passages`; at
  least one and at most five passages; `short == (returned < requested)`; every
  citation points to an indexed edition; at least one citation belongs to the
  English work-2; `expanded_terms` contains `trust`; order is score, numeric
  work id, passage index.
- `/healthz` is readiness only, never the assertion.
- Step file `tests/steps/passage_retrieval_e2e.rs`; support `mcp_client.rs`.
- Single-scenario (after the stack is up and ready):
  `cargo test --test features_runner -- --name '^Agent retrieves cited passages for a claim$'`

- [ ] **RED** — remove `@wip` from this scenario only; bind the steps with
  `unimplemented!()`; run the single-scenario command; paste non-zero.
  ```
  paste command, exit code, output
  ```
- [ ] **GREEN** — implement the rmcp client call and assertions; list
  production/test files; compile; run targeted; paste zero.
  ```
  paste file list, command, exit code, output
  ```
- [ ] **REFACTOR** — targeted; paste zero.
  ```
  paste command, exit code, output
  ```
- [ ] **COMMIT** — `givn commit -m "$(printf '%s\n\n%s' "$RELEASE_NOTE" 'Agent retrieves cited passages for a claim')"`.
  `COMMIT:`

### S19 — Agent widens context around a cited passage

`@givn.added @e2e` · second `@e2e` scenario

**Design constraints**
- The same rmcp client: take `edition_id` and `passage_index` from a citation
  returned by `search_passages` in this scenario, then `tools/call`
  `read_passage` with `window_chars: 400`.
- Primary assertions on `structuredContent`: `kind = passage_context`; the
  returned `text` at `cited.start_char..cited.end_char` equals the cited
  passage text; `bounds.start_char <= cited.start_char < cited.end_char <=
  bounds.end_char`; the bounds chapter equals the cited chapter.
- Step file `tests/steps/passage_retrieval_e2e.rs`; support `mcp_client.rs`.
- Single-scenario (after the stack is up and ready):
  `cargo test --test features_runner -- --name '^Agent widens context around a cited passage$'`

- [ ] **RED** — remove `@wip` from this scenario only; stubs; targeted; paste
  non-zero.
  ```
  paste command, exit code, output
  ```
- [ ] **GREEN** — implement the read call and assertions; list files; compile;
  targeted; paste zero.
  ```
  paste file list, command, exit code, output
  ```
- [ ] **REFACTOR** — targeted; paste zero.
  ```
  paste command, exit code, output
  ```
- [ ] **COMMIT** — `givn commit -m "$(printf '%s\n\n%s' "$RELEASE_NOTE" 'Agent widens context around a cited passage')"`.
  `COMMIT:`

---

## Closeout

- [ ] Remove every remaining `@wip` tag; no `@e2e` tag was ever removed.
  `givn lint --change passage-retrieval` exits 0. Paste the output.
  ```
  paste command, exit code, output
  ```
- [ ] `./run-tests.sh` exits 0 on the full non-e2e suite; paste the scenario
  count and output.
  ```
  paste command, exit code, counts
  ```
- [ ] `./run-tests.sh --e2e` exits 0 on the `@e2e` suite; paste the scenario
  count and output.
  ```
  paste command, exit code, counts
  ```
- [ ] Both coverage reports are produced by distinct runs and the merged
  Cobertura file exists; a known production path is non-zero in each.
  ```
  paste commands, exit codes, file listing
  ```
- [ ] Step-definition closeout: every phrase bound by the delta steps is bound
  in the permanent per-capability files (`tests/steps/passage_retrieval.rs`,
  `tests/steps/passage_retrieval_e2e.rs`); no delta step copy exists. Paste the
  check.
  ```
  paste command, exit code, output
  ```
- [ ] `givn status --change passage-retrieval` shows every task checked with a
  `COMMIT:` hash; no scenario was committed by a revision that touches only
  `tasks.md` or the `.feature` text.
  ```
  paste status output and the revision list
  ```
- [ ] `givn archive --change passage-retrieval` is the next gate; do not run it
  before the steps above pass.
