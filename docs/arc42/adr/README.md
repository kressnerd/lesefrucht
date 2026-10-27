# Architecture Decision Records

One architecturally significant decision per ADR file. The chapter-09 register
(`../09-architecture-decisions.md`) and this index link to the same files and
must stay in sync.

The authoring template is [`adr-template.md`](adr-template.md); archived
historical records live under [`archive/`](archive/).

## Register

| ADR | Title | Status | Date |
|---|---|---|---|
| [ADR-0001](ADR-0001-embedded-hybrid-index.md) | Embedded hybrid index built from Tantivy, usearch and SQLite | accepted | 2026-10-27 |
| [ADR-0002](ADR-0002-mcp-streamable-http-transport.md) | Agent interface as MCP over Streamable HTTP using the official Rust SDK | accepted | 2026-10-27 |

## Notes

- Sorted by ADR number; numbers are never reused.
- Moving a record to `archive/` preserves its number and requires an index
  update in both this file and chapter 9.
- A refinement of an existing decision amends that ADR; it does not add a
  parallel file.
