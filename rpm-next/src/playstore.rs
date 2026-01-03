//! Google Play Store Adapter
//!
//! Connects to the Google Play Store for Android app discovery and download.
//! Uses the unofficial Google Play API since there's no official public API.
//!
//! Note: This requires a Google account and device registration.
//! For legal use only with properly licensed apps.

use std::collections::HashMap;

use crate::{Dependency, PackageFormat, PackageInfo, PkgError, Repository};

/// Play Store API endpoints
pub const PLAY_STORE_API: &str = "https://android.clients.google.com";
pub const PLAY_STORE_FDROID: &str = "https://f-droid.org/repo";

/// App categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppCategory {
    Games,
    Business,
    Education,
    Entertainment,
    Finance,
    Health,
    Lifestyle,
    Music,
    News,
    Photography,
    Productivity,
    Shopping,
    Social,
    Sports,
    Tools,
    Travel,
    Utilities,
    Video,
    Weather,
    Unknown,
}

impl AppCategory {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "games" | "game" => Self::Games,
            "business" => Self::Business,
            "education" => Self::Education,
            "entertainment" => Self::Entertainment,
            "finance" => Self::Finance,
            "health" | "health_and_fitness" => Self::Health,
            "lifestyle" => Self::Lifestyle,
            "music" | "music_and_audio" => Self::Music,
            "news" | "news_and_magazines" => Self::News,
            "photography" => Self::Photography,
            "productivity" => Self::Productivity,
            "shopping" => Self::Shopping,
            "social" => Self::Social,
            "sports" => Self::Sports,
            "tools" => Self::Tools,
            "travel" | "travel_and_local" => Self::Travel,
            "utilities" => Self::Utilities,
            "video" | "video_players" => Self::Video,
            "weather" => Self::Weather,
            _ => Self::Unknown,
        }
    }
}

/// Android app metadata from Play Store
#[derive(Debug, Clone)]
pub struct PlayStoreApp {
    pub package_name: String,
    pub title: String,
    pub version_name: String,
    pub version_code: u32,
    pub developer: String,
    pub category: AppCategory,
    pub description: String,
    pub icon_url: String,
    pub download_url: String,
    pub size: u64,
    pub min_sdk: u32,
    pub target_sdk: u32,
    pub permissions: Vec<String>,
    pub rating: f32,
    pub num_downloads: u64,
    pub price: f32, // 0.0 for free apps
    pub in_app_purchases: bool,
    pub last_updated: u64,
}

/// F-Droid app metadata (open source alternative)
#[derive(Debug, Clone, Default)]
pub struct FDroidApp {
    pub package_name: String,
    pub name: String,
    pub summary: String,
    pub description: String,
    pub license: String,
    pub web_site: String,
    pub source_code: String,
    pub issue_tracker: String,
    pub categories: Vec<String>,
    pub anti_features: Vec<String>,
    pub suggested_version_code: u32,
    pub packages: Vec<FDroidPackage>,
}

#[derive(Debug, Clone, Default)]
pub struct FDroidPackage {
    pub version_name: String,
    pub version_code: u32,
    pub apk_name: String,
    pub hash: String,
    pub hash_type: String,
    pub size: u64,
    pub min_sdk: u32,
    pub target_sdk: u32,
    pub native_code: Vec<String>,
    pub permissions: Vec<String>,
}

/// Parse F-Droid index.json
pub fn parse_fdroid_index(json: &str) -> Result<Vec<FDroidApp>, PkgError> {
    // In production, use serde_json
    // This is a simplified parser

    let mut apps = Vec::new();

    // TODO: Parse JSON properly
    // For now, return empty list

    Ok(apps)
}

impl From<FDroidApp> for PackageInfo {
    fn from(app: FDroidApp) -> Self {
        let latest = app.packages.first();

        PackageInfo {
            name: app.package_name,
            version: latest.map(|p| p.version_name.clone()).unwrap_or_default(),
            release: latest.map(|p| p.version_code).unwrap_or(1),
            arch: "any".to_string(),
            format: PackageFormat::Android,
            description: if app.summary.is_empty() {
                app.description
            } else {
                app.summary
            },
            maintainer: String::new(),
            license: app.license,
            homepage: app.web_site,
            size: latest.map(|p| p.size).unwrap_or(0),
            installed_size: 0,
            dependencies: Vec::new(),
            conflicts: Vec::new(),
            provides: Vec::new(),
            replaces: Vec::new(),
            files: Vec::new(),
            checksum: latest.map(|p| p.hash.clone()).unwrap_or_default(),
        }
    }
}

