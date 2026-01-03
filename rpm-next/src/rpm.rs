//! RPM Package Adapter
//!
//! Handles Red Hat Package Manager format (.rpm)

use std::path::Path;

use crate::{Dependency, PackageFormat, PackageInfo, PkgError};

/// RPM header tags
pub mod tag {
    pub const NAME: u32 = 1000;
    pub const VERSION: u32 = 1001;
    pub const RELEASE: u32 = 1002;
    pub const SUMMARY: u32 = 1004;
    pub const DESCRIPTION: u32 = 1005;
    pub const SIZE: u32 = 1009;
    pub const LICENSE: u32 = 1014;
    pub const GROUP: u32 = 1016;
    pub const URL: u32 = 1020;
    pub const ARCH: u32 = 1022;
    pub const FILENAMES: u32 = 1027;
    pub const REQUIRES_NAME: u32 = 1049;
    pub const REQUIRES_VERSION: u32 = 1050;
    pub const REQUIRES_FLAGS: u32 = 1048;
    pub const CONFLICTS_NAME: u32 = 1054;
    pub const PROVIDES_NAME: u32 = 1047;
    pub const OBSOLETES_NAME: u32 = 1090;
}

/// Parse an .rpm package
pub fn parse_rpm(path: &Path) -> Result<PackageInfo, PkgError> {
    // RPM format:
    // - Lead (96 bytes, obsolete)
    // - Signature (header structure)
    // - Header (metadata)
    // - Payload (cpio archive, usually compressed)

    let file = std::fs::File::open(path).map_err(|e| PkgError::IoError(e))?;

    // For now, return a stub - real implementation would parse RPM headers
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(PackageInfo {
        name,
        version: "1.0.0".to_string(),
        release: 1,
        arch: "x86_64".to_string(),
        format: PackageFormat::Rpm,
        description: String::new(),
        maintainer: String::new(),
        license: String::new(),
        homepage: String::new(),
        size: 0,
        installed_size: 0,
        dependencies: Vec::new(),
        conflicts: Vec::new(),
        provides: Vec::new(),
        replaces: Vec::new(),
        files: Vec::new(),
        checksum: String::new(),
    })
}

/// RPM header entry
#[derive(Debug)]
pub struct HeaderEntry {
    pub tag: u32,
    pub entry_type: u32,
    pub offset: u32,
    pub count: u32,
}

/// Parse RPM header
pub fn parse_header(data: &[u8]) -> Result<Vec<HeaderEntry>, PkgError> {
    if data.len() < 16 {
        return Err(PkgError::ExtractionError("Header too short".to_string()));
    }

    // Check magic
    let magic = &data[0..4];
    if magic != [0x8e, 0xad, 0xe8, 0x01] {
        return Err(PkgError::ExtractionError(
            "Invalid header magic".to_string(),
        ));
    }

    let num_entries = u32::from_be_bytes([data[8], data[9], data[10], data[11]]) as usize;
    let data_size = u32::from_be_bytes([data[12], data[13], data[14], data[15]]) as usize;

    let mut entries = Vec::with_capacity(num_entries);
    let entry_start = 16;

    for i in 0..num_entries {
        let offset = entry_start + i * 16;
        if offset + 16 > data.len() {
            break;
        }

        let tag = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        let entry_type = u32::from_be_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);
        let data_offset = u32::from_be_bytes([
            data[offset + 8],
            data[offset + 9],
            data[offset + 10],
            data[offset + 11],
        ]);
        let count = u32::from_be_bytes([
            data[offset + 12],
            data[offset + 13],
            data[offset + 14],
            data[offset + 15],
        ]);

        entries.push(HeaderEntry {
            tag,
            entry_type,
            offset: data_offset,
            count,
        });
    }

    Ok(entries)
}
