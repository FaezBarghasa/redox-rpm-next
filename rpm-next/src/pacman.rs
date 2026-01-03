//! Pacman Repository Adapter
//!
//! Connects to Arch Linux pacman repositories.
//! Supports the .db.tar.gz package database format.

use std::collections::HashMap;
use std::io::Read;

use crate::{
    ConstraintOp, Dependency, PackageFormat, PackageInfo, PkgError, Repository, VersionConstraint,
};

/// Official Arch Linux mirrors
pub const ARCH_MIRROR: &str = "https://mirror.rackspace.com/archlinux";
pub const ARCH_REPOS: &[&str] = &["core", "extra", "multilib"];

/// Pacman database entry
#[derive(Debug, Clone, Default)]
pub struct PacmanPackage {
    pub name: String,
    pub version: String,
    pub base: String,
    pub desc: String,
    pub url: String,
    pub arch: String,
    pub builddate: u64,
    pub installdate: u64,
    pub packager: String,
    pub size: u64,
    pub isize: u64,
    pub license: String,
    pub groups: Vec<String>,
    pub depends: Vec<Dependency>,
    pub optdepends: Vec<String>,
    pub makedepends: Vec<String>,
    pub conflicts: Vec<String>,
    pub provides: Vec<String>,
    pub replaces: Vec<String>,
    pub filename: String,
    pub md5sum: String,
    pub sha256sum: String,
    pub pgpsig: String,
}

/// Parse a pacman desc file
pub fn parse_desc(content: &str) -> PacmanPackage {
    let mut pkg = PacmanPackage::default();
    let mut current_field = String::new();
    let mut values: Vec<String> = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        if line.is_empty() {
            if !current_field.is_empty() {
                apply_field(&mut pkg, &current_field, &values);
                current_field.clear();
                values.clear();
            }
            continue;
        }

        if line.starts_with('%') && line.ends_with('%') {
            if !current_field.is_empty() {
                apply_field(&mut pkg, &current_field, &values);
                values.clear();
            }
            current_field = line[1..line.len() - 1].to_string();
        } else if !current_field.is_empty() {
            values.push(line.to_string());
        }
    }

    if !current_field.is_empty() {
        apply_field(&mut pkg, &current_field, &values);
    }

    pkg
}

fn apply_field(pkg: &mut PacmanPackage, field: &str, values: &[String]) {
    let first = values.first().map(|s| s.as_str()).unwrap_or("");

    match field {
        "NAME" => pkg.name = first.to_string(),
        "VERSION" => pkg.version = first.to_string(),
        "BASE" => pkg.base = first.to_string(),
        "DESC" => pkg.desc = first.to_string(),
        "URL" => pkg.url = first.to_string(),
        "ARCH" => pkg.arch = first.to_string(),
        "BUILDDATE" => pkg.builddate = first.parse().unwrap_or(0),
        "INSTALLDATE" => pkg.installdate = first.parse().unwrap_or(0),
        "PACKAGER" => pkg.packager = first.to_string(),
        "SIZE" => pkg.size = first.parse().unwrap_or(0),
        "ISIZE" => pkg.isize = first.parse().unwrap_or(0),
        "LICENSE" => pkg.license = first.to_string(),
        "GROUPS" => pkg.groups = values.to_vec(),
        "DEPENDS" => pkg.depends = values.iter().map(|s| parse_pacman_dep(s)).collect(),
        "OPTDEPENDS" => pkg.optdepends = values.to_vec(),
        "MAKEDEPENDS" => pkg.makedepends = values.iter().map(|s| s.to_string()).collect(),
        "CONFLICTS" => pkg.conflicts = values.to_vec(),
        "PROVIDES" => pkg.provides = values.to_vec(),
        "REPLACES" => pkg.replaces = values.to_vec(),
        "FILENAME" => pkg.filename = first.to_string(),
        "MD5SUM" => pkg.md5sum = first.to_string(),
        "SHA256SUM" => pkg.sha256sum = first.to_string(),
        "PGPSIG" => pkg.pgpsig = first.to_string(),
        _ => {}
    }
}

