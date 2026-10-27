# Change seed: genre-classification

Status: ready

- Use-case IDs: classify-works
- Topic: evidence-search
- Personas: custodian--metadata, upstream--calibre
- Suggested start: 5 — depends on passage-retrieval for indexed text
- Handoff: handoff confirmed 2026-10-20; ready for `/givn-propose`

## Accepted decisions

### The classification is called Genre (Q2)

- Context: The research called it "Domäne" and stored it as `domain.<class>`;
  the project uses "domain" for the problem area.
- Options:
  1. **WorkKind** — `workkind.<class>`. Gain: unambiguous. Cost: deviates.
  2. **DomainClass** — keep `domain.<class>`. Gain: no rename. Cost: ambiguity.
  3. **Genre** — `genre.<class>`. Gain: unambiguous. Cost: literary connotation.
- Chosen: **Genre**. Rejected: WorkKind because the user preferred the
  established word; DomainClass because it keeps two meanings.

### Classification stays a custodian goal (Q8)

- Context: The chain runs classification from a policy, while the custodian's
  commands are apply and reject. The cut review called the goal a policy flow.
- Options:
  1. **Keep it, restated** — the custodian determines the genre of each indexed
     work; the product proposes.
     In practice: a new work is proposed `fachbuch` and the custodian confirms.
     Gain: classification keeps its own contract and the AK-4 acceptance.
     Cost: the same write path appears in review for the genre field.
  2. **Demote it to a subfunction** of review.
     Gain: one write path. Cost: classification rules and AK-4 live inside the
     review contract.
- Chosen: **Option 1** because AK-4 needs a contract of its own. Rejected:
  Option 2 because every classification change would reopen review.

### The product owns the cited answer (Q1)

- Context: Classifying and writing metadata is the changing side of the boundary.
- Chosen: **Option 2** for the product, with the catalog authority left to
  calibre and the custodian. Rejected: Option 1 as the boundary.

## Open questions### The classification writes above a confidence threshold (Q4)

- Context: The supplied research allowed an autonomous first run into empty
  fields; the Frame required human confirmation. Both cannot hold, and the
  success guarantee of this case depends on which rule applies.
- Options:
  1. **Proposal only** — every write waits for confirmation.
     In practice: every new work appears in the inbox; nothing changes until
     confirmed. Gain: one rule, no invisible writes. Cost: three thousand
     items at setup, no usable genre filter until then.
  2. **Autonomous first run into empty fields** — later runs propose.
     Gain: classified after one run. Cost: thousands of decisions nobody made.
  3. **Autonomous above a confidence threshold** — confident results are
     written; uncertain ones become proposals.
     Gain: bounded exposure, the inbox holds the uncertain cases.
     Cost: the threshold needs calibration and two write paths.
- Chosen: **Option 3** because it bounds unreviewed writes to the reliable
  cases; the custodian sets the threshold and can override any write.
  Rejected: Option 1 because the library would start with three thousand inbox
  items; Option 2 because the custodian would inherit decisions nobody made.

## Open questions

None.
