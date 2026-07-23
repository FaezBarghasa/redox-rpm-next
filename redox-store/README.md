# `redox-store`

`redox-store` is a high-concurrency storage backend server designed to host and serve AI models (e.g., GGUF binaries) and Game Assets (e.g., Vulkan demos and packages) for the Redox OS ecosystem.

## Features

- **Asynchronous HTTP/1.1 Engine**: Powered by `tokio` multi-thread runtime and `hyper` HTTP server stack.
- **RESTful Metadata Endpoints**: Query hosted asset lists, model details, versions, and SHA256 checksums via JSON APIs.
- **Asynchronous Asset Downloads**: Streams binary assets (`/download/*`) asynchronously using Tokio file I/O with path traversal protection.
- **Thread-Safe In-Memory State**: Manages asset metadata dynamically with `Arc<RwLock<HashMap>>`.
- **Health Verification**: Exposes `/health` endpoint for monitoring system status.

## REST API Specifications

| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/api/assets` | Returns a list of all registered assets (JSON array). |
| `GET` | `/api/assets/:id` | Returns metadata details for a specific asset. |
| `GET` | `/download/*path` | Downloads asset payload from the storage directory. |
| `GET` | `/health` | Health check endpoint (Returns `OK` with HTTP 200). |

## Environment Variables

- `STORE_ROOT`: Path to the asset storage directory (Default: `./store_data`).
- `PORT`: Port to listen on (Default: `8080`).

## Running the Server

```bash
# Start server with default port (8080) and storage path (./store_data)
cargo run --bin redox-store

# Run on custom port and root directory
PORT=9090 STORE_ROOT=/var/redox-store cargo run --bin redox-store
```
