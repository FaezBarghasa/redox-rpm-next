//! Repository Management
//!
//! Handles package repository synchronization and querying.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{PackageFormat, PackageInfo, PkgError, Repository};

/// Repository cache
pub struct RepositoryCache {
    /// Cache directory
    cache_dir: PathBuf,
    /// Loaded repositories
    repos: HashMap<String, RepositoryIndex>,
}

/// Repository index
pub struct RepositoryIndex {
    /// Repository metadata
    pub repo: Repository,
    /// Available packages
    pub packages: Vec<PackageInfo>,
    /// Last sync time
    pub last_sync: u64,
}

impl RepositoryCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            repos: HashMap::new(),
        }
    }

    /// Sync a repository
    pub fn sync(&mut self, repo: &Repository) -> Result<(), PkgError> {
        let index_url = match repo.format {
            PackageFormat::Deb => format!("{}/Packages.gz", repo.url),
            PackageFormat::Rpm => format!("{}/repodata/primary.xml.gz", repo.url),
            PackageFormat::Native => format!("{}/packages.json", repo.url),
            _ => return Err(PkgError::UnsupportedFormat),
        };

        // Download and parse index
        // TODO: Implement actual download and parsing

        let index = RepositoryIndex {
            repo: repo.clone(),
            packages: Vec::new(),
            last_sync: 0, // TODO: Get current time
        };

        self.repos.insert(repo.name.clone(), index);
        Ok(())
    }

    /// Sync all repositories
    pub fn sync_all(&mut self, repos: &[Repository]) -> Result<(), PkgError> {
        for repo in repos {
            if repo.enabled {
                self.sync(repo)?;
            }
        }
        Ok(())
    }

    /// Search for a package across all repositories
    pub fn search(&self, query: &str) -> Vec<&PackageInfo> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for index in self.repos.values() {
            for pkg in &index.packages {
                if pkg.name.to_lowercase().contains(&query_lower)
                    || pkg.description.to_lowercase().contains(&query_lower)
                {
                    results.push(pkg);
                }
            }
        }

        results
    }

    /// Find a package by exact name
    pub fn find(&self, name: &str) -> Option<&PackageInfo> {
        // Search in priority order (higher priority first)
        let mut repo_list: Vec<_> = self.repos.values().collect();
        repo_list.sort_by(|a, b| b.repo.priority.cmp(&a.repo.priority));

        for index in repo_list {
            for pkg in &index.packages {
                if pkg.name == name {
                    return Some(pkg);
                }
            }
        }
        None
    }

    /// Get all versions of a package
    pub fn get_versions(&self, name: &str) -> Vec<&PackageInfo> {
        let mut versions = Vec::new();
        for index in self.repos.values() {
            for pkg in &index.packages {
                if pkg.name == name {
                    versions.push(pkg);
                }
            }
        }
        versions
    }

    /// Get package download URL
    pub fn get_download_url(&self, pkg: &PackageInfo) -> Option<String> {
        // Find repository that contains this package
        for index in self.repos.values() {
            if index
                .packages
                .iter()
                .any(|p| p.name == pkg.name && p.version == pkg.version)
            {
                let filename = match pkg.format {
                    PackageFormat::Deb => format!("{}_{}.deb", pkg.name, pkg.version),
                    PackageFormat::Rpm => format!("{}-{}.{}.rpm", pkg.name, pkg.version, pkg.arch),
                    PackageFormat::Native => {
                        format!("{}-{}-{}.pkg.tar.zst", pkg.name, pkg.version, pkg.arch)
                    }
                    _ => return None,
                };
                return Some(format!("{}/{}", index.repo.url, filename));
            }
        }
        None
    }
}

impl Default for RepositoryCache {
    fn default() -> Self {
        Self::new(PathBuf::from("/var/cache/rpm-next/repos"))
    }
}
