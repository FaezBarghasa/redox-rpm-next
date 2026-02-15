# Package Format Adapters

RPM-Next supports several package formats through modular adapters. Each adapter handles the extraction and metadata parsing of a specific format.

## Native (`pkg.rs`)

Native Redox packages use the `.pkg.tar.zst` format. These are handled by extracting the tar archive (compressed with Zstd) directly to the system root.

## Debian (`deb.rs`, `apt.rs`)

Handles `.deb` packages. Standard Debian packages consist of an `ar` archive containing `control.tar.gz` (metadata) and `data.tar.gz/xz/zst` (files).

## RPM (`rpm.rs`, `dnf.rs`)

Handles `.rpm` packages. RPMs use a lead-in header followed by a `cpio` archive compressed with `xz` or `zstd`.

## Repository Priority

When searching for packages, the `UnifiedRepositoryManager` uses the following priority:

1. Native (Redox)
2. Arch (Pacman)
3. Debian/Ubuntu (APT)
4. Fedora (DNF)
5. Windows (Winget)
6. Android (F-Droid)
