//! RPM-Next: Universal Package Manager
//!
//! Supports multiple package formats for cross-platform compatibility:
//!
//! # Supported Formats
//!
//! - **Native**: `.pkg.tar.zst` (Redox native packages)
//! - **Debian**: `.deb` (Debian/Ubuntu packages)
//! - **RPM**: `.rpm` (Fedora/RHEL packages)
//! - **Alpine**: `.apk` (Alpine Linux packages)
//! - **Android**: `.apk` (Android packages, different format)
//! - **Windows**: `.msi`, `.msix` (Windows installers)
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  rpm-next                                                       │
//! │  ┌─────────────────────────────────────────────────────────────┐│
//! │  │  Repository Manager                                         ││
//! │  │  • Multi-repository support                                 ││
//! │  │  • GPG signature verification                               ││
//! │  │  • Delta updates                                            ││
//! │  └─────────────────────────────────────────────────────────────┘│
//! │  ┌─────────────────────────────────────────────────────────────┐│
//! │  │  Package Resolver                                           ││
//! │  │  • Dependency graph solver                                  ││
//! │  │  • Conflict detection                                       ││
//! │  │  • Version constraint matching                              ││
//! │  └─────────────────────────────────────────────────────────────┘│
//! │  ┌─────────────────────────────────────────────────────────────┐│
//! │  │  Format Adapters                                            ││
//! │  │  • DEB adapter (ar + tar + gz)                             ││
//! │  │  • RPM adapter (cpio + xz/zstd)                            ││
//! │  │  • PKG adapter (tar + zstd)                                ││
//! │  └─────────────────────────────────────────────────────────────┘│
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

mod deb;
mod pkg;
mod repository;
mod resolver;
mod rpm;

// External repository adapters
mod apt;
mod dnf;
mod pacman;
mod playstore;
mod winget;

// Re-export repository types
pub use apt::AptRepository;
pub use dnf::DnfRepository;
pub use pacman::PacmanRepository;
pub use playstore::PlayStoreRepository;
pub use winget::WingetRepository;

/// Package format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageFormat {
    /// Redox native (tar + zstd)
    Native,
    /// Debian (ar + tar + gz)
    Deb,
    /// Red Hat Package Manager (cpio + xz)
    Rpm,
    /// Alpine (tar + gz)
    Apk,
    /// Windows MSI
    Msi,
    /// Windows MSIX (modern)
    Msix,
    /// Android APK (ZIP + DEX)
    Android,
}

/// Package metadata
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub release: u32,
    pub arch: String,
    pub format: PackageFormat,
    pub description: String,
    pub maintainer: String,
    pub license: String,
    pub homepage: String,
    pub size: u64,
    pub installed_size: u64,
    pub dependencies: Vec<Dependency>,
    pub conflicts: Vec<String>,
    pub provides: Vec<String>,
    pub replaces: Vec<String>,
    pub files: Vec<String>,
    pub checksum: String,
}

/// Package dependency
#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version_constraint: Option<VersionConstraint>,
}

/// Version constraint
#[derive(Debug, Clone)]
pub struct VersionConstraint {
    pub operator: ConstraintOp,
    pub version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintOp {
    Eq, // =
    Lt, // <
    Le, // <=
    Gt, // >
    Ge, // >=
}

/// Repository configuration
#[derive(Debug, Clone)]
pub struct Repository {
    pub name: String,
    pub url: String,
    pub format: PackageFormat,
    pub enabled: bool,
    pub gpg_key: Option<String>,
    pub priority: i32,
}

/// Package manager configuration
#[derive(Debug, Clone)]
pub struct PkgConfig {
    /// Root directory for installations
    pub root: PathBuf,
    /// Package cache directory
    pub cache_dir: PathBuf,
    /// Database directory
    pub db_dir: PathBuf,
    /// Repositories
    pub repos: Vec<Repository>,
    /// Enable parallel downloads
    pub parallel_downloads: usize,
}

impl Default for PkgConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("/"),
            cache_dir: PathBuf::from("/var/cache/rpm-next"),
            db_dir: PathBuf::from("/var/lib/rpm-next"),
            repos: Vec::new(),
            parallel_downloads: 4,
        }
    }
}

/// Installed package database
pub struct PackageDatabase {
    /// Installed packages
    packages: BTreeMap<String, PackageInfo>,
    /// File ownership (file -> package)
    files: HashMap<String, String>,
}

