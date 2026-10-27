# Change seed: cited-answer

Status: ready

- Use-case IDs: answer-question
- Topic: evidence-search
- Personas: reader--researcher, agent--for-reader, rights--holder
- Suggested start: 2 — depends on passage-retrieval
- Handoff: handoff confirmed 2026-10-20; ready for `/givn-propose`

## Accepted decisions

### The product owns the cited answer (Q1)

- Context: The boundary decides whether the product guarantees answer quality or
  only evidence fidelity.
- Options:
  1. **Evidence with bounded synthesis** — the agent composes.
     Gain: small contracts. Cost: the reader needs an agent.
  2. **The answer as a product goal** — the product composes.
     Gain: usable without an agent. Cost: answer quality is product responsibility.
- Chosen: **Option 2** because the reader must not depend on an MCP client.
  Rejected: Option 1 because the web surface would stay a passage list.

### The classification is called Genre (Q2)

- Context: The research called it "Domäne"; the project uses "domain" for the
  problem area.
- Options: **WorkKind**, **DomainClass**, **Genre**.
- Chosen: **Genre** because it is unambiguous against "domain". Rejected:
  WorkClass/WorkKind because the user preferred the established word;
  DomainClass because it keeps two meanings.

### Answer in the question's language, quotations in the source language (Q6)

- Context: The library is German and English; a German question can be answered
  from an English passage, and the response must stay checkable.
- Options:
  1. **Quotations in their source language** — the answer is in the question's
     language, each quotation in its own.
     Gain: the quotation is exactly the text in the book. Cost: a bilingual answer.
  2. **Translated with the original alongside** — readable end to end.
     Gain: readable. Cost: a translation per quotation, a second failure mode.
  3. **Translated only** — shortest answer.
     Gain: short. Cost: the quotation cannot be checked against the book.
- Chosen: **Option 1** because a translation would put a second failure mode
  between the reader and the evidence. Rejected: Option 2 because the accuracy
  cost is unmeasured; Option 3 because a translation error would be
  indistinguishable from an invented citation.

## Open questions### The acceptance run judges each answer sentence (Q3)

- Context: The product composes prose, and AK-1 to AK-12 measure retrieval,
  citation accuracy, classification, latency and abstention — none measures
  whether an answer is supported by the passages it carries.
- Options:
  1. **Per-sentence coverage check by a judge** — 50 answers per run, each
     sentence labelled covered or uncovered by the supplied passages.
     Gain: measures the guarantee, catches wrongly attached citations.
     Cost: judge calibration and model calls.
  2. **Structural citation requirement** — every sentence carries an identifier.
     Gain: free and deterministic. Cost: a wrong citation passes.
  3. **Human sample only** — 20 answers read manually.
     Gain: no judge setup. Cost: not regression-capable.
- Chosen: **Option 1** because it measures the guarantee and extends to new
  response shapes; the judge sees only the question, the supplied passages and
  the answer; a one-time human-labelled sample of about 100 sentences calibrates
  it; runs are offline on recorded artifacts, fixture recordings in CI and
  real-library recordings on the host. Rejected: Option 2 because R1 already
  enforces it and it cannot detect a wrongly attached citation; Option 3
  because it is not regression-capable and does not scale with prompt or model
  changes.

## Open questions

None.