impl From<PlayStoreApp> for PackageInfo {
    fn from(app: PlayStoreApp) -> Self {
        PackageInfo {
            name: app.package_name,
            version: app.version_name,
            release: app.version_code,
            arch: "any".to_string(),
            format: PackageFormat::Android,
            description: app.description,
            maintainer: app.developer,
            license: String::new(),
            homepage: String::new(),
            size: app.size,
            installed_size: 0,
            dependencies: Vec::new(),
            conflicts: Vec::new(),
            provides: Vec::new(),
            replaces: Vec::new(),
            files: Vec::new(),
            checksum: String::new(),
        }
    }
}

/// Play Store/F-Droid repository manager
pub struct PlayStoreRepository {
    /// Use F-Droid instead of Play Store
    use_fdroid: bool,
    /// F-Droid repo URL
    fdroid_url: String,
    /// App cache
    apps: HashMap<String, FDroidApp>,
}

impl PlayStoreRepository {
    pub fn new_fdroid() -> Self {
        Self {
            use_fdroid: true,
            fdroid_url: PLAY_STORE_FDROID.to_string(),
            apps: HashMap::new(),
        }
    }

    /// Add a custom F-Droid repository
    pub fn add_fdroid_repo(&mut self, url: &str) {
        self.fdroid_url = url.to_string();
    }

    /// Sync the repository
    pub fn sync(&mut self) -> Result<(), PkgError> {
        if self.use_fdroid {
            let index_url = format!("{}/index-v2.json", self.fdroid_url);
            // TODO: Download and parse index
        }
        Ok(())
    }

    /// Search for apps
    pub fn search(&self, query: &str) -> Vec<&FDroidApp> {
        let query_lower = query.to_lowercase();

        self.apps
            .values()
            .filter(|app| {
                app.package_name.to_lowercase().contains(&query_lower)
                    || app.name.to_lowercase().contains(&query_lower)
                    || app.summary.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Get a specific app
    pub fn get(&self, package_name: &str) -> Option<&FDroidApp> {
        self.apps.get(package_name)
    }

    /// Get download URL for an app
    pub fn get_download_url(&self, app: &FDroidApp) -> Option<String> {
        app.packages
            .first()
            .map(|pkg| format!("{}/{}", self.fdroid_url, pkg.apk_name))
    }

    /// Search by category
    pub fn search_by_category(&self, category: AppCategory) -> Vec<&FDroidApp> {
        let cat_str = format!("{:?}", category);

        self.apps
            .values()
            .filter(|app| {
                app.categories
                    .iter()
                    .any(|c| c.eq_ignore_ascii_case(&cat_str))
            })
            .collect()
    }
}

impl Default for PlayStoreRepository {
    fn default() -> Self {
        Self::new_fdroid()
    }
}

/// Create an F-Droid repository configuration
pub fn create_fdroid_repo(name: &str, url: &str) -> Repository {
    Repository {
        name: name.to_string(),
        url: url.to_string(),
        format: PackageFormat::Android,
        enabled: true,
        gpg_key: None,
        priority: 60,
    }
}

/// Create F-Droid main repository
pub fn create_fdroid_main_repo() -> Repository {
    create_fdroid_repo("fdroid", PLAY_STORE_FDROID)
}

/// Common F-Droid repositories
pub mod fdroid_repos {
    pub const MAIN: &str = "https://f-droid.org/repo";
    pub const ARCHIVE: &str = "https://f-droid.org/archive";
    pub const IZZY: &str = "https://apt.izzysoft.de/fdroid/repo";
    pub const GUARDIAN: &str = "https://guardianproject.info/fdroid/repo";
    pub const BITWARDEN: &str = "https://mobileapp.bitwarden.com/fdroid/repo";
}