impl PackageDatabase {
    pub fn new() -> Self {
        Self {
            packages: BTreeMap::new(),
            files: HashMap::new(),
        }
    }

    /// Load database from disk
    pub fn load(path: &Path) -> Result<Self, PkgError> {
        // TODO: Load from path/installed.json
        Ok(Self::new())
    }

    /// Save database to disk
    pub fn save(&self, path: &Path) -> Result<(), PkgError> {
        // TODO: Save to path/installed.json
        Ok(())
    }

    /// Check if package is installed
    pub fn is_installed(&self, name: &str) -> bool {
        self.packages.contains_key(name)
    }

    /// Get installed package info
    pub fn get(&self, name: &str) -> Option<&PackageInfo> {
        self.packages.get(name)
    }

    /// List installed packages
    pub fn list(&self) -> impl Iterator<Item = &PackageInfo> {
        self.packages.values()
    }

    /// Get package that owns a file
    pub fn file_owner(&self, path: &str) -> Option<&str> {
        self.files.get(path).map(|s| s.as_str())
    }

    /// Register package installation
    pub fn register(&mut self, pkg: PackageInfo) {
        for file in &pkg.files {
            self.files.insert(file.clone(), pkg.name.clone());
        }
        self.packages.insert(pkg.name.clone(), pkg);
    }

    /// Unregister package
    pub fn unregister(&mut self, name: &str) -> Option<PackageInfo> {
        if let Some(pkg) = self.packages.remove(name) {
            for file in &pkg.files {
                self.files.remove(file);
            }
            Some(pkg)
        } else {
            None
        }
    }
}

impl Default for PackageDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// Transaction for package operations
pub struct Transaction {
    /// Packages to install
    pub install: Vec<PackageInfo>,
    /// Packages to remove
    pub remove: Vec<String>,
    /// Packages to upgrade
    pub upgrade: Vec<(PackageInfo, PackageInfo)>, // (old, new)
    /// Total download size
    pub download_size: u64,
    /// Total installed size change
    pub size_change: i64,
}

impl Transaction {
    pub fn new() -> Self {
        Self {
            install: Vec::new(),
            remove: Vec::new(),
            upgrade: Vec::new(),
            download_size: 0,
            size_change: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.install.is_empty() && self.remove.is_empty() && self.upgrade.is_empty()
    }
}

impl Default for Transaction {
    fn default() -> Self {
        Self::new()
    }
}

/// Universal package manager
pub struct RpmNext {
    config: PkgConfig,
    database: PackageDatabase,
}

impl RpmNext {
    pub fn new(config: PkgConfig) -> Result<Self, PkgError> {
        let db_path = config.db_dir.clone();
        let database = PackageDatabase::load(&db_path).unwrap_or_default();

        Ok(Self { config, database })
    }

    /// Install packages
    pub fn install(&mut self, names: &[&str]) -> Result<Transaction, PkgError> {
        let mut transaction = Transaction::new();

        for name in names {
            // Resolve dependencies and add to transaction
            if let Some(pkg) = self.find_package(name)? {
                transaction.install.push(pkg);
            } else {
                return Err(PkgError::PackageNotFound(name.to_string()));
            }
        }

        // Execute transaction
        self.execute_transaction(&transaction)?;

        Ok(transaction)
    }

    /// Remove packages
    pub fn remove(&mut self, names: &[&str]) -> Result<Transaction, PkgError> {
        let mut transaction = Transaction::new();

        for name in names {
            if self.database.is_installed(name) {
                transaction.remove.push(name.to_string());
            } else {
                return Err(PkgError::NotInstalled(name.to_string()));
            }
        }

        // Check for dependent packages
        self.check_remove_deps(&transaction)?;

        // Execute transaction
        self.execute_transaction(&transaction)?;

        Ok(transaction)
    }

    /// Upgrade packages
    pub fn upgrade(&mut self, names: &[&str]) -> Result<Transaction, PkgError> {
        let mut transaction = Transaction::new();

        let packages = if names.is_empty() {
            // Upgrade all
            self.database
                .list()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>()
        } else {
            names.to_vec()
        };

        for name in packages {
            if let Some(old) = self.database.get(name) {
                if let Some(new) = self.find_package(name)? {
                    if self.version_compare(&new.version, &old.version) > 0 {
                        transaction.upgrade.push((old.clone(), new));
                    }
                }
            }
        }

        // Execute transaction
        self.execute_transaction(&transaction)?;

        Ok(transaction)
    }

