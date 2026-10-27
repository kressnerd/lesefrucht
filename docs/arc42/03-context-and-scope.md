# 3. Context and Scope

## Business context

The product sits between a private calibre library and the agent that answers
for the reader. It reads the library, derives an index, and serves evidence.

```mermaid
flowchart LR
  Reader(("Reader"))
  Agent(("Agent for the Reader"))
  Custodian(("Library custodian"))
  LH(["lesefrucht"])
  Calibre(["calibre content server"])
  Books[("Library editions")]
  Viewer["Viewer (calibre://)"]

  Reader -->|"asks, checks evidence"| Agent
  Agent -->|"claim, scope, limit; citation"| LH
  LH -->|"catalogue and edition files"| Calibre
  Calibre --- Books
  LH -->|"citation to open"| Viewer
  Viewer -->|"exact position"| Reader
  Custodian -->|"metadata decisions (not in this change)"| Calibre
```

| Partner / User | Input to system | Output from system |
|---|---|---|
| Agent for the Reader | A claim or phrase in one language, an optional scope (genre, language, authors, year range, works), a requested number of passages, and later a citation to widen | Up to the requested number of candidate passages (at most 20) with citation, score and expansion used; surrounding text with exact bounds; `no evidence` with its reason; an unavailability report |
| Reader | Nothing directly in this change — the reader is served through the agent | Via the agent: citations the reader can open in the viewer |
| calibre content server | Catalogue listing and edition files | Requests for the catalogue and for edition bytes |
| OpenAI-compatible endpoint | Embeddings for passage text and for the search claim | Passage texts, batched |
| Viewer (`calibre://`) | A citation | The opened edition at the exact position |

## Technical context

```mermaid
flowchart LR
  Agent(("MCP client"))
  LH(["lesefrucht"])
  CS(["calibre content server<br/>HTTP"])
  EP(["OpenAI-compatible endpoint<br/>HTTP"])
  FS[("Derived index<br/>local files")]

  Agent -->|"MCP / JSON-RPC over Streamable HTTP"| LH
  LH -->|"GET /ajax/books, GET /get/&lt;fmt&gt;/&lt;id&gt;"| CS
  LH -->|"POST /embeddings"| EP
  LH -->|"read and write"| FS
```

| Interface | Technology / Protocol | Direction |
|---|---|---|
| Agent surface | MCP (JSON-RPC 2.0) over Streamable HTTP, single endpoint path | in |
| Library catalogue and edition files | HTTP GET against the calibre content server | out |
| Embeddings | HTTP POST `/embeddings`, OpenAI-compatible JSON | out |
| Citation opening | `calibre://` viewer link plus exact position data | out (to the reader's viewer) |
| Index | Local derived files, rebuildable from the library | internal persistence |

Scope of this change: the two agent actions of `find-passages` and the indexing
subfunction that makes them possible. Outside the boundary: composing answers,
ranking works, comparing or summarizing works, classification and metadata
review, the glossary's changing cases, the web surface and any reader-facing
CLI.
