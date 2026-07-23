# `cargo-redox`

`cargo-redox` is a Cargo CLI wrapper utility designed for Redox OS. It provides automatic architecture and target triple detection based on Redox deployment profiles (`nano`, `pro`, `titan`).

## Features

- **Profile-Driven Compilation**: Automatically maps profile options (`--redox-profile`) to targeted Rust triples.
- **Nano Target Support**: Injects custom linker scripts and flags (`CARGO_TARGET_XTENSA_ESP32_NONE_ELF_RUSTFLAGS`) when building for embedded microcontrollers (`xtensa-esp32-none-elf`).
- **Seamless Proxying**: Transparently forwards all subcommands and parameters (`build`, `check`, `run`, `test`) to `cargo`.

## Installation & Usage

### Usage

```bash
# Build for standard Redox OS target (x86_64-unknown-redox)
cargo redox build

# Build for Nano profile (ESP32 microcontrollers)
cargo redox build --redox-profile=nano

# Pass environment variable instead of command line flags
REDOX_PROFILE=nano cargo redox build
```

## Profiles Overview

| Profile | Target Triple | Primary Usecase |
| :--- | :--- | :--- |
| `nano` / `nano-esp32` | `xtensa-esp32-none-elf` | Microcontroller & low-power embedded RTOS deployments |
| `pro` / `workstation` | `x86_64-unknown-redox` | Redox OS Workstations, desktop apps, and gaming |
| `titan` / `server` | `x86_64-unknown-redox` | Enterprise servers and cloud workloads |