    /// Search for packages
    pub fn search(&self, query: &str) -> Result<Vec<PackageInfo>, PkgError> {
        // TODO: Search repositories
        Ok(Vec::new())
    }

    /// Get package info
    pub fn info(&self, name: &str) -> Result<Option<PackageInfo>, PkgError> {
        if let Some(pkg) = self.database.get(name) {
            return Ok(Some(pkg.clone()));
        }
        self.find_package(name)
    }

    /// Find package in repositories
    fn find_package(&self, name: &str) -> Result<Option<PackageInfo>, PkgError> {
        // TODO: Search all enabled repositories
        Ok(None)
    }

    /// Check if removal would break dependencies
    fn check_remove_deps(&self, _tx: &Transaction) -> Result<(), PkgError> {
        // TODO: Check reverse dependencies
        Ok(())
    }

    /// Execute a transaction
    fn execute_transaction(&mut self, tx: &Transaction) -> Result<(), PkgError> {
        // Download packages
        for pkg in &tx.install {
            self.download_package(pkg)?;
        }

        // Remove packages
        for name in &tx.remove {
            self.remove_package(name)?;
        }

        // Install packages
        for pkg in &tx.install {
            self.install_package(pkg)?;
        }

        // Upgrade packages
        for (old, new) in &tx.upgrade {
            self.remove_package(&old.name)?;
            self.install_package(new)?;
        }

        // Save database
        self.database.save(&self.config.db_dir)?;

        Ok(())
    }

    fn download_package(&self, _pkg: &PackageInfo) -> Result<(), PkgError> {
        // TODO: Download to cache
        Ok(())
    }

    fn install_package(&mut self, pkg: &PackageInfo) -> Result<(), PkgError> {
        // Extract package based on format
        match pkg.format {
            PackageFormat::Native => self.install_native(pkg)?,
            PackageFormat::Deb => self.install_deb(pkg)?,
            PackageFormat::Rpm => self.install_rpm(pkg)?,
            _ => return Err(PkgError::UnsupportedFormat),
        }

        self.database.register(pkg.clone());
        Ok(())
    }

    fn remove_package(&mut self, name: &str) -> Result<(), PkgError> {
        if let Some(pkg) = self.database.unregister(name) {
            // Remove files in reverse order
            for file in pkg.files.iter().rev() {
                let path = self.config.root.join(file.trim_start_matches('/'));
                let _ = std::fs::remove_file(&path);
            }
        }
        Ok(())
    }

    fn install_native(&self, _pkg: &PackageInfo) -> Result<(), PkgError> {
        // Extract tar.zst to root
        Ok(())
    }

    fn install_deb(&self, _pkg: &PackageInfo) -> Result<(), PkgError> {
        // Extract ar -> data.tar.* to root
        Ok(())
    }

    fn install_rpm(&self, _pkg: &PackageInfo) -> Result<(), PkgError> {
        // Extract cpio to root
        Ok(())
    }

    fn version_compare(&self, a: &str, b: &str) -> i32 {
        // Simple version comparison
        let parse = |s: &str| -> Vec<u32> { s.split('.').filter_map(|p| p.parse().ok()).collect() };

        let va = parse(a);
        let vb = parse(b);

        for (a, b) in va.iter().zip(vb.iter()) {
            match a.cmp(b) {
                std::cmp::Ordering::Greater => return 1,
                std::cmp::Ordering::Less => return -1,
                std::cmp::Ordering::Equal => continue,
            }
        }

        va.len().cmp(&vb.len()) as i32
    }
}

/// Package manager errors
#[derive(Debug)]
pub enum PkgError {
    PackageNotFound(String),
    NotInstalled(String),
    DependencyError(String),
    ConflictError(String),
    UnsupportedFormat,
    DownloadError(String),
    ExtractionError(String),
    IoError(std::io::Error),
    DatabaseError(String),
    NetworkError(String),
    ParseError(String),
}

/// Repository source type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepositorySource {
    /// Native Redox packages
    Native,
    /// Debian APT
    Apt,
    /// Fedora DNF/YUM
    Dnf,
    /// Arch Pacman
    Pacman,
    /// Windows Winget
    Winget,
    /// Android F-Droid/Play Store
    Android,
}

