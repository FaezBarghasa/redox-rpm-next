# Changelog - redox-store

All notable changes to the `redox-store` project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-23

### Added
- Initial release of `redox-store` backend server.
- Built-in HTTP server using `hyper` 1.3 and `tokio` multi-thread runtime.
- REST API `/api/assets` and `/api/assets/:id` for listing AI models and game assets.
- Streaming file downloader under `/download/*` endpoint.
- Built-in path traversal security checks blocking `..` and relative traversal attacks.
- Mock data loader pre-populating Llama 3 8B Instruct model and Forge Vulcan Tech Demo assets.
- Structured logging using `tracing` and `tracing-subscriber`.
- `forbid(unsafe_code)` enforcement.