/// Parse a pacman dependency string like "glibc>=2.17"
fn parse_pacman_dep(s: &str) -> Dependency {
    // Remove description after colon
    let s = s.split(':').next().unwrap_or(s).trim();

    // Check for version constraint
    if let Some(idx) = s.find(|c| c == '>' || c == '<' || c == '=') {
        let name = s[..idx].to_string();
        let constraint_str = &s[idx..];

        let constraint = parse_pacman_version_constraint(constraint_str);
        Dependency {
            name,
            version_constraint: constraint,
        }
    } else {
        Dependency {
            name: s.to_string(),
            version_constraint: None,
        }
    }
}

fn parse_pacman_version_constraint(s: &str) -> Option<VersionConstraint> {
    let s = s.trim();

    let (op, version) = if let Some(v) = s.strip_prefix(">=") {
        (ConstraintOp::Ge, v)
    } else if let Some(v) = s.strip_prefix("<=") {
        (ConstraintOp::Le, v)
    } else if let Some(v) = s.strip_prefix('>') {
        (ConstraintOp::Gt, v)
    } else if let Some(v) = s.strip_prefix('<') {
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

impl From<PacmanPackage> for PackageInfo {
    fn from(pac: PacmanPackage) -> Self {
        PackageInfo {
            name: pac.name,
            version: pac.version,
            release: 1,
            arch: pac.arch,
            format: PackageFormat::Native, // Pacman uses tar.zst like our native
            description: pac.desc,
            maintainer: pac.packager,
            license: pac.license,
            homepage: pac.url,
            size: pac.size,
            installed_size: pac.isize,
            dependencies: pac.depends,
            conflicts: pac.conflicts,
            provides: pac.provides,
            replaces: pac.replaces,
            files: Vec::new(),
            checksum: pac.sha256sum,
        }
    }
}

/// Pacman repository manager
pub struct PacmanRepository {
    /// Mirror URL
    mirror: String,
    /// Repositories to use (core, extra, multilib, etc.)
    repos: Vec<String>,
    /// Package cache
    packages: HashMap<String, Vec<PacmanPackage>>,
}

impl PacmanRepository {
    pub fn new(mirror: &str) -> Self {
        Self {
            mirror: mirror.to_string(),
            repos: ARCH_REPOS.iter().map(|s| s.to_string()).collect(),
            packages: HashMap::new(),
        }
    }

    /// Get database URL for a repository
    pub fn db_url(&self, repo: &str, arch: &str) -> String {
        format!("{}/{}/os/{}/{}.db", self.mirror, repo, arch, repo)
    }

    /// Get compressed database URL
    pub fn db_gz_url(&self, repo: &str, arch: &str) -> String {
        format!("{}/{}/os/{}/{}.db.tar.gz", self.mirror, repo, arch, repo)
    }

    /// Sync all repositories
    pub fn sync(&mut self) -> Result<(), PkgError> {
        for repo in &self.repos.clone() {
            let url = self.db_gz_url(repo, "x86_64");
            // TODO: Download and extract database
            // Each package has a directory: name-version/desc
        }
        Ok(())
    }

    /// Search for packages
    pub fn search(&self, query: &str) -> Vec<&PacmanPackage> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for versions in self.packages.values() {
            for pkg in versions {
                if pkg.name.to_lowercase().contains(&query_lower)
                    || pkg.desc.to_lowercase().contains(&query_lower)
                {
                    results.push(pkg);
                }
            }
        }

        results
    }

    /// Get a specific package
    pub fn get(&self, name: &str) -> Option<&PacmanPackage> {
        self.packages.get(name)?.last()
    }

    /// Get download URL for a package
    pub fn get_download_url(&self, repo: &str, pkg: &PacmanPackage) -> String {
        format!("{}/{}/os/{}/{}", self.mirror, repo, pkg.arch, pkg.filename)
    }
}

impl Default for PacmanRepository {
    fn default() -> Self {
        Self::new(ARCH_MIRROR)
    }
}

/// Create a pacman repository configuration
pub fn create_pacman_repo(name: &str, mirror: &str, repo: &str) -> Repository {
    Repository {
        name: format!("pacman-{}-{}", name, repo),
        url: format!("{}/{}/os/x86_64", mirror, repo),
        format: PackageFormat::Native,
        enabled: true,
        gpg_key: None,
        priority: 75,
    }
}
