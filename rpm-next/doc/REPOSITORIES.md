# Repositories

The `UnifiedRepositoryManager` coordinates multiple package sources to provide a seamless cross-platform experience.

## Supported Adapters

### APT (Debian/Ubuntu/Pop!_OS)

- **File**: `apt.rs`
- Deals with `dists/` hierarchies and `Packages` files.
- Includes specific optimization for **Pop!_OS** repositories including Proprietary and CUDA sources.
- Maps Pop!_OS releases (e.g., 22.04) to Ubuntu base codenames (e.g., jammy).

### DNF (Fedora/RHEL)

- **File**: `dnf.rs`
- Supports XML-based repodata (`repomd.xml`).

### Pacman (Arch Linux)

- **File**: `pacman.rs`
- Parses standard Arch repository databases.

### Winget (Windows)

- **File**: `winget.rs`
- Interfaces with Microsoft's community repository manifests.

### Android (Play Store/F-Droid)

- **File**: `playstore.rs`
- Focused on F-Droid repository parsing and APK metadata.

## Priority Order

When a package is available in multiple repositories, the priority (lower number = higher priority) is:

1. Native Redox
2. Pacman (Arch)
3. APT (Debian/Ubuntu)
4. DNF (Fedora)
5. Winget (Windows)
6. Android (F-Droid)
