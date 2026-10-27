# Change seed: work-discovery

Status: ready

- Use-case IDs: find-works
- Topic: evidence-search
- Personas: reader--researcher, agent--for-reader
- Suggested start: 3 — depends on passage-retrieval
- Handoff: handoff confirmed 2026-10-20; ready for `/givn-propose`

## Accepted decisions

### The product owns the cited answer (Q1)

- Context: The boundary decides what the product guarantees. This seed is an
  observing goal below the answer: it ranks works and returns no prose.
- Options:
  1. **Evidence with bounded synthesis** — the product returns evidence.
     Gain: small contracts. Cost: the reader needs an agent.
  2. **The answer as a product goal** — the product composes.
     Gain: usable without an agent. Cost: answer quality is product responsibility.
- Chosen: **Option 2** for the product overall; this seed remains evidence-only
  by its own boundary. Rejected: Option 1 as the product boundary.

### The classification is called Genre (Q2)

- Context: The research called it "Domäne"; the project uses "domain" for the
  problem area.
- Options: **WorkKind**, **DomainClass**, **Genre**.
- Chosen: **Genre**. Rejected: WorkKind because the user preferred the
  established word; DomainClass because it keeps two meanings.

## Open questions

None.
