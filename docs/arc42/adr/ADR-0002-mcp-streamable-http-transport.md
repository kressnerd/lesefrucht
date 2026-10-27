---
status: accepted
date: 2026-10-27
decision-makers: project operator (owner of the library and this repository)
consulted: none
informed: none
---

# Agent interface as MCP over Streamable HTTP using the official Rust SDK

## Qualification

ADR-QUALIFICATION: PASS

### Qualification verdict

qualification: QUALIFIED
alternatives: PASS
architectural_impact: PASS
durable_consequence: PASS
lower_level_artifact: PASS
existing_adr_check: PASS
must_be_shared: SUPPORTING
routing: NEW_ADR
canonical_artifact: null
target_adr: null
replacement_adr: null

Evidence:

- alternatives: MCP over Streamable HTTP, MCP over stdio, a bespoke REST/JSON
  API, and a hand-rolled MCP JSON-RPC transport are four materially different
  options, each with a stated gain and cost.
- architectural_impact: the choice fixes the external contract surface, the
  process model (a long-running listener rather than a per-call subprocess), the
  compatibility boundary with MCP clients, and the deployment exposure (loopback
  bind, no authentication in this slice).
- durable_consequence: changing transport later breaks every connected agent and
  the e2e driver, and requires both a server and a client rewrite; the SDK also
  pins the protocol revisions the product speaks.
- lower_level_artifact: the tool schemas and result shapes are owned by
  `design.md` (MCP Contract) and the behaviour by the Gherkin; the transport, process model
  and compatibility boundary are not fully ownable there because future surfaces
  (the cited answer, the web surface) must reuse the same server and negotiate
  the same revisions.
- existing_adr_check: no active or archived ADR covers the agent interface
  (register empty, `docs/arc42/adr/` held only the template); route is
  `NEW_ADR`.
- must_be_shared: everyone adding a tool must route it through this server and
  keep the tagged-result contract, or agents receive an incompatible surface.

### MADR checklist evidence

- [x] **Hard to reverse** - Reverting later would cost significant effort
      because every consumer, the e2e driver and the readiness path are written
      against this transport and session model.
- [x] **Shapes quality attributes** - It materially affects interoperability and
      security exposure because a network listener without authentication is a
      different risk posture than a stdio subprocess.
- [x] **Structural** - It defines a boundary, contract, or dependency direction
      because the MCP layer becomes the only entry point into retrieval, and the
      product becomes a server rather than a command.
- [x] **Must be shared** - Developers unfamiliar with it would systematically
      violate the architecture because they would add a second entry point or
      return errors where the contract requires a tagged abstention.

## Context and Problem Statement

The agent for the reader is an LLM client that searches and reads on the
reader's behalf and decides its own tool usage. The product must expose both
actions of `find-passages` to such a client with a machine-readable, tagged
result, and it must remain usable when a second surface (the cited answer, the
web surface) arrives. The choice covers `mcp::server`, `mcp::tools`, the readiness
path and the e2e driver.

## Decision Drivers

- A real protocol contract instead of a bespoke one, because the consumer is a
  general-purpose agent.
- One process serving several clients over the life of the library.
- The same surface must carry a readiness probe for local operation.
- No hand-maintained protocol implementation if an official SDK exists.
- Loopback by default: the library is private and this slice adds no
  authentication.

## Considered Options

- MCP over Streamable HTTP, implemented with the official Rust SDK (`rmcp`)
- MCP over stdio, implemented with the same SDK
- A bespoke REST/JSON API
- A hand-rolled MCP JSON-RPC transport over HTTP

## Decision Outcome

Chosen option: "MCP over Streamable HTTP with the official Rust SDK", because it
is the only option that gives a standard agent contract, a long-running server
and a single implementation of session handling without writing protocol code.

### Consequences

- Good, because any MCP client can connect, and the SDK carries capability
  negotiation, session ids and SSE framing.
- Good, because one listener serves the tool surface and `/healthz`, so local
  operation and tests share a process model.
- Bad, because sessions live in memory, so a restart forces every client to
  re-initialize and a client that caches a session id must recover.
- Bad, because a network listener without authentication must stay on loopback
  until an authentication decision is made; exposing it is now a security
  question rather than a configuration change.
- Bad, because protocol revisions follow the SDK's release cadence, so the
  product inherits its upgrade rhythm.
- Bad, because SSE framing adds a failure mode that a stdio pipe does not have,
  and the e2e driver must parse through the SDK rather than a raw HTTP body.

### Confirmation

The two `@e2e` scenarios drive the product exclusively through the rmcp client
over Streamable HTTP (`tests/steps/passage_retrieval_e2e.rs`), and
`./run-tests.sh --prove-strictness` plus the e2e run prove that an undefined step
or a broken session fails the build. A review checkpoint confirms that no route
other than `/mcp` and `/healthz` is exposed and that the default bind is
loopback.

## Pros and Cons of the Options

### MCP over Streamable HTTP with `rmcp`

- Good, because it is the standard agent interface and the SDK is official.
- Good, because several clients can use one server, and a second surface can
  reuse it.
- Neutral, because it requires a port and therefore a bind-address decision.
- Bad, because it makes a network listener part of the product's surface and
  inherits the SDK's protocol cadence.

### MCP over stdio with `rmcp`

- Good, because it needs no port, no bind address and no authentication story.
- Good, because the SDK handles the protocol identically.
- Bad, because one process serves one client for its lifetime, which does not fit
  a library server that outlives a conversation.
- Bad, because readiness and remote agents cannot be reached through it.

### A bespoke REST/JSON API

- Good, because the shapes could be designed freely.
- Bad, because every agent integration would need custom prompting or glue, and
  the project would own a contract no client understands natively.
- Bad, because it discards capability negotiation and session semantics that
  MCP already standardises.

### A hand-rolled MCP JSON-RPC transport

- Good, because it avoids an SDK dependency and its release cadence.
- Bad, because session ids, SSE framing, capability negotiation and error shapes
  would all be re-implemented and re-tested.
- Bad, because the e2e driver would need the same hand-rolled client, doubling
  the surface to maintain.

## More Information

The tool schemas, tagged results and error taxonomy are specified in
`givn/changes/passage-retrieval/design.md` (MCP Contract). The unauthenticated-listener risk
is tracked in arc42 chapter 11.
