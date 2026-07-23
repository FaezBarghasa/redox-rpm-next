# Changelog - rpm-next

All notable changes to the `rpm-next` project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-23

### Added
- Multi-ABI universal package manager architecture.
- Package format adapters for Native (`.pkg`), Debian (`.deb`), Red Hat (`.rpm`), Alpine (`.apk`), Windows (`.msi`/`.msix`), and Android (`.apk`).
- Unified repository manager (`UnifiedRepositoryManager`) supporting APT, DNF, Pacman, Winget, and PlayStore/F-Droid integrations.
- Local JSON database engine (`PackageDatabase`) tracking file ownership and package metadata (`installed.json`).
- Transaction engine (`Transaction`) for safe batch execution of install, remove, and upgrade operations.
- Version comparison algorithm supporting semantic version ordering.
- Dependency checking preventing removal of required packages.
