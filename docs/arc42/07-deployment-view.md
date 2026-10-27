# 7. Deployment View

## Infrastructure level 1

```mermaid
flowchart TB
  subgraph host["NixOS host (8 cores, 64 GB RAM, GPU)"]
    SVC["systemd unit: lesefrucht"]
    DATA[("data/<br/>sqlite + tantivy + usearch + text")]
    CAL["calibre content server"]
    VIEW["calibre viewer"]
    SVC --- DATA
    SVC -->|"HTTP"| CAL
  end
  AGENT(("Agent / MCP client"))
  EP(["Configured OpenAI-compatible endpoint"])

  AGENT -->|"MCP over Streamable HTTP<br/>127.0.0.1:8787"| SVC
  SVC -->|"POST /embeddings"| EP
  VIEW -->|"opens calibre:// citation"| DATA
```

| Node / Environment | Description |
|---|---|
| NixOS host | Production and development machine of the single owner. The service runs as a systemd unit, not in a container. |
| `data/` directory | The derived index: SQLite database, Tantivy directory, usearch file, normalized text. Rebuildable from the library; not a backup target. |
| calibre content server | Runs on the same host and is the only calibre interface. The desktop application may run in parallel. |
| Configured embedding endpoint | Any OpenAI-compatible service the owner configures; there is no default. |
| Agent | Any MCP client the reader points at the endpoint. |

## Infrastructure level 2

### `lesefrucht` (production)

| Property | Value |
|---|---|
| Hardware / VM | Shared NixOS host, 8 cores, 64 GB RAM, GPU (unused by this change) |
| OS | NixOS, Linux |
| Middleware / Runtime | Rust binary under systemd; no container runtime required |
| Key software deployed | `lesefrucht` server, configuration at a path given on the command line, data directory on local disk |
| Network exposure | Loopback by default; the bind address is configuration |
| Readiness | `/healthz` returns 200 when the MCP endpoint is mounted and the store is open |

### Test and local stack

```mermaid
flowchart LR
  subgraph net["isolated compose network lesefrucht-e2e"]
    P["lesefrucht<br/>(instrumented in e2e)"]
    CT["calibre content-server twin<br/>+ fixture library"]
    OT["OpenAI-compatible twin<br/>deterministic embeddings"]
    RUN["features_runner<br/>(on the host)"]
    P -->|"library + editions"| CT
    P -->|"embeddings"| OT
    RUN -->|"MCP over 127.0.0.1:8787"| P
  end
```

The runner runs on the host and reaches the product through a loopback-published
port; the product reaches both twins by service name inside the network. Only
the three loopback ports are published, no dependency is shared with another
project, and no live third-party service participates. Teardown removes the
network and its volumes (`podman-compose down -v`).
