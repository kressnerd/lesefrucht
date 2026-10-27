# Persona: Maintainer of the calibre interface

## Stable core

- Role: The upstream maintainer whose documented interfaces the product depends
  on.
- Authority boundary: Decides which calibre interfaces stay supported; the
  product decides nothing about them.
- Evaluation capability: Can judge whether the product stays inside documented
  contracts or leans on internals.
- Own vocabulary: content server, `calibredb`, `calibre://`. Refuses "plug-in
  API" as a stable interface where no stability promise exists.
- Success criterion: A calibre upgrade does not break the product, and the
  product never reads `metadata.db` behind the server's back.

## Running record

- 2026-10-20 — proposed in the ideation topic "evidence-search"; confirmed by
  the user.

## Topic stances

- 2026-10-20 — evidence-search: only documented contracts; reading `metadata.db`
  directly is out even where it would be easier.
