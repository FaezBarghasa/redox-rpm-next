# RPM-Next Documentation

RPM-Next is a universal package manager for RedoxOS, designed to support multiple package formats and external repositories seamlessly.

## Architecture

RPM-Next follows a modular architecture consisting of the following components:

- **Core Package Manager (`main.rs`)**: Orchestrates the installation, removal, and upgrading of packages.
- **Repository Manager (`repository.rs`, `UnifiedRepositoryManager`)**: Manages multiple repository sources (APT, DNF, Pacman, etc.) and provides a unified interface for searching and syncing.
- **Dependency Resolver (`resolver.rs`)**: Implements a SAT-based dependency resolution engine to ensure all package requirements are met.
- **Format Adapters**:
  - `deb.rs`: Debian/Ubuntu package format support.
  - `rpm.rs`: Red Hat/Fedora package format support.
  - `pkg.rs`: Redox native package format support.
- **External Repository Adapters**:
  - `apt.rs`: Connects to Debian/Ubuntu APT repositories (includes Pop!_OS support).
  - `dnf.rs`: Connects to Fedora/RHEL DNF repositories.
  - `pacman.rs`: Connects to Arch Linux Pacman repositories.
  - `winget.rs`: Connects to Windows Winget repositories.
  - `playstore.rs`: Connects to Android F-Droid/Play Store.

## Features

- **Multi-Format Support**: Install `.deb`, `.rpm`, and native `.pkg.tar.zst` packages.
- **Cross-Platform Repositories**: Search and sync from diverse sources including APT, DNF, and Winget.
- **Dependency Resolution**: Automatic calculation of dependency trees and conflict detection.
- **Transaction-Based Operations**: Atomic package operations with rollback capabilities (planned).
- **Pop!_OS Integration**: First-class support for System76 Pop!_OS repositories (Main, Proprietary, and CUDA).

## Usage

### Synchronization

```bash
rpm-next sync
```

### Searching

```bash
rpm-next search <query>
```

### Installation

```bash
rpm-next install <package_name>
```

### Viewing Package Info

```bash
rpm-next info <package_name>
```

## Internal Synchronization

The `UnifiedRepositoryManager` ensures that queries are handled by the most appropriate source based on a predefined priority order: Native > Pacman > APT > DNF > Winget > Android.
