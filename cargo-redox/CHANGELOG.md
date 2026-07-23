# Changelog - cargo-redox

All notable changes to the `cargo-redox` project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-23

### Added
- Initial release of `cargo-redox` Cargo wrapper.
- Command-line argument parsing for `--redox-profile` / `--redox-profile=<profile>` and environment variable `REDOX_PROFILE`.
- Target triple auto-injection for `nano` (`xtensa-esp32-none-elf`) and `pro`/`titan` (`x86_64-unknown-redox`).
- Custom `RUSTFLAGS` handling for Xtensa ESP32 linker configuration (`-C link-arg=-nostartfiles`).
- Safe execution proxying via Rust standard `std::process::Command`.
