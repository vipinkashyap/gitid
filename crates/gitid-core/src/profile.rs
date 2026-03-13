//! Profile data model for GitID.
//!
//! A profile represents a complete Git identity: name, email, SSH key,
//! optional signing key, and host associations.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// A single Git identity profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    /// Display name for git commits (user.name)
    pub name: String,

    /// Email for git commits (user.email)
    pub email: String,

    /// Path to the SSH private key for this profile
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh_key: Option<String>,

    /// Path to GPG or SSH signing key (for commit signing)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signing_key: Option<String>,

    /// Signing format: "gpg", "ssh", or "x509"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signing_format: Option<String>,

    /// Default host associations for this profile
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hosts: Vec<String>,

    /// Username for HTTPS authentication (e.g., GitHub username)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

/// Top-level container for all profiles, serialized to profiles.yaml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileStore {
    /// Map of profile name → profile data
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
}

impl Profile {
    /// Create a new profile with the minimum required fields.
    pub fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            email: email.into(),
            ssh_key: None,
            signing_key: None,
            signing_format: None,
            hosts: Vec::new(),
            username: None,
        }
    }

    /// Set the SSH key path.
    pub fn with_ssh_key(mut self, path: impl Into<String>) -> Self {
        self.ssh_key = Some(path.into());
        self
    }

    /// Set the signing key path and format.
    pub fn with_signing(mut self, key: impl Into<String>, format: impl Into<String>) -> Self {
        self.signing_key = Some(key.into());
        self.signing_format = Some(format.into());
        self
    }

    /// Add host associations.
    pub fn with_hosts(mut self, hosts: Vec<String>) -> Self {
        self.hosts = hosts;
        self
    }

    /// Set the HTTPS username.
    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Resolve the SSH key path, expanding ~ and environment variables.
    pub fn resolved_ssh_key(&self) -> Option<PathBuf> {
        self.ssh_key.as_ref().map(|p| {
            let expanded = shellexpand::tilde(p);
            PathBuf::from(expanded.as_ref())
        })
    }

    /// Check if this profile is associated with a given host.
    pub fn matches_host(&self, host: &str) -> bool {
        self.hosts.iter().any(|h| h == host)
    }
}

impl ProfileStore {
    /// Create an empty profile store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or update a profile.
    pub fn set(&mut self, name: impl Into<String>, profile: Profile) {
        self.profiles.insert(name.into(), profile);
    }

    /// Get a profile by name.
    pub fn get(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }

    /// Remove a profile by name. Returns the removed profile if it existed.
    pub fn remove(&mut self, name: &str) -> Option<Profile> {
        self.profiles.remove(name)
    }

    /// List all profile names.
    pub fn names(&self) -> Vec<&str> {
        self.profiles.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a profile exists.
    pub fn contains(&self, name: &str) -> bool {
        self.profiles.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_builder() {
        let profile = Profile::new("Vipin", "vipin@test.dev")
            .with_ssh_key("~/.ssh/id_ed25519")
            .with_hosts(vec!["github.com".into()])
            .with_username("vipin");

        assert_eq!(profile.name, "Vipin");
        assert_eq!(profile.email, "vipin@test.dev");
        assert_eq!(profile.ssh_key.as_deref(), Some("~/.ssh/id_ed25519"));
        assert!(profile.matches_host("github.com"));
        assert!(!profile.matches_host("gitlab.com"));
    }

    #[test]
    fn test_profile_store_crud() {
        let mut store = ProfileStore::new();
        let profile = Profile::new("Vipin", "vipin@test.dev");

        store.set("personal", profile.clone());
        assert!(store.contains("personal"));
        assert_eq!(store.get("personal").unwrap().email, "vipin@test.dev");

        store.remove("personal");
        assert!(!store.contains("personal"));
    }

    #[test]
    fn test_resolved_ssh_key() {
        let profile = Profile::new("Test", "test@test.dev")
            .with_ssh_key("~/.ssh/id_ed25519");

        let resolved = profile.resolved_ssh_key().unwrap();
        assert!(resolved.to_str().unwrap().contains(".ssh/id_ed25519"));
        assert!(!resolved.to_str().unwrap().contains("~"));
    }
}
