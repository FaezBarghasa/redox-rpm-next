//! APT Repository Adapter
//!
//! Connects to Debian/Ubuntu APT repositories.
//! Supports both legacy (dists/) and modern repository layouts.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

use crate::{
    ConstraintOp, Dependency, PackageFormat, PackageInfo, PkgError, Repository, VersionConstraint,
};

/// Common Debian/Ubuntu mirrors
pub const DEBIAN_MIRROR: &str = "http://deb.debian.org/debian";
pub const UBUNTU_MIRROR: &str = "http://archive.ubuntu.com/ubuntu";

/// Pop!_OS mirrors (System76)
pub const POP_OS_MIRROR: &str = "http://apt.pop-os.org/release";
pub const POP_OS_PROPRIETARY: &str = "http://apt.pop-os.org/proprietary";
pub const POP_OS_CUDA: &str = "http://apt.pop-os.org/proprietary-cuda";

/// APT repository source
#[derive(Debug, Clone)]
pub struct AptSource {
    pub source_type: String, // deb or deb-src
    pub uri: String,
    pub distribution: String,
    pub components: Vec<String>,
    pub architectures: Vec<String>,
}

impl AptSource {
    /// Parse a sources.list line
    pub fn parse(line: &str) -> Option<Self> {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            return None;
        }

        let source_type = parts[0].to_string();
        if source_type != "deb" && source_type != "deb-src" {
            return None;
        }

        // Handle [arch=...] options
        let (uri_idx, archs) = if parts[1].starts_with('[') {
            let opts = parts[1].trim_start_matches('[').trim_end_matches(']');
            let archs = if let Some(arch_str) = opts.strip_prefix("arch=") {
                arch_str.split(',').map(|s| s.to_string()).collect()
            } else {
                vec!["amd64".to_string()]
            };
            (2, archs)
        } else {
            (1, vec!["amd64".to_string()])
        };

        Some(Self {
            source_type,
            uri: parts[uri_idx].to_string(),
            distribution: parts[uri_idx + 1].to_string(),
            components: parts[uri_idx + 2..].iter().map(|s| s.to_string()).collect(),
            architectures: archs,
        })
    }

    /// Get the Packages file URL
    pub fn packages_url(&self, component: &str, arch: &str) -> String {
        format!(
            "{}/dists/{}/{}/binary-{}/Packages",
            self.uri, self.distribution, component, arch
        )
    }

    /// Get the compressed Packages file URL
    pub fn packages_gz_url(&self, component: &str, arch: &str) -> String {
        format!("{}.gz", self.packages_url(component, arch))
    }
}

/// APT package entry from Packages file
#[derive(Debug, Clone, Default)]
pub struct AptPackage {
    pub package: String,
    pub version: String,
    pub architecture: String,
    pub maintainer: String,
    pub installed_size: u64,
    pub depends: Vec<Dependency>,
    pub pre_depends: Vec<Dependency>,
    pub recommends: Vec<String>,
    pub suggests: Vec<String>,
    pub conflicts: Vec<String>,
    pub provides: Vec<String>,
    pub replaces: Vec<String>,
    pub filename: String,
    pub size: u64,
    pub md5sum: String,
    pub sha256: String,
    pub section: String,
    pub priority: String,
    pub description: String,
    pub homepage: String,
}

