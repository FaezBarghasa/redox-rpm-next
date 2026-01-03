//! DNF/YUM Repository Adapter
//!
//! Connects to Fedora/RHEL DNF/YUM repositories.
//! Supports repodata/primary.xml.gz metadata format.

use std::collections::HashMap;
use std::io::Read;

use crate::{
    ConstraintOp, Dependency, PackageFormat, PackageInfo, PkgError, Repository, VersionConstraint,
};

/// Fedora mirrors
pub const FEDORA_MIRROR: &str = "https://mirrors.fedoraproject.org/metalink";
pub const FEDORA_DL: &str = "https://download.fedoraproject.org/pub/fedora/linux";

/// RPM repository repodata URL patterns
pub fn repomd_url(base: &str) -> String {
    format!("{}/repodata/repomd.xml", base)
}

pub fn primary_xml_url(base: &str) -> String {
    format!("{}/repodata/primary.xml.gz", base)
}

/// RPM package from primary.xml
#[derive(Debug, Clone, Default)]
pub struct DnfPackage {
    pub name: String,
    pub arch: String,
    pub version: RpmVersion,
    pub checksum: String,
    pub checksum_type: String,
    pub summary: String,
    pub description: String,
    pub url: String,
    pub license: String,
    pub vendor: String,
    pub packager: String,
    pub buildtime: u64,
    pub size_package: u64,
    pub size_installed: u64,
    pub size_archive: u64,
    pub location_href: String,
    pub requires: Vec<RpmRequire>,
    pub provides: Vec<String>,
    pub conflicts: Vec<String>,
    pub obsoletes: Vec<String>,
    pub files: Vec<String>,
}

/// RPM version (epoch:version-release)
#[derive(Debug, Clone, Default)]
pub struct RpmVersion {
    pub epoch: u32,
    pub ver: String,
    pub rel: String,
}

impl RpmVersion {
    pub fn to_string(&self) -> String {
        if self.epoch > 0 {
            format!("{}:{}-{}", self.epoch, self.ver, self.rel)
        } else {
            format!("{}-{}", self.ver, self.rel)
        }
    }
}

/// RPM requirement
#[derive(Debug, Clone)]
pub struct RpmRequire {
    pub name: String,
    pub flags: Option<String>,
    pub epoch: Option<u32>,
    pub ver: Option<String>,
    pub rel: Option<String>,
    pub pre: bool,
}

impl RpmRequire {
    fn to_dependency(&self) -> Dependency {
        let constraint = if let (Some(flags), Some(ver)) = (&self.flags, &self.ver) {
            let op = match flags.as_str() {
                "EQ" => ConstraintOp::Eq,
                "LT" => ConstraintOp::Lt,
                "LE" => ConstraintOp::Le,
                "GT" => ConstraintOp::Gt,
                "GE" => ConstraintOp::Ge,
                _ => {
                    return Dependency {
                        name: self.name.clone(),
                        version_constraint: None,
                    }
                }
            };
            Some(VersionConstraint {
                operator: op,
                version: ver.clone(),
            })
        } else {
            None
        };

        Dependency {
            name: self.name.clone(),
            version_constraint: constraint,
        }
    }
}

/// Simple XML parser for primary.xml
/// In production, use xml-rs or quick-xml
pub fn parse_primary_xml(content: &str) -> Vec<DnfPackage> {
    let mut packages = Vec::new();
    let mut current = DnfPackage::default();
    let mut in_package = false;
    let mut current_tag = String::new();

    // Very simplified XML parsing - production would use proper parser
    // This handles the basic structure only
    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("<package ") || line == "<package>" {
            in_package = true;
            current = DnfPackage::default();
        } else if line == "</package>" {
            if !current.name.is_empty() {
                packages.push(current.clone());
            }
            in_package = false;
        } else if in_package {
            // Extract simple tags
            if let Some(name) = extract_tag_value(line, "name") {
                current.name = name;
            } else if let Some(arch) = extract_tag_value(line, "arch") {
                current.arch = arch;
            } else if let Some(summary) = extract_tag_value(line, "summary") {
                current.summary = summary;
            } else if let Some(desc) = extract_tag_value(line, "description") {
                current.description = desc;
            } else if let Some(url) = extract_tag_value(line, "url") {
                current.url = url;
            } else if let Some(license) = extract_tag_value(line, "rpm:license") {
                current.license = license;
            } else if line.starts_with("<location ") {
                if let Some(href) = extract_attribute(line, "href") {
                    current.location_href = href;
                }
            } else if line.starts_with("<checksum ") {
                if let Some(ctype) = extract_attribute(line, "type") {
                    current.checksum_type = ctype;
                }
                if let Some(checksum) = extract_tag_content(line) {
                    current.checksum = checksum;
                }
            } else if line.starts_with("<version ") {
                if let Some(epoch) = extract_attribute(line, "epoch") {
                    current.version.epoch = epoch.parse().unwrap_or(0);
                }
                if let Some(ver) = extract_attribute(line, "ver") {
                    current.version.ver = ver;
                }
                if let Some(rel) = extract_attribute(line, "rel") {
                    current.version.rel = rel;
                }
            } else if line.starts_with("<size ") {
                if let Some(pkg) = extract_attribute(line, "package") {
                    current.size_package = pkg.parse().unwrap_or(0);
                }
                if let Some(inst) = extract_attribute(line, "installed") {
                    current.size_installed = inst.parse().unwrap_or(0);
                }
            }
        }
    }

    packages
}

