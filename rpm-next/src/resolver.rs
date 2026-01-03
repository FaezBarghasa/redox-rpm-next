//! Dependency Resolver
//!
//! SAT-based dependency resolution for package management.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::{ConstraintOp, Dependency, PackageInfo, PkgError, VersionConstraint};

/// Resolver state
pub struct Resolver {
    /// Known packages (name -> versions)
    packages: HashMap<String, Vec<PackageInfo>>,
    /// Installed packages
    installed: HashMap<String, PackageInfo>,
    /// Resolution result
    solution: Vec<PackageInfo>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            installed: HashMap::new(),
            solution: Vec::new(),
        }
    }

    /// Add available packages
    pub fn add_available(&mut self, packages: Vec<PackageInfo>) {
        for pkg in packages {
            self.packages
                .entry(pkg.name.clone())
                .or_insert_with(Vec::new)
                .push(pkg);
        }
    }

    /// Set installed packages
    pub fn set_installed(&mut self, packages: Vec<PackageInfo>) {
        for pkg in packages {
            self.installed.insert(pkg.name.clone(), pkg);
        }
    }

    /// Resolve dependencies for requested packages
    pub fn resolve(&mut self, requests: &[&str]) -> Result<Vec<PackageInfo>, PkgError> {
        self.solution.clear();
        let mut to_install: Vec<String> = requests.iter().map(|s| s.to_string()).collect();
        let mut seen: HashSet<String> = HashSet::new();

        while let Some(name) = to_install.pop() {
            if seen.contains(&name) {
                continue;
            }
            seen.insert(name.clone());

            // Check if already installed
            if self.installed.contains_key(&name) {
                continue;
            }

            // Find best version
            let pkg = self.find_best_version(&name)?;

            // Add dependencies to queue
            for dep in &pkg.dependencies {
                if !seen.contains(&dep.name) && !self.installed.contains_key(&dep.name) {
                    to_install.push(dep.name.clone());
                }
            }

            self.solution.push(pkg);
        }

        // Sort by dependencies (topological sort)
        self.topological_sort();

        Ok(self.solution.clone())
    }

    /// Find the best version of a package
    fn find_best_version(&self, name: &str) -> Result<PackageInfo, PkgError> {
        let versions = self
            .packages
            .get(name)
            .ok_or_else(|| PkgError::PackageNotFound(name.to_string()))?;

        // Return highest version
        versions
            .iter()
            .max_by(|a, b| self.compare_versions(&a.version, &b.version))
            .cloned()
            .ok_or_else(|| PkgError::PackageNotFound(name.to_string()))
    }

    /// Find version satisfying constraint
    fn find_version_satisfying(
        &self,
        name: &str,
        constraint: &VersionConstraint,
    ) -> Result<PackageInfo, PkgError> {
        let versions = self
            .packages
            .get(name)
            .ok_or_else(|| PkgError::PackageNotFound(name.to_string()))?;

        for pkg in versions {
            if self.version_satisfies(&pkg.version, constraint) {
                return Ok(pkg.clone());
            }
        }

        Err(PkgError::DependencyError(format!(
            "No version of {} satisfies constraint",
            name
        )))
    }

    /// Check if version satisfies constraint
    fn version_satisfies(&self, version: &str, constraint: &VersionConstraint) -> bool {
        let cmp = self.compare_versions(version, &constraint.version);
        match constraint.operator {
            ConstraintOp::Eq => cmp == std::cmp::Ordering::Equal,
            ConstraintOp::Lt => cmp == std::cmp::Ordering::Less,
            ConstraintOp::Le => cmp != std::cmp::Ordering::Greater,
            ConstraintOp::Gt => cmp == std::cmp::Ordering::Greater,
            ConstraintOp::Ge => cmp != std::cmp::Ordering::Less,
        }
    }

    /// Compare two version strings
    fn compare_versions(&self, a: &str, b: &str) -> std::cmp::Ordering {
        let parse = |s: &str| -> Vec<u32> {
            s.split(|c: char| !c.is_ascii_digit())
                .filter_map(|p| p.parse().ok())
                .collect()
        };

        let va = parse(a);
        let vb = parse(b);

        for (a, b) in va.iter().zip(vb.iter()) {
            match a.cmp(b) {
                std::cmp::Ordering::Equal => continue,
                other => return other,
            }
        }

        va.len().cmp(&vb.len())
    }

    /// Topological sort of solution by dependencies
    fn topological_sort(&mut self) {
        let mut result = Vec::new();
        let mut satisfied: HashSet<String> = self.installed.keys().cloned().collect();
        let mut remaining: Vec<PackageInfo> = self.solution.drain(..).collect();

        while !remaining.is_empty() {
            let mut made_progress = false;

            remaining.retain(|pkg| {
                let deps_satisfied = pkg
                    .dependencies
                    .iter()
                    .all(|dep| satisfied.contains(&dep.name));

                if deps_satisfied {
                    satisfied.insert(pkg.name.clone());
                    result.push(pkg.clone());
                    made_progress = true;
                    false
                } else {
                    true
                }
            });

            if !made_progress && !remaining.is_empty() {
                // Circular dependency - just add remaining
                result.extend(remaining.drain(..));
            }
        }

        self.solution = result;
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}