/// Parse APT Packages file content
pub fn parse_packages(content: &str) -> Vec<AptPackage> {
    let mut packages = Vec::new();
    let mut current = AptPackage::default();
    let mut in_description = false;

    for line in content.lines() {
        if line.is_empty() {
            if !current.package.is_empty() {
                packages.push(current);
                current = AptPackage::default();
            }
            in_description = false;
            continue;
        }

        if line.starts_with(' ') && in_description {
            current.description.push('\n');
            current.description.push_str(line.trim());
            continue;
        }

        in_description = false;

        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "Package" => current.package = value.to_string(),
                "Version" => current.version = value.to_string(),
                "Architecture" => current.architecture = value.to_string(),
                "Maintainer" => current.maintainer = value.to_string(),
                "Installed-Size" => current.installed_size = value.parse().unwrap_or(0) * 1024,
                "Depends" => current.depends = parse_depends(value),
                "Pre-Depends" => current.pre_depends = parse_depends(value),
                "Recommends" => {
                    current.recommends = value.split(',').map(|s| s.trim().to_string()).collect()
                }
                "Suggests" => {
                    current.suggests = value.split(',').map(|s| s.trim().to_string()).collect()
                }
                "Conflicts" => {
                    current.conflicts = value.split(',').map(|s| s.trim().to_string()).collect()
                }
                "Provides" => {
                    current.provides = value.split(',').map(|s| s.trim().to_string()).collect()
                }
                "Replaces" => {
                    current.replaces = value.split(',').map(|s| s.trim().to_string()).collect()
                }
                "Filename" => current.filename = value.to_string(),
                "Size" => current.size = value.parse().unwrap_or(0),
                "MD5sum" => current.md5sum = value.to_string(),
                "SHA256" => current.sha256 = value.to_string(),
                "Section" => current.section = value.to_string(),
                "Priority" => current.priority = value.to_string(),
                "Homepage" => current.homepage = value.to_string(),
                "Description" => {
                    current.description = value.to_string();
                    in_description = true;
                }
                _ => {}
            }
        }
    }

    if !current.package.is_empty() {
        packages.push(current);
    }

    packages
}

