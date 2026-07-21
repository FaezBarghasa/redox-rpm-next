//! DEB Package Adapter
//!
//! Handles Debian package format (.deb)

use std::io::{Read, Seek};
use std::path::Path;

use crate::{ConstraintOp, Dependency, PackageFormat, PackageInfo, PkgError, VersionConstraint};

/// Parse a .deb package
pub fn parse_deb(path: &Path) -> Result<PackageInfo, PkgError> {
    // DEB format: ar archive containing:
    // - debian-binary (version)
    // - control.tar.gz (metadata)
    // - data.tar.* (files)

    let file = std::fs::File::open(path).map_err(|e| PkgError::IoError(e))?;

    // For now, return a stub - real implementation would parse ar archive
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(PackageInfo {
        name,
        version: "1.0.0".to_string(),
        release: 1,
        arch: "amd64".to_string(),
        format: PackageFormat::Deb,
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

/// Parse control file content
pub fn parse_control(content: &str) -> Result<PackageInfo, PkgError> {
    let mut info = PackageInfo {
        name: String::new(),
        version: String::new(),
        release: 1,
        arch: String::new(),
        format: PackageFormat::Deb,
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
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "Package" => info.name = value.to_string(),
                "Version" => info.version = value.to_string(),
                "Architecture" => info.arch = value.to_string(),
                "Description" => info.description = value.to_string(),
                "Maintainer" => info.maintainer = value.to_string(),
                "Homepage" => info.homepage = value.to_string(),
                "Installed-Size" => {
                    info.installed_size = value.parse().unwrap_or(0) * 1024;
                }
                "Depends" => {
                    info.dependencies = parse_depends(value);
                }
                "Conflicts" => {
                    info.conflicts = value.split(',').map(|s| s.trim().to_string()).collect();
                }
                "Provides" => {
                    info.provides = value.split(',').map(|s| s.trim().to_string()).collect();
                }
                "Replaces" => {
                    info.replaces = value.split(',').map(|s| s.trim().to_string()).collect();
                }
                _ => {}
            }
        }
    }

    Ok(info)
}

/// Parse dependency string
fn parse_depends(deps: &str) -> Vec<Dependency> {
    deps.split(',')
        .filter_map(|dep| {
            let dep = dep.trim();
            // Handle alternatives (|) by taking first option
            let dep = dep.split('|').next()?.trim();

            // Parse version constraint
            if let Some(idx) = dep.find('(') {
                let name = dep[..idx].trim().to_string();
                let rest = dep[idx..].trim_matches(|c| c == '(' || c == ')');
                let parts: Vec<&str> = rest.split_whitespace().collect();
                let version_constraint = if parts.len() >= 2 {
                    let op = match parts[0] {
                        "=" => ConstraintOp::Eq,
                        "<" => ConstraintOp::Lt,
                        "<=" => ConstraintOp::Le,
                        ">" => ConstraintOp::Gt,
                        ">=" => ConstraintOp::Ge,
                        _ => ConstraintOp::Eq,
                    };
                    Some(VersionConstraint {
                        operator: op,
                        version: parts[1].to_string(),
                    })
                } else {
                    None
                };
                Some(Dependency {
                    name,
                    version_constraint,
                })
            } else {
                Some(Dependency {
                    name: dep.to_string(),
                    version_constraint: None,
                })
            }
        })
        .collect()
}
