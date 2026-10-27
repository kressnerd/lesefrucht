# Change seed: passage-retrieval

Status: proposed by passage-retrieval

- Use-case IDs: find-passages
- Topic: evidence-search
- Personas: agent--for-reader, reader--researcher
- Suggested start: 1 — no dependency; every other observing seed builds on it
- Handoff: handoff confirmed 2026-10-20; ready for `/givn-propose`

## Accepted decisions

### The product owns the cited answer (Q1)

- Context: The supplied research described both cited passage search and
  synthesis tools. The boundary decides whether the product guarantees answer
  quality or only evidence fidelity; this seed is on the evidence side of it.
- Options:
  1. **Evidence with bounded synthesis** — the product returns passages and
     cited summaries, the agent composes the answer.
     In practice: the agent calls `search_passages` and writes the prose itself.
     Gain: small contracts, no answer-quality guarantee.
     Cost: the reader always needs an agent.
  2. **The answer as a product goal** — the product composes the cited answer.
     In practice: the reader asks in the CLI and receives a cited answer.
     Gain: usable without an agent.
     Cost: answer quality becomes product responsibility.
- Chosen: **Option 2** because the reader must be able to use the library
  without an MCP client. Rejected: Option 1 because the web surface would stay a
  passage list.

### The classification is called Genre (Q2)

- Context: The research called it "Domäne" and stored it as `domain.<class>`,
  while the project already uses "domain" for the problem area.
- Options:
  1. **WorkKind** — tag `workkind.<class>`.
     Gain: unambiguous. Cost: deviates from the supplied document.
  2. **DomainClass** — keep `domain.<class>`.
     Gain: no rename. Cost: permanent ambiguity.
  3. **Genre** — tag `genre.<class>`.
     Gain: unambiguous, familiar. Cost: literary connotation.
- Chosen: **Genre** because it is unambiguous against "domain". Rejected:
  WorkKind because the user preferred the established word; DomainClass because
  it keeps two meanings of "domain" forever.

### Terms and aggregates of the topic

The twelve domain terms and the four aggregates are confirmed as recorded in
`givn/ideation/evidence-search/domain-terms.md` and the arc42 glossary. This
seed builds the `Work`, `Edition` and `Question` aggregates on the retrieval
side.

## Open questions

None.
