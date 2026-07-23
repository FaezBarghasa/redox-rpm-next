# Architecture - cargo-redox

`cargo-redox` acts as an interceptor front-end between the developer/CI build system and the `cargo` executable.

## System Architecture

```text
[ Developer / CI Command ]
          │
          ▼
┌─────────────────────────┐
│       cargo-redox       │
│  (Profile & Arg Parser) │
└───────────┬─────────────┘
            │
    ┌───────┴───────────────┐
    ▼                       ▼
[ Determine Target ]   [ Inject RUSTFLAGS ]
 (nano / pro / titan)  (e.g., -nostartfiles)
    │                       │
    └───────┬───────────────┘
            ▼
┌─────────────────────────┐
│     cargo (binary)      │
│  --target <target_triple>│
└─────────────────────────┘
```

## Core Modules & Design Decisions

1. **Argument Extraction**:
   - Drops `cargo-redox` / `cargo redox` prefixes from the argument array.
   - Extracts and strips `--redox-profile` parameters to avoid invalid flag errors when delegating to Cargo.

2. **Target Mapping**:
   - Maps profiles to target triples cleanly in a pure `match` block.
   - Supports fallback to default system target (`x86_64-unknown-redox`).

3. **Subcommand Injection**:
   - Scans position of subcommands (`build`, `check`, `test`, `run`) and injects `--target <triple>` immediately after the subcommand if `--target` is not already present.

4. **Environment Manipulation**:
   - Dynamically appends `CARGO_TARGET_XTENSA_ESP32_NONE_ELF_RUSTFLAGS` when targeting ESP32 architectures to ensure `no_std` linking constraints are fulfilled.