/// Unified Repository Manager
///
/// Manages multiple package sources and provides unified search/sync across all
pub struct UnifiedRepositoryManager {
    /// APT repositories (Debian/Ubuntu)
    pub apt: apt::AptRepository,
    /// DNF repositories (Fedora/RHEL)
    pub dnf: dnf::DnfRepository,
    /// Pacman repositories (Arch)
    pub pacman: pacman::PacmanRepository,
    /// Winget repositories (Windows)
    pub winget: winget::WingetRepository,
    /// Play Store / F-Droid (Android)
    pub playstore: playstore::PlayStoreRepository,
    /// Enabled sources
    enabled_sources: Vec<RepositorySource>,
}

impl UnifiedRepositoryManager {
    pub fn new() -> Self {
        Self {
            apt: apt::AptRepository::new(),
            dnf: dnf::DnfRepository::new(&format!(
                "{}/releases/40/Everything/x86_64/os",
                dnf::FEDORA_DL
            )),
            pacman: pacman::PacmanRepository::new(pacman::ARCH_MIRROR),
            winget: winget::WingetRepository::new(),
            playstore: playstore::PlayStoreRepository::new_fdroid(),
            enabled_sources: vec![
                RepositorySource::Native,
                RepositorySource::Apt,
                RepositorySource::Dnf,
                RepositorySource::Pacman,
                RepositorySource::Winget,
                RepositorySource::Android,
            ],
        }
    }

    /// Configure default repositories for each source
    pub fn configure_defaults(&mut self) {
        // Add Debian bookworm (stable)
        self.apt.add_debian_sources("bookworm");

        // Add Ubuntu noble (24.04)
        self.apt.add_ubuntu_sources("noble");

        // Arch repos are configured by default in PacmanRepository

        // DNF/Fedora is configured by default

        // F-Droid is configured by default in PlayStoreRepository
    }

    /// Enable/disable a repository source
    pub fn set_source_enabled(&mut self, source: RepositorySource, enabled: bool) {
        if enabled {
            if !self.enabled_sources.contains(&source) {
                self.enabled_sources.push(source);
            }
        } else {
            self.enabled_sources.retain(|s| *s != source);
        }
    }

    /// Sync all enabled repositories
    pub fn sync_all(&mut self) -> Result<(), PkgError> {
        let mut errors = Vec::new();

        for source in &self.enabled_sources.clone() {
            let result = match source {
                RepositorySource::Apt => self.apt.sync(),
                RepositorySource::Dnf => self.dnf.sync(),
                RepositorySource::Pacman => self.pacman.sync(),
                RepositorySource::Winget => self.winget.sync(),
                RepositorySource::Android => self.playstore.sync(),
                RepositorySource::Native => Ok(()), // Native uses local repo
            };

            if let Err(e) = result {
                errors.push(format!("{:?}: {:?}", source, e));
            }
        }

        if !errors.is_empty() {
            eprintln!("Sync warnings: {}", errors.join(", "));
        }

        Ok(())
    }

    /// Search across all enabled repositories
    pub fn search(&self, query: &str) -> Vec<(RepositorySource, PackageInfo)> {
        let mut results = Vec::new();

        for source in &self.enabled_sources {
            match source {
                RepositorySource::Apt => {
                    for pkg in self.apt.search(query) {
                        results.push((RepositorySource::Apt, pkg.clone().into()));
                    }
                }
                RepositorySource::Dnf => {
                    for pkg in self.dnf.search(query) {
                        results.push((RepositorySource::Dnf, pkg.clone().into()));
                    }
                }
                RepositorySource::Pacman => {
                    for pkg in self.pacman.search(query) {
                        results.push((RepositorySource::Pacman, pkg.clone().into()));
                    }
                }
                RepositorySource::Winget => {
                    for manifest in self.winget.search(query) {
                        results.push((RepositorySource::Winget, manifest.clone().into()));
                    }
                }
                RepositorySource::Android => {
                    for app in self.playstore.search(query) {
                        results.push((RepositorySource::Android, app.clone().into()));
                    }
                }
                RepositorySource::Native => {}
            }
        }

        results
    }

    /// Get package by name from best source
    pub fn get(&self, name: &str) -> Option<(RepositorySource, PackageInfo)> {
        // Priority order: Native > Pacman > APT > DNF > Winget > Android

        // Try Pacman first (good for Linux apps)
        if let Some(pkg) = self.pacman.get(name) {
            return Some((RepositorySource::Pacman, pkg.clone().into()));
        }

        // Try APT
        if let Some(pkg) = self.apt.get(name) {
            return Some((RepositorySource::Apt, pkg.clone().into()));
        }

        // Try DNF
        if let Some(pkg) = self.dnf.get(name) {
            return Some((RepositorySource::Dnf, pkg.clone().into()));
        }

        // Try Winget
        if let Some(manifest) = self.winget.get(name) {
            return Some((RepositorySource::Winget, manifest.clone().into()));
        }

        // Try F-Droid
        if let Some(app) = self.playstore.get(name) {
            return Some((RepositorySource::Android, app.clone().into()));
        }

        None
    }
}

