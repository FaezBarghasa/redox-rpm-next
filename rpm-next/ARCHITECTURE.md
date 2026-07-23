# Architecture - rpm-next

`rpm-next` acts as a universal package manager bridging foreign binary distributions and Redox OS.

## Core Component Diagram

```text
┌─────────────────────────────────────────────────────────────────┐
│  rpm-next Core                                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │  Unified Repository Manager                                 ││
│  │  • APT (Debian/Ubuntu)     • DNF (Fedora/RHEL)              ││
│  │  • Pacman (Arch Linux)     • Winget (Windows)               ││
│  │  • F-Droid (Android)       • Native (Redox PKG)             ││
│  └─────────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────────┐│
│  │  Package Resolver & Transaction Engine                      ││
│  │  • Dependency solver       • File conflict checking         ││
│  │  • Transaction creation    • Database persistence           ││
│  └─────────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────────┐│
│  │  Format Adapters                                            ││
│  │  • DEB adapter (ar + tar)  • RPM adapter (cpio + zstd)      ││
│  │  • PKG adapter (tar + zstd)                                 ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

## Key Architectural Modules

1. **Repository Manager (`UnifiedRepositoryManager`)**:
   - Manages multiple repository backends (`apt`, `dnf`, `pacman`, `winget`, `playstore`).
   - Prioritizes search results according to distribution compatibility order: Native > Pacman > APT > DNF > Winget > Android.

2. **Package Database (`PackageDatabase`)**:
   - Maintains an in-memory BTreeMap of installed packages and a flat file-to-package ownership index map.
   - Serializes local database state to disk at `/var/lib/rpm-next/installed.json`.

3. **Transaction Processing (`Transaction`)**:
   - Stages `install`, `remove`, and `upgrade` operations into an atomic transaction plan.
   - Calculates total download sizes and net changes in disk space before committing operations.
