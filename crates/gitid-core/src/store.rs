//! Profile and rules persistence — reads/writes YAML config files.
//!
//! Config files are stored in `~/.config/gitid/` (or XDG equivalent):
//! - profiles.yaml — all identity profiles
//! - rules.yaml — matching rules for profile resolution

use crate::error::{Error, Result};
use crate::profile::ProfileStore;
use crate::resolver::RuleStore;
use std::fs;
use std::path::{Path, PathBuf};

/// Returns the GitID config directory, creating it if needed.
/// Uses XDG_CONFIG_HOME on Linux, ~/Library/Application Support on macOS,
/// %APPDATA% on Windows.
pub fn config_dir() -> Result<PathBuf> {
    let base = dirs::config_dir().ok_or(Error::NoConfigDir)?;
    let dir = base.join("gitid");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| Error::Io {
            path: dir.clone(),
            source: e,
        })?;
    }
    Ok(dir)
}

/// Path to the profiles.yaml file.
pub fn profiles_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("profiles.yaml"))
}

/// Path to the rules.yaml file.
pub fn rules_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("rules.yaml"))
}

/// Load profiles from disk. Returns an empty store if the file doesn't exist.
pub fn load_profiles() -> Result<ProfileStore> {
    load_profiles_from(&profiles_path()?)
}

/// Load profiles from a specific path.
pub fn load_profiles_from(path: &Path) -> Result<ProfileStore> {
    if !path.exists() {
        return Ok(ProfileStore::new());
    }
    let content = fs::read_to_string(path).map_err(|e| Error::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    if content.trim().is_empty() {
        return Ok(ProfileStore::new());
    }
    let store: ProfileStore = serde_yaml::from_str(&content).map_err(|e| Error::YamlParse {
        path: path.to_path_buf(),
        source: e,
    })?;
    Ok(store)
}

/// Save profiles to disk.
pub fn save_profiles(store: &ProfileStore) -> Result<()> {
    save_profiles_to(store, &profiles_path()?)
}

/// Save profiles to a specific path.
pub fn save_profiles_to(store: &ProfileStore, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| Error::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
    }
    let content = serde_yaml::to_string(store).map_err(Error::YamlSerialize)?;
    fs::write(path, content).map_err(|e| Error::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    Ok(())
}

/// Load rules from disk. Returns an empty rule store if the file doesn't exist.
pub fn load_rules() -> Result<RuleStore> {
    load_rules_from(&rules_path()?)
}

/// Load rules from a specific path.
pub fn load_rules_from(path: &Path) -> Result<RuleStore> {
    if !path.exists() {
        return Ok(RuleStore::new());
    }
    let content = fs::read_to_string(path).map_err(|e| Error::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    if content.trim().is_empty() {
        return Ok(RuleStore::new());
    }
    let store: RuleStore = serde_yaml::from_str(&content).map_err(|e| Error::YamlParse {
        path: path.to_path_buf(),
        source: e,
    })?;
    Ok(store)
}

/// Save rules to disk.
pub fn save_rules(store: &RuleStore) -> Result<()> {
    save_rules_to(store, &rules_path()?)
}

/// Save rules to a specific path.
pub fn save_rules_to(store: &RuleStore, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| Error::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
    }
    let content = serde_yaml::to_string(store).map_err(Error::YamlSerialize)?;
    fs::write(path, content).map_err(|e| Error::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::Profile;
    use tempfile::TempDir;

    #[test]
    fn test_roundtrip_profiles() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("profiles.yaml");

        let mut store = ProfileStore::new();
        store.set(
            "personal",
            Profile::new("Vipin", "vipin@personal.dev")
                .with_ssh_key("~/.ssh/id_personal")
                .with_hosts(vec!["github.com".into()]),
        );
        store.set(
            "work",
            Profile::new("Vipin Sharma", "vipin@company.com")
                .with_ssh_key("~/.ssh/id_work")
                .with_username("vipin-work"),
        );

        save_profiles_to(&store, &path).unwrap();
        let loaded = load_profiles_from(&path).unwrap();

        assert_eq!(loaded.profiles.len(), 2);
        assert_eq!(loaded.get("personal").unwrap().email, "vipin@personal.dev");
        assert_eq!(
            loaded.get("work").unwrap().username.as_deref(),
            Some("vipin-work")
        );
    }

    #[test]
    fn test_load_missing_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.yaml");

        let store = load_profiles_from(&path).unwrap();
        assert!(store.profiles.is_empty());
    }
}
