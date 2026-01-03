//! Winget Repository Adapter
//!
//! Connects to Microsoft's Windows Package Manager (winget) repositories.
//! Uses the winget-pkgs manifest format from GitHub.
//!
//! Repository: https://github.com/microsoft/winget-pkgs

use std::collections::HashMap;

use crate::{Dependency, PackageFormat, PackageInfo, PkgError, Repository};

/// Winget manifest source URL
pub const WINGET_MANIFEST_URL: &str = "https://cdn.winget.microsoft.com/cache";
pub const WINGET_GITHUB_URL: &str =
    "https://raw.githubusercontent.com/microsoft/winget-pkgs/master";

/// Winget installer types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerType {
    Msix,
    Msi,
    Exe,
    Zip,
    Inno,
    Nullsoft,
    Burn,
    Portable,
}

impl InstallerType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "msix" | "appx" => Self::Msix,
            "msi" => Self::Msi,
            "exe" => Self::Exe,
            "zip" => Self::Zip,
            "inno" => Self::Inno,
            "nullsoft" => Self::Nullsoft,
            "burn" | "wix" => Self::Burn,
            "portable" => Self::Portable,
            _ => Self::Exe,
        }
    }
}

/// Winget package manifest
#[derive(Debug, Clone)]
pub struct WingetManifest {
    pub package_id: String,
    pub publisher: String,
    pub name: String,
    pub version: String,
    pub license: String,
    pub description: String,
    pub homepage: String,
    pub installer_type: InstallerType,
    pub installer_url: String,
    pub installer_sha256: String,
    pub architecture: String,
    pub dependencies: Vec<String>,
}

/// Parse a winget YAML manifest
pub fn parse_manifest(yaml_content: &str) -> Result<WingetManifest, PkgError> {
    let mut manifest = WingetManifest {
        package_id: String::new(),
        publisher: String::new(),
        name: String::new(),
        version: String::new(),
        license: String::new(),
        description: String::new(),
        homepage: String::new(),
        installer_type: InstallerType::Exe,
        installer_url: String::new(),
        installer_sha256: String::new(),
        architecture: "x64".to_string(),
        dependencies: Vec::new(),
    };

    // Simple YAML parsing (production would use serde_yaml)
    for line in yaml_content.lines() {
        let line = line.trim();
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'');

            match key {
                "PackageIdentifier" | "Id" => manifest.package_id = value.to_string(),
                "Publisher" => manifest.publisher = value.to_string(),
                "PackageName" | "Name" => manifest.name = value.to_string(),
                "PackageVersion" | "Version" => manifest.version = value.to_string(),
                "License" => manifest.license = value.to_string(),
                "ShortDescription" | "Description" => manifest.description = value.to_string(),
                "PackageUrl" | "Homepage" => manifest.homepage = value.to_string(),
                "InstallerType" => manifest.installer_type = InstallerType::from_str(value),
                "InstallerUrl" => manifest.installer_url = value.to_string(),
                "InstallerSha256" | "Sha256" => manifest.installer_sha256 = value.to_string(),
                "Architecture" => manifest.architecture = value.to_string(),
                _ => {}
            }
        }
    }

    if manifest.package_id.is_empty() {
        return Err(PkgError::ExtractionError(
            "Missing PackageIdentifier".to_string(),
        ));
    }

    Ok(manifest)
}

/// Convert winget manifest to PackageInfo
impl From<WingetManifest> for PackageInfo {
    fn from(manifest: WingetManifest) -> Self {
        let format = match manifest.installer_type {
            InstallerType::Msix => PackageFormat::Msix,
            InstallerType::Msi => PackageFormat::Msi,
            _ => PackageFormat::Msi, // Treat other Windows formats as MSI-like
        };

        PackageInfo {
            name: manifest.package_id.clone(),
            version: manifest.version,
            release: 1,
            arch: manifest.architecture,
            format,
            description: manifest.description,
            maintainer: manifest.publisher,
            license: manifest.license,
            homepage: manifest.homepage,
            size: 0,
            installed_size: 0,
            dependencies: manifest
                .dependencies
                .into_iter()
                .map(|name| Dependency {
                    name,
                    version_constraint: None,
                })
                .collect(),
            conflicts: Vec::new(),
            provides: Vec::new(),
            replaces: Vec::new(),
            files: Vec::new(),
            checksum: manifest.installer_sha256,
        }
    }
}

/// Winget repository
pub struct WingetRepository {
    /// Cache of package manifests
    cache: HashMap<String, WingetManifest>,
    /// Index URL
    index_url: String,
}

impl WingetRepository {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            index_url: WINGET_MANIFEST_URL.to_string(),
        }
    }

    /// Sync the package index
    pub fn sync(&mut self) -> Result<(), PkgError> {
        // Winget uses a REST API or GitHub raw manifests
        // In production: fetch index from WINGET_MANIFEST_URL

        // For now, just mark as synced
        Ok(())
    }

    /// Search for packages
    pub fn search(&self, query: &str) -> Vec<&WingetManifest> {
        let query_lower = query.to_lowercase();
        self.cache
            .values()
            .filter(|m| {
                m.package_id.to_lowercase().contains(&query_lower)
                    || m.name.to_lowercase().contains(&query_lower)
                    || m.description.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Get a specific package
    pub fn get(&self, package_id: &str) -> Option<&WingetManifest> {
        self.cache.get(package_id)
    }

    /// Get download URL for a package
    pub fn get_download_url(&self, manifest: &WingetManifest) -> String {
        manifest.installer_url.clone()
    }

    /// Fetch a single manifest from GitHub
    pub fn fetch_manifest(&mut self, package_id: &str) -> Result<WingetManifest, PkgError> {
        // Package ID format: Publisher.PackageName
        // Path: manifests/p/Publisher/PackageName/version/PackageName.yaml
        let parts: Vec<&str> = package_id.split('.').collect();
        if parts.len() < 2 {
            return Err(PkgError::PackageNotFound(package_id.to_string()));
        }

        let publisher = parts[0];
        let name = parts[1..].join(".");
        let first_letter = publisher
            .chars()
            .next()
            .ok_or_else(|| PkgError::PackageNotFound(package_id.to_string()))?
            .to_lowercase();

        let _manifest_path = format!(
            "{}/manifests/{}/{}/{}/",
            WINGET_GITHUB_URL, first_letter, publisher, name
        );

        // TODO: Fetch and parse manifest
        Err(PkgError::PackageNotFound(package_id.to_string()))
    }
}

impl Default for WingetRepository {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a winget repository configuration
pub fn create_winget_repo() -> Repository {
    Repository {
        name: "winget".to_string(),
        url: WINGET_MANIFEST_URL.to_string(),
        format: PackageFormat::Msix,
        enabled: true,
        gpg_key: None,
        priority: 50,
    }
}
