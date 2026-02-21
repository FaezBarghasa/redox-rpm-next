//! Redox-Store Backend
//! Serves AI models and Game assets with high-concurrency for the Redox ecosystem.

#![forbid(unsafe_code)]

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

/// Asset Metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub id: String,
    pub name: String,
    pub asset_type: String, // "ai_model" or "game"
    pub version: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub path: String,
}

/// Store State
pub struct StoreState {
    pub root_dir: PathBuf,
    pub metadata_db: RwLock<HashMap<String, AssetMetadata>>,
}

impl StoreState {
    pub fn new(root_dir: PathBuf) -> Self {
        Self {
            root_dir,
            metadata_db: RwLock::new(HashMap::new()),
        }
    }

    /// Load mock data for verification purposes.
    pub async fn load_mock_data(&self) {
        let mut db = self.metadata_db.write().await;
        db.insert(
            "llama-3-8b-instruct.gguf".to_string(),
            AssetMetadata {
                id: "llama-3-8b-instruct.gguf".to_string(),
                name: "META Llama 3 8B Instruct (Q4_K_M)".to_string(),
                asset_type: "ai_model".to_string(),
                version: "1.0.0".to_string(),
                size_bytes: 4_920_000_000,
                sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                    .to_string(),
                path: "models/llama-3-8b-instruct.gguf".to_string(),
            },
        );
        db.insert(
            "forge-vulcan-demo".to_string(),
            AssetMetadata {
                id: "forge-vulcan-demo".to_string(),
                name: "Forge Vulcan Tech Demo".to_string(),
                asset_type: "game".to_string(),
                version: "0.1.0".to_string(),
                size_bytes: 250_000_000,
                sha256: "4a2fd2028bb91dfa279c6560965d1bf2ee9cb8ce9c1cae8b4e7240c5e7b5f6d6"
                    .to_string(),
                path: "games/forge-vulcan-demo.pkg.tar.zst".to_string(),
            },
        );
        tracing::info!("Loaded {} mocked assets", db.len());
    }
}

/// Helper function to create empty boxed body responses
fn empty_response(status: StatusCode) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .body(Full::new(Bytes::new()))
        .unwrap()
}

/// Helper for JSON responses
fn json_response<T: Serialize>(status: StatusCode, data: &T) -> Response<Full<Bytes>> {
    match serde_json::to_string(data) {
        Ok(json) => Response::builder()
            .status(status)
            .header(CONTENT_TYPE, "application/json")
            .body(Full::new(Bytes::from(json)))
            .unwrap(),
        Err(e) => {
            tracing::error!("JSON serialization error: {}", e);
            empty_response(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Serve files asynchronously
async fn serve_file(path: &Path) -> Response<Full<Bytes>> {
    match File::open(path).await {
        Ok(mut file) => {
            let mut contents = Vec::new();
            if let Err(e) = file.read_to_end(&mut contents).await {
                tracing::error!("Error reading file {:?}: {}", path, e);
                return empty_response(StatusCode::INTERNAL_SERVER_ERROR);
            }
            Response::builder()
                .status(StatusCode::OK)
                .header(
                    CONTENT_TYPE,
                    "application/octet-stream", // Fallback, could map by extension
                )
                .header(CONTENT_LENGTH, contents.len().to_string())
                .body(Full::new(Bytes::from(contents)))
                .unwrap()
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                empty_response(StatusCode::NOT_FOUND)
            } else {
                tracing::error!("Error opening file {:?}: {}", path, e);
                empty_response(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Request Router
async fn handle_request(
    req: Request<Incoming>,
    state: Arc<StoreState>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let method = req.method();
    let path = req.uri().path();
    
    // Simple routing logic without bringing in a large router framework like Axum
    match (method, path) {
        (&Method::GET, "/api/assets") => {
            let db = state.metadata_db.read().await;
            let assets: Vec<AssetMetadata> = db.values().cloned().collect();
            Ok(json_response(StatusCode::OK, &assets))
        }
        (&Method::GET, p) if p.starts_with("/api/assets/") => {
            let id = p.strip_prefix("/api/assets/").unwrap();
            let db = state.metadata_db.read().await;
            if let Some(asset) = db.get(id) {
                Ok(json_response(StatusCode::OK, asset))
            } else {
                Ok(empty_response(StatusCode::NOT_FOUND))
            }
        }
        (&Method::GET, p) if p.starts_with("/download/") => {
            let rel_path = p.strip_prefix("/download/").unwrap();
            
            // Prevent path traversal
            if rel_path.contains("..") || rel_path.contains("//") {
                return Ok(empty_response(StatusCode::BAD_REQUEST));
            }
            
            let full_path = state.root_dir.join(rel_path);
            Ok(serve_file(&full_path).await)
        }
        // Health check endpoint
        (&Method::GET, "/health") => {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Full::new(Bytes::from("OK")))
                .unwrap())
        }
        // Catch-all 404
        _ => Ok(empty_response(StatusCode::NOT_FOUND)),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Redox-Store Server...");

    // Default storage directory
    let root_dir = std::env::var("STORE_ROOT").unwrap_or_else(|_| "./store_data".to_string());
    let root_path = PathBuf::from(&root_dir);
    tokio::fs::create_dir_all(&root_path)
        .await
        .expect("Failed to create storage directory");

    let state = Arc::new(StoreState::new(root_path));
    
    // Load some mock data so the API always returns something during integration
    state.load_mock_data().await;

    // Bind address
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Listening on http://{}", addr);

    loop {
        let (stream, remote_addr) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let state_clone = state.clone();
        
        // Spawn a new task to serve each connection securely and efficiently
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| handle_request(req, state_clone.clone())),
                )
                .await
            {
                tracing::error!("Error serving connection from {}: {:?}", remote_addr, err);
            }
        });
    }
}
