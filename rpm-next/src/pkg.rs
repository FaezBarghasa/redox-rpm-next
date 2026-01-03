//! Native PKG Adapter
//!
//! Handles Redox native package format (.pkg.tar.zst)

use std::path::Path;

use crate::{Dependency, PackageFormat, PackageInfo, PkgError};

/// Parse a native .pkg.tar.zst package
pub fn parse_pkg(path: &Path) -> Result<PackageInfo, PkgError> {
    // Native format: tar archive compressed with zstd
    // Contains:
    // - .PKGINFO (metadata)
    // - .INSTALL (optional install script)
    // - files...

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
        format: PackageFormat::Native,
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

/// Parse .PKGINFO content
pub fn parse_pkginfo(content: &str) -> Result<PackageInfo, PkgError> {
    let mut info = PackageInfo {
        name: String::new(),
        version: String::new(),
        release: 1,
        arch: String::new(),
        format: PackageFormat::Native,
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
    };

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "pkgname" => info.name = value.to_string(),
                "pkgver" => info.version = value.to_string(),
                "pkgdesc" => info.description = value.to_string(),
                "url" => info.homepage = value.to_string(),
                "size" => info.installed_size = value.parse().unwrap_or(0),
                "arch" => info.arch = value.to_string(),
                "license" => info.license = value.to_string(),
                "depend" => {
                    info.dependencies.push(Dependency {
                        name: value.to_string(),
                        version_constraint: None,
                    });
                }
                "conflict" => info.conflicts.push(value.to_string()),
                "provides" => info.provides.push(value.to_string()),
                "replaces" => info.replaces.push(value.to_string()),
                _ => {}
            }
        }
    }

    Ok(info)
}