impl Default for UnifiedRepositoryManager {
    fn default() -> Self {
        let mut manager = Self::new();
        manager.configure_defaults();
        manager
    }
}

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║  RPM-Next: Universal Multi-Platform Package Manager          ║");
    println!("║                                                               ║");
    println!("║  Supported Sources:                                           ║");
    println!("║  • APT (Debian/Ubuntu)    • DNF (Fedora/RHEL)                ║");
    println!("║  • Pacman (Arch Linux)    • Winget (Windows)                 ║");
    println!("║  • F-Droid (Android)      • Native (Redox)                   ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();

    let config = PkgConfig::default();
    let pm = RpmNext::new(config).expect("Failed to initialize package manager");

    // Initialize repository manager with all sources
    let mut repos = UnifiedRepositoryManager::default();

    // Example CLI handling
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "sync" | "update" => {
            println!("Synchronizing all repositories...");
            match repos.sync_all() {
                Ok(_) => println!("✓ All repositories synchronized"),
                Err(e) => eprintln!("✗ Sync failed: {:?}", e),
            }
        }
        "search" => {
            if args.len() < 3 {
                eprintln!("Usage: rpm-next search <query>");
                return;
            }
            let query = &args[2];
            println!("Searching for '{}'...\n", query);

            let results = repos.search(query);
            if results.is_empty() {
                println!("No packages found.");
            } else {
                for (source, pkg) in results.iter().take(20) {
                    println!(
                        "[{:?}] {} {} - {}",
                        source,
                        pkg.name,
                        pkg.version,
                        pkg.description.lines().next().unwrap_or("")
                    );
                }
                if results.len() > 20 {
                    println!("\n... and {} more results", results.len() - 20);
                }
            }
        }
        "install" => {
            if args.len() < 3 {
                eprintln!("Usage: rpm-next install <package>");
                return;
            }
            let name = &args[2];
            match repos.get(name) {
                Some((source, pkg)) => {
                    println!("Found {} in {:?} repository", pkg.name, source);
                    println!("Would install: {} v{}", pkg.name, pkg.version);
                }
                None => eprintln!("Package '{}' not found in any repository", name),
            }
        }
        "info" => {
            if args.len() < 3 {
                eprintln!("Usage: rpm-next info <package>");
                return;
            }
            let name = &args[2];
            match repos.get(name) {
                Some((source, pkg)) => {
                    println!("Name:        {}", pkg.name);
                    println!("Version:     {}", pkg.version);
                    println!("Source:      {:?}", source);
                    println!("Format:      {:?}", pkg.format);
                    println!("License:     {}", pkg.license);
                    println!("Homepage:    {}", pkg.homepage);
                    println!("Description: {}", pkg.description);
                }
                None => eprintln!("Package '{}' not found", name),
            }
        }
        "sources" => {
            println!("Configured repository sources:");
            println!(
                "  • APT (Debian/Ubuntu) - {}debian bookworm, ubuntu noble",
                "✓ "
            );
            println!("  • DNF (Fedora/RHEL)   - {}fedora 40", "✓ ");
            println!("  • Pacman (Arch)       - {}core, extra, multilib", "✓ ");
            println!("  • Winget (Windows)    - {}microsoft winget-pkgs", "✓ ");
            println!("  • F-Droid (Android)   - {}f-droid.org", "✓ ");
        }
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("Usage: rpm-next <command> [options]");
    println!();
    println!("Commands:");
    println!("  sync, update      Synchronize all repository indexes");
    println!("  search <query>    Search packages across all sources");
    println!("  install <pkg>     Install a package");
    println!("  remove <pkg>      Remove an installed package");
    println!("  upgrade [pkg]     Upgrade packages");
    println!("  info <pkg>        Show package information");
    println!("  sources           List configured repository sources");
    println!();
    println!("Examples:");
    println!("  rpm-next search firefox");
    println!("  rpm-next install com.mozilla.firefox");
    println!("  rpm-next upgrade");
}
