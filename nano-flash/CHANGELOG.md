# Changelog - nano-flash

All notable changes to the `nano-flash` project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-23

### Added
- Initial release of `nano-flash` executable tool.
- CLI subcommand structure powered by `clap`.
- `build` subcommand for bundling `kernel`, `hal`, and `ion` binaries into a unified flash layout.
- `RDOX` magic identification signature (`0x52`, `0x44`, `0x4F`, `0x58`).
- Header generation with CRC32 checksums powered by `crc32fast`.
- `write` subcommand for serial device flashing interface.