/// Parse dependency string with version constraints
fn parse_depends(deps: &str) -> Vec<Dependency> {
    deps.split(',')
        .filter_map(|dep| {
            let dep = dep.trim();
            // Handle alternatives (|) by taking first option
            let dep = dep.split('|').next()?.trim();

            // Remove :any suffix
            let dep = dep.split(':').next()?.trim();

            // Parse version constraint
            if let Some(paren_start) = dep.find('(') {
                let name = dep[..paren_start].trim().to_string();
                let constraint_str = dep[paren_start..].trim_matches(|c| c == '(' || c == ')');

                let constraint = parse_version_constraint(constraint_str);
                Some(Dependency {
                    name,
                    version_constraint: constraint,
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

/// Parse version constraint like ">= 1.0"
fn parse_version_constraint(s: &str) -> Option<VersionConstraint> {
    let s = s.trim();

    let (op, version) = if let Some(v) = s.strip_prefix(">=") {
        (ConstraintOp::Ge, v)
    } else if let Some(v) = s.strip_prefix("<=") {
        (ConstraintOp::Le, v)
    } else if let Some(v) = s.strip_prefix(">>") {
        (ConstraintOp::Gt, v)
    } else if let Some(v) = s.strip_prefix("<<") {
        (ConstraintOp::Lt, v)
    } else if let Some(v) = s.strip_prefix('=') {
        (ConstraintOp::Eq, v)
    } else {
        return None;
    };

    Some(VersionConstraint {
        operator: op,
        version: version.trim().to_string(),
    })
}

impl From<AptPackage> for PackageInfo {
    fn from(apt: AptPackage) -> Self {
        let mut deps = apt.depends;
        deps.extend(apt.pre_depends);

        PackageInfo {
            name: apt.package,
            version: apt.version,
            release: 1,
            arch: apt.architecture,
            format: PackageFormat::Deb,
            description: apt.description,
            maintainer: apt.maintainer,
            license: String::new(),
            homepage: apt.homepage,
            size: apt.size,
            installed_size: apt.installed_size,
            dependencies: deps,
            conflicts: apt.conflicts,
            provides: apt.provides,
            replaces: apt.replaces,
            files: Vec::new(),
            checksum: apt.sha256,
        }
    }
}

/// APT repository manager
pub struct AptRepository {
    /// Repository sources
    sources: Vec<AptSource>,
    /// Package cache
    packages: HashMap<String, Vec<AptPackage>>,
}

impl AptRepository {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            packages: HashMap::new(),
        }
    }

    /// Add a repository source
    pub fn add_source(&mut self, source: AptSource) {
        self.sources.push(source);
    }

    /// Add default Debian sources
    pub fn add_debian_sources(&mut self, release: &str) {
        self.sources.push(AptSource {
            source_type: "deb".to_string(),
            uri: DEBIAN_MIRROR.to_string(),
            distribution: release.to_string(),
            components: vec![
                "main".to_string(),
                "contrib".to_string(),
                "non-free".to_string(),
            ],
            architectures: vec!["amd64".to_string()],
        });
    }

    /// Add default Ubuntu sources
    pub fn add_ubuntu_sources(&mut self, release: &str) {
        self.sources.push(AptSource {
            source_type: "deb".to_string(),
            uri: UBUNTU_MIRROR.to_string(),
            distribution: release.to_string(),
            components: vec![
                "main".to_string(),
                "universe".to_string(),
                "multiverse".to_string(),
            ],
            architectures: vec!["amd64".to_string()],
        });
    }

    /// Add Pop!_OS sources (System76)
    ///
    /// Pop!_OS uses Ubuntu as a base but has its own repositories for:
    /// - Main release packages (pop-os-release)
    /// - Proprietary drivers and software
    /// - CUDA toolkit and libraries
    pub fn add_pop_os_sources(&mut self, release: &str) {
        // Main Pop!_OS release repository
        self.sources.push(AptSource {
            source_type: "deb".to_string(),
            uri: POP_OS_MIRROR.to_string(),
            distribution: release.to_string(),
            components: vec!["main".to_string()],
            architectures: vec!["amd64".to_string()],
        });

        // Pop!_OS proprietary repository (NVIDIA drivers, Steam, etc.)
        self.sources.push(AptSource {
            source_type: "deb".to_string(),
            uri: POP_OS_PROPRIETARY.to_string(),
            distribution: release.to_string(),
            components: vec!["main".to_string()],
            architectures: vec!["amd64".to_string()],
        });

        // Pop!_OS CUDA repository (for machine learning/AI)
        self.sources.push(AptSource {
            source_type: "deb".to_string(),
            uri: POP_OS_CUDA.to_string(),
            distribution: release.to_string(),
            components: vec!["main".to_string()],
            architectures: vec!["amd64".to_string()],
        });

        // Also add Ubuntu base (Pop!_OS is based on Ubuntu)
        // Pop!_OS releases map to Ubuntu releases:
        // - 22.04 LTS -> jammy
        // - 24.04 LTS -> noble
        let ubuntu_release = match release {
            "jammy" | "22.04" => "jammy",
            "noble" | "24.04" => "noble",
            _ => release,
        };
        self.add_ubuntu_sources(ubuntu_release);
    }

    /// Sync all sources
    pub fn sync(&mut self) -> Result<(), PkgError> {
        for source in &self.sources {
            for component in &source.components {
                for arch in &source.architectures {
                    let url = source.packages_gz_url(component, arch);
                    // TODO: Download and decompress Packages.gz
                    // let content = download(&url)?;
                    // let packages = parse_packages(&content);
                    // self.packages.extend(...);
                }
            }
        }
        Ok(())
    }

    /// Search for packages
    pub fn search(&self, query: &str) -> Vec<&AptPackage> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for versions in self.packages.values() {
            for pkg in versions {
                if pkg.package.to_lowercase().contains(&query_lower)
                    || pkg.description.to_lowercase().contains(&query_lower)
                {
                    results.push(pkg);
                }
            }
        }

        results
    }

    /// Get a specific package
    pub fn get(&self, name: &str) -> Option<&AptPackage> {
        self.packages.get(name)?.last()
    }

    /// Get download URL for a package
    pub fn get_download_url(&self, source: &AptSource, pkg: &AptPackage) -> String {
        format!("{}/{}", source.uri, pkg.filename)
    }
}

impl Default for AptRepository {
    fn default() -> Self {
        Self::new()
    }
}

/// Create an APT repository configuration
pub fn create_apt_repo(
    name: &str,
    uri: &str,
    distribution: &str,
    components: &[&str],
) -> Repository {
    Repository {
        name: name.to_string(),
        url: format!("{}/dists/{}", uri, distribution),
        format: PackageFormat::Deb,
        enabled: true,
        gpg_key: None,
        priority: 100,
    }
}