fn extract_tag_value(line: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);

    if line.starts_with(&open) && line.ends_with(&close) {
        let start = open.len();
        let end = line.len() - close.len();
        if start < end {
            return Some(line[start..end].to_string());
        }
    }
    None
}

fn extract_attribute(line: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    if let Some(start) = line.find(&pattern) {
        let start = start + pattern.len();
        if let Some(end) = line[start..].find('"') {
            return Some(line[start..start + end].to_string());
        }
    }
    None
}

fn extract_tag_content(line: &str) -> Option<String> {
    let start = line.find('>')?;
    let end = line.rfind('<')?;
    if start + 1 < end {
        Some(line[start + 1..end].to_string())
    } else {
        None
    }
}

impl From<DnfPackage> for PackageInfo {
    fn from(dnf: DnfPackage) -> Self {
        let release: u32 = dnf
            .version
            .rel
            .split('.')
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        PackageInfo {
            name: dnf.name,
            version: dnf.version.ver,
            release,
            arch: dnf.arch,
            format: PackageFormat::Rpm,
            description: if dnf.description.is_empty() {
                dnf.summary
            } else {
                dnf.description
            },
            maintainer: dnf.packager,
            license: dnf.license,
            homepage: dnf.url,
            size: dnf.size_package,
            installed_size: dnf.size_installed,
            dependencies: dnf.requires.iter().map(|r| r.to_dependency()).collect(),
            conflicts: dnf.conflicts,
            provides: dnf.provides,
            replaces: dnf.obsoletes,
            files: dnf.files,
            checksum: dnf.checksum,
        }
    }
}

/// DNF repository manager
pub struct DnfRepository {
    /// Base URL
    base_url: String,
    /// Package cache
    packages: HashMap<String, Vec<DnfPackage>>,
}

impl DnfRepository {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            packages: HashMap::new(),
        }
    }

    /// Sync the repository
    pub fn sync(&mut self) -> Result<(), PkgError> {
        let _primary_url = primary_xml_url(&self.base_url);
        // TODO: Download and parse primary.xml.gz
        Ok(())
    }

    /// Search for packages
    pub fn search(&self, query: &str) -> Vec<&DnfPackage> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for versions in self.packages.values() {
            for pkg in versions {
                if pkg.name.to_lowercase().contains(&query_lower)
                    || pkg.summary.to_lowercase().contains(&query_lower)
                {
                    results.push(pkg);
                }
            }
        }

        results
    }

    /// Get a specific package
    pub fn get(&self, name: &str) -> Option<&DnfPackage> {
        self.packages.get(name)?.last()
    }

    /// Get download URL for a package
    pub fn get_download_url(&self, pkg: &DnfPackage) -> String {
        format!("{}/{}", self.base_url, pkg.location_href)
    }
}

impl Default for DnfRepository {
    fn default() -> Self {
        Self::new(&format!("{}/releases/40/Everything/x86_64/os", FEDORA_DL))
    }
}

/// Create a DNF/YUM repository configuration
pub fn create_dnf_repo(name: &str, base_url: &str) -> Repository {
    Repository {
        name: name.to_string(),
        url: base_url.to_string(),
        format: PackageFormat::Rpm,
        enabled: true,
        gpg_key: None,
        priority: 90,
    }
}
