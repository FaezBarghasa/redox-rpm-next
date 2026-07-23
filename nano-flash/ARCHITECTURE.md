# Architecture - nano-flash

`nano-flash` organizes microkernel and OS component binaries into a single, structured contiguous flash memory image.

## Flash Layout Architecture

```text
+--------------------------------------------------------+
| Offset 0x00: Magic Signature [0x52, 0x44, 0x4F, 0x58]   |
+--------------------------------------------------------+
| Offset 0x04: Table of Contents (TOC)                   |
|   - Component Header 0: Kernel (28 bytes)              |
|   - Component Header 1: HAL (28 bytes)                 |
|   - Component Header 2: Ion Shell (28 bytes)           |
+--------------------------------------------------------+
| Offset 0x58: Payload Component Data                    |
|   - Kernel Payload Data                                |
|   - HAL Payload Data                                   |
|   - Ion Shell Payload Data                             |
+--------------------------------------------------------+
```

## Component Header Definition

Each entry in the TOC is represented by a C-compatible memory layout (`#[repr(C)]`):

```rust
struct ComponentHeader {
    name: [u8; 16], // Null-padded 16-byte component identifier
    offset: u32,    // Start offset in binary file
    size: u32,      // Total byte length of binary payload
    crc: u32,       // CRC32 checksum over the payload bytes
}
```

## Operational Flow

1. **Header Space Reservation**: Allocates initial byte region for Magic signature and 3 component headers.
2. **Sequential Packing**: Appends raw binary data for `kernel`, `hal`, and `ion` sequentially while maintaining exact byte offsets.
3. **CRC Verification**: Calculates `crc32fast` checksums during file reading and constructs TOC headers.
4. **TOC Serialization**: Rewinds file cursor to offset `0x04` and writes TOC headers to form the final unified image.
