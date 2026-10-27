# Change seed: metadata-review

Status: ready

- Use-case IDs: review-metadata-proposals
- Topic: evidence-search
- Personas: custodian--metadata, upstream--calibre
- Suggested start: 6 — depends on genre-classification for proposals to decide
- Handoff: handoff confirmed 2026-10-20; ready for `/givn-propose`

## Accepted decisions

### The classification is called Genre (Q2)

- Context: Review decides proposals on calibre fields, including the genre tag.
- Options: **WorkKind**, **DomainClass**, **Genre**.
- Chosen: **Genre**, with the tag form `genre.<class>`. Rejected: WorkKind and
  DomainClass for the reasons in the topic record.

### Classification stays a custodian goal (Q8)

- Context: Review and classification share the genre write path.
- Options: keep classification as a goal; demote it to a subfunction of review.
- Chosen: **keep**, so review stays about deciding proposals and does not absorb
  the classification rules. Rejected: demote for the same reason.

## Open questions### Writes go through one persistent queue (Q5)

- Context: The product writes through the content server while the desktop may
  hold the same library; between the read and the write another writer can
  change the field. AK-7 demands no corruption and no error.
- Options:
  1. **Re-read immediately before the write, abort on mismatch.**
     Gain: cheap, catches the ordinary cases. Cost: the window stays open.
  2. **Serialize every product write through one persistent queue.**
     Gain: retries survive a restart, order is deterministic inside the
     product. Cost: orders only the product's own writes.
  3. **Refuse writes while a desktop session is detected.**
     Gain: strongest protection. Cost: needs a signal calibre may not offer.
- Chosen: **Option 2** because AK-7 must hold across restarts and a conflict
  can be retried; the expected-value check still catches a desktop change and a
  conflict re-bases the proposal. Rejected: Option 1 because the window stays
  open and a failure has no retry; Option 3 because it blocks writes while
  reading and depends on an unreliable signal.

## Open questions

None.
