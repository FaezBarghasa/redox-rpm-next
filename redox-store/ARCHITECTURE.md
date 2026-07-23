# Architecture - redox-store

`redox-store` is designed as a lock-free read-heavy HTTP asset distribution server using Tokio tasks.

## System Architecture

```text
[ Client Requests ]
        │
        ▼
┌─────────────────────────┐
│     hyper / tokio       │
│  Async TCP Listener     │
└───────────┬─────────────┘
            │
            ▼
┌─────────────────────────┐
│     handle_request      │
│  (HTTP Method & Router) │
└───────────┬─────────────┘
            │
  ┌─────────┴────────────────┬────────────────────────┐
  ▼                          ▼                        ▼
┌────────────────────┐   ┌────────────────────┐   ┌───────────────────────┐
│ GET /api/assets    │   │ GET /download/*    │   │ GET /health           │
│ Query Metadata DB  │   │ Asynchronous File  │   │ Returns 200 OK Status │
│ (RwLock Read Lock) │   │ Stream Reader      │   │                       │
└────────────────────┘   └────────────────────┘   └───────────────────────┘
```

## Core Components

1. **`StoreState`**:
   - Holds reference to root directory (`root_dir: PathBuf`).
   - Maintains an in-memory database of asset metadata wrapped in `tokio::sync::RwLock<HashMap<String, AssetMetadata>>`.
2. **Request Router (`handle_request`)**:
   - Zero-dependency custom path match router mapping HTTP paths to handler functions.
   - Prevents directory traversal attacks by validating path sanitization rules (`..` or `//` rejection).
3. **Async File Streaming (`serve_file`)**:
   - Opens target files asynchronously via `tokio::fs::File` and constructs binary response streams (`application/octet-stream`).
