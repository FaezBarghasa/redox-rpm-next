# `rpm-next`

`rpm-next` is a universal package manager designed for multi-ABI and cross-platform Redox OS deployments. It supports native Redox packages as well as foreign package formats (Debian, RPM, Alpine, Windows, Android) through custom format adapters and external repository integration.

## Features

- **Multi-Format Package Support**:
  - **Native**: `.pkg.tar.zst` (Redox native tar + zstd packages)
  - **Debian**: `.deb` (ar + tar + gz)
  - **RPM**: `.rpm` (cpio + xz/zstd)
  - **Alpine**: `.apk` (tar + gz)
  - **Windows**: `.msi` and `.msix`
  - **Android**: `.apk` (ZIP + DEX)
- **Multi-Repository Architecture**: Integrated repository adapters for Debian APT, Fedora DNF, Arch Pacman, Windows Winget, and Android Play Store/F-Droid.
- **Dependency & Conflict Solver**: Automatic dependency graph resolution and conflict validation.
- **Transaction Engine**: Atomic transactional engine tracking installation, upgrade, and removal sequences with disk state database persistence (`installed.json`).

## Usage & Commands

```bash
# Synchronize all enabled repositories
rpm-next sync

# Search for packages across all package repos (Pacman, APT, DNF, Winget, F-Droid)
rpm-next search <query>

# Install packages with dependency resolution
rpm-next install <package_name>

# Remove installed packages
rpm-next remove <package_name>

# Upgrade installed packages
rpm-next upgrade
```
