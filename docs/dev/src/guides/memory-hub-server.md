# Build a memory-backed HTTP hub server

This guide shows how to assemble a **local, ephemeral** hub server with an HTTP
API from existing workspace crates. It is documentation-only — no binary crate
is shipped in the repo.

Use this for local development, manual protocol testing, or understanding how
`track-hub-memory` wires components together.

> **Limitation:** An in-memory hub is **not durable**. It does not satisfy
> HUB-CONF restart cases and is unsuitable for production. See
> [ADR 0005](../../adr/0005-hub-implementation-conformance.md) ephemeral class.

## Stack overview

```text
InMemoryHubService
  → Arc<dyn HttpHubService>
    → HubHttpServer
      → Axum router (track-hub-http)
        → HTTP clients (track-sync HttpTransport, curl, etc.)
```

Canonical reference: [`TestHubHandle`](../../crates/track-hub-memory/src/test_hub_handle.rs).

## 1. Dependencies

In your application's `Cargo.toml`:

```toml
[dependencies]
track-hub = { workspace = true }
track-hub-http = { workspace = true }
track-id = { workspace = true }
tokio = { workspace = true, features = ["macros", "rt-multi-thread", "signal"] }
```

Optional: depend on `track-hub-memory` and use `TestHubHandle` directly instead
of writing `main` yourself.

## 2. Minimal server sketch

The following derives from `TestHubHandle::start_with`:

```rust
use std::net::SocketAddr;
use std::sync::Arc;

use track_hub::InMemoryHubService;
use track_hub_http::HubHttpServer;
use track_id::TrackUlid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_uuid = TrackUlid::new(); // or parse from CLI/env
    let addr: SocketAddr = "127.0.0.1:8080".parse()?;

    let hub = Arc::new(InMemoryHubService::new());
    let hub_http: Arc<dyn track_hub_http::HttpHubService> = hub.clone();

    let server = HubHttpServer::bind(addr, workspace_uuid, hub_http).await?;

    println!("Hub listening on {}", server.base_url);

    // Graceful shutdown on Ctrl+C
    tokio::signal::ctrl_c().await?;
    server.shutdown().await?;

    Ok(())
}
```

For dynamic port assignment, bind `127.0.0.1:0` via `TcpListener` and use
`HubHttpServer::serve` (see `TestHubHandle`).

## 3. Authentication

Default: allow-all auth (`InMemoryHubService::new()`).

Restrict actors for testing auth scenarios:

```rust
use track_hub::{ActorAllowlistAuthorizer, InMemoryHubService};

let authorizer = Arc::new(ActorAllowlistAuthorizer::new(&["alice", "bob"]));
let hub = Arc::new(InMemoryHubService::with_authorizer(authorizer));
```

## 4. Node registration

Clients must register nodes before push (ADR 0004 §Node registry). After
creating the hub, register each authoring node:

```rust
hub.register_node(workspace_uuid, node_uuid).await?;
```

`TestHubHandle` exposes the inner `Arc<InMemoryHubService>` as `.hub` for this
purpose in tests.

## 5. Verify with a client

### Option A: track-sync

Use `HttpTransport` and `SyncEngine` from `track-sync` pointed at
`server.base_url`. This exercises the full push/pull pipeline.

### Option B: track-hub-memory in tests

```rust
let handle = track_hub_memory::TestHubHandle::start(workspace_uuid).await?;
// handle.base_url — use in HttpTransport or reqwest
handle.shutdown().await?;
```

### Option C: raw HTTP

Push and pull routes follow [ADR 0004](../../adr/0004-hub-sync-protocol-and-compaction.md)
and SRD Appendix D. Include the protocol version header
(`TRACK_PROTOCOL_VERSION_HEADER`).

## 6. Push stream observer (optional)

Tests that inspect NDJSON push streams can attach an observer:

```rust
use track_hub_http::{HubHttpServer, PushStreamObserver};

HubHttpServer::bind_with_observer(addr, workspace_uuid, hub, Some(observer)).await?;
```

`track-hub-memory` uses `InMemoryPushObserver` for this pattern.

## 7. Using TestHubHandle directly

If you do not need a standalone binary, the embeddable test hub is already
available:

```rust
use track_hub_memory::TestHubHandle;
use track_id::TrackUlid;

let workspace = TrackUlid::new();
let handle = TestHubHandle::start(workspace).await?;
let url = handle.base_url.clone();
// … run tests against url …
handle.shutdown().await?;
```

This is what `track-sync-testing::MemoryHubFixture` uses for CI.

## Limitations

| Topic | In-memory hub behaviour |
| --- | --- |
| Durability | State lost on process exit |
| HUB-CONF | Not applicable (ephemeral class) |
| Compaction across restart | Not tested for durability |
| Production use | Not recommended — use a future durable backend |

For production deployment patterns, see [infra/README.md](../../../infra/README.md)
and SRD §5.2.

## Related

- [track-hub-http](../crates/track-hub-http.md)
- [track-hub-memory](../crates/track-hub-memory.md)
- [Implement a new hub service](./new-hub-implementation.md)
