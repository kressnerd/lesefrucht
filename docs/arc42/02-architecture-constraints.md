# 2. Architecture Constraints

## Technical constraints

| Constraint | Motivation |
|---|---|
| Product code is written in Rust (stable) | Given constraint of the topic (`givn/ideation/evidence-search/frame.md`, second supplied document). Python is permitted only as a throwaway spike tool. |
| The calibre content server is the only calibre interface | Given constraint. The product reads the catalogue and edition files through the documented content-server HTTP interface and never reads `metadata.db`; writes, when they arrive, go through `calibredb` against the server URL. |
| Metadata authority stays with calibre | The product may propose; the catalogue remains binding. This change therefore never writes. |
| Model access is OpenAI-compatible endpoints | Given constraint. No vendor SDK is assumed. |
| No default cloud endpoint and no telemetry carrying book text | Confirmed boundary of `rights--holder`. Every outbound endpoint (`library.base_url`, `embedding.base_url`) is required configuration; startup fails when one is missing. |
| No mandatory container in production | Given non-goal. The service runs directly on the shared NixOS host (8 cores, 64 GB RAM, GPU). Containers are test and local-development infrastructure only. |
| One language per request | The domain contract requires the request to be expressed in one language; cross-language reach is achieved by expansion, not by mixed-language input. |
| Offsets must be reproducible | A citation's exactness depends on stable text normalization and stable offsets; this forbids an extraction or chunking step whose output varies between runs. |
| MCP is the agent-facing contract | The agent surface is the Model Context Protocol over Streamable HTTP; no reader-facing CLI or web surface exists in this change. |
| Library size is "a few thousand books" | Given operating envelope. Index size, embedding throughput and query latency are designed for a personal library, not a multi-tenant corpus. |

## Organisational constraints

| Constraint | Motivation |
|---|---|
| One person, part-time, no deadline | Architecture must be operable by a single owner: no cluster, no managed service, no on-call rotation. |
| Copyleft licences are permitted, but every dependency must pass a licence allowlist | Given constraint. The concrete licence of the product remains open and is decided through `givn/config.yaml`'s dependency policy, not by this change. |
| The givn artifact sequence is the process | proposal → specs → design → design-review → tasks → review → archive. Architectural documentation is a first-class artifact (arc42 addon enabled and gating archive). |
| Behaviour lives in executable Gherkin | The `.feature` files are the specification; a parallel hand-written test suite for the same behaviour is not permitted. |
| Change-scoped work | Each change carries one guarantee. This change does not pre-build the answer surface, the web surface or the classification flow. |

## Conventions

| Convention | Motivation |
|---|---|
| Mermaid only for diagrams | ASCII diagrams are unreadable in review and are rejected by the documentation addon. |
| MADR for architecture decisions | One decision per file under `docs/arc42/adr/`; chapter 9 is the register only. |
| Domain vocabulary from the arc42 glossary | Specs, design, code symbols and documentation use the same terms (`Work`, `Edition`, `Passage`, `Citation`, `Question`, `Genre`, `Index`, `Library`, `Abstention`). |
| Tagged results over exceptions on the agent surface | The consumer must distinguish `no evidence` from failure; the contract therefore carries a tagged outcome, and protocol errors are reserved for malformed requests. |
| Version freshness over remembered versions | Every version-bearing decision records either "latest/LTS" as the resolution strategy or a looked-up version with its source and date. |
