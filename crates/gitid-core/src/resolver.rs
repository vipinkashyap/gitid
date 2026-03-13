//! Profile resolver — determines which profile to use based on context.
//!
//! Resolution priority (highest to lowest):
//! 1. Repo-level override (.git/config has gitid.profile)
//! 2. Directory rules (CWD matches a glob pattern)
//! 3. Remote URL patterns (origin URL matches a pattern)
//! 4. Host defaults (host → profile mapping)
//! 5. Global default (fallback profile)

use crate::error::{Error, Result};
use crate::profile::ProfileStore;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

/// A directory-based matching rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DirectoryRule {
    /// Glob pattern to match against CWD, e.g. "~/work/**"
    pub path: String,
    /// Profile name to use when matched
    pub profile: String,
}

/// A remote URL pattern matching rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RemoteRule {
    /// Glob pattern to match against remote URLs, e.g. "*github.com/my-company/*"
    pub pattern: String,
    /// Profile name to use when matched
    pub profile: String,
}

/// A host-level default rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HostRule {
    /// Hostname, e.g. "github.com"
    pub host: String,
    /// Profile name to use for this host
    pub profile: String,
}

/// Container for all matching rules, serialized to rules.yaml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleStore {
    /// Directory-based rules (highest priority)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub directory: Vec<DirectoryRule>,

    /// Remote URL pattern rules
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remote: Vec<RemoteRule>,

    /// Host-level defaults
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub host: Vec<HostRule>,

    /// Global default profile name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

/// The context available when resolving a profile.
#[derive(Debug, Clone, Default)]
pub struct ResolveContext {
    /// Current working directory (or repo root)
    pub cwd: Option<PathBuf>,
    /// The remote host (e.g., "github.com")
    pub host: Option<String>,
    /// The full remote URL (e.g., "github.com/my-company/repo.git")
    pub remote_url: Option<String>,
}

/// Result of profile resolution — which profile matched and why.
#[derive(Debug, Clone)]
pub struct ResolveResult {
    /// The name of the resolved profile
    pub profile_name: String,
    /// Why this profile was selected
    pub reason: ResolveReason,
}

/// Explains why a particular profile was resolved.
#[derive(Debug, Clone, PartialEq)]
pub enum ResolveReason {
    /// Matched a repo-level override in .git/config
    RepoOverride,
    /// Matched a directory glob pattern
    DirectoryRule(String),
    /// Matched a remote URL pattern
    RemoteRule(String),
    /// Matched a host-level default
    HostDefault(String),
    /// Fell back to the global default
    GlobalDefault,
}

impl std::fmt::Display for ResolveReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveReason::RepoOverride => write!(f, "repo override (.git/config)"),
            ResolveReason::DirectoryRule(p) => write!(f, "directory rule: {}", p),
            ResolveReason::RemoteRule(p) => write!(f, "remote URL rule: {}", p),
            ResolveReason::HostDefault(h) => write!(f, "host default: {}", h),
            ResolveReason::GlobalDefault => write!(f, "global default"),
        }
    }
}

impl RuleStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a directory rule.
    pub fn add_directory_rule(&mut self, path: impl Into<String>, profile: impl Into<String>) {
        self.directory.push(DirectoryRule {
            path: path.into(),
            profile: profile.into(),
        });
    }

    /// Add a remote URL pattern rule.
    pub fn add_remote_rule(&mut self, pattern: impl Into<String>, profile: impl Into<String>) {
        self.remote.push(RemoteRule {
            pattern: pattern.into(),
            profile: profile.into(),
        });
    }

    /// Add a host-level default rule.
    pub fn add_host_rule(&mut self, host: impl Into<String>, profile: impl Into<String>) {
        self.host.push(HostRule {
            host: host.into(),
            profile: profile.into(),
        });
    }

    /// Set the global default profile.
    pub fn set_default(&mut self, profile: impl Into<String>) {
        self.default = Some(profile.into());
    }

    /// Remove a directory rule by index.
    pub fn remove_directory_rule(&mut self, index: usize) -> Option<DirectoryRule> {
        if index < self.directory.len() {
            Some(self.directory.remove(index))
        } else {
            None
        }
    }

    /// Remove a remote rule by index.
    pub fn remove_remote_rule(&mut self, index: usize) -> Option<RemoteRule> {
        if index < self.remote.len() {
            Some(self.remote.remove(index))
        } else {
            None
        }
    }

    /// Remove a host rule by index.
    pub fn remove_host_rule(&mut self, index: usize) -> Option<HostRule> {
        if index < self.host.len() {
            Some(self.host.remove(index))
        } else {
            None
        }
    }

    /// Total number of rules across all categories.
    pub fn total_rules(&self) -> usize {
        self.directory.len() + self.remote.len() + self.host.len()
    }
}

/// Resolve which profile should be used given a context and rules.
pub fn resolve(
    context: &ResolveContext,
    rules: &RuleStore,
    profiles: &ProfileStore,
) -> Result<ResolveResult> {
    // 1. Check repo-level override
    if let Some(ref cwd) = context.cwd {
        if let Some(profile_name) = read_repo_override(cwd) {
            if profiles.contains(&profile_name) {
                return Ok(ResolveResult {
                    profile_name,
                    reason: ResolveReason::RepoOverride,
                });
            }
        }
    }

    // 2. Check directory rules
    if let Some(ref cwd) = context.cwd {
        for rule in &rules.directory {
            if match_directory(cwd, &rule.path) {
                if profiles.contains(&rule.profile) {
                    return Ok(ResolveResult {
                        profile_name: rule.profile.clone(),
                        reason: ResolveReason::DirectoryRule(rule.path.clone()),
                    });
                }
            }
        }
    }

    // 3. Check remote URL patterns
    if let Some(ref remote_url) = context.remote_url {
        for rule in &rules.remote {
            if match_glob_pattern(remote_url, &rule.pattern) {
                if profiles.contains(&rule.profile) {
                    return Ok(ResolveResult {
                        profile_name: rule.profile.clone(),
                        reason: ResolveReason::RemoteRule(rule.pattern.clone()),
                    });
                }
            }
        }
    }

    // 4. Check host defaults
    if let Some(ref host) = context.host {
        for rule in &rules.host {
            if rule.host == *host {
                if profiles.contains(&rule.profile) {
                    return Ok(ResolveResult {
                        profile_name: rule.profile.clone(),
                        reason: ResolveReason::HostDefault(rule.host.clone()),
                    });
                }
            }
        }
    }

    // 5. Global default
    if let Some(ref default) = rules.default {
        if profiles.contains(default) {
            return Ok(ResolveResult {
                profile_name: default.clone(),
                reason: ResolveReason::GlobalDefault,
            });
        }
    }

    Err(Error::NoProfileResolved)
}

/// Read the gitid.profile setting from a repo's .git/config.
fn read_repo_override(cwd: &Path) -> Option<String> {
    // Walk up to find .git directory
    let mut dir = cwd.to_path_buf();
    loop {
        let git_dir = dir.join(".git");
        if git_dir.exists() {
            // Try reading gitid.profile from git config
            let output = Command::new("git")
                .args(["config", "--local", "gitid.profile"])
                .current_dir(&dir)
                .output()
                .ok()?;
            if output.status.success() {
                let profile = String::from_utf8(output.stdout).ok()?.trim().to_string();
                if !profile.is_empty() {
                    return Some(profile);
                }
            }
            return None;
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Match a directory path against a glob pattern (with ~ expansion).
fn match_directory(cwd: &Path, pattern: &str) -> bool {
    let expanded = shellexpand::tilde(pattern);
    let pattern_str = expanded.as_ref();

    // Use glob-style matching
    if let Ok(glob_pattern) = glob::Pattern::new(pattern_str) {
        // Try matching against the canonical path
        if let Ok(canonical) = cwd.canonicalize() {
            return glob_pattern.matches_path(&canonical);
        }
        // Fall back to matching the raw path
        return glob_pattern.matches_path(cwd);
    }

    // Simple prefix match as fallback (strip trailing /**)
    let prefix = pattern_str.trim_end_matches("/**").trim_end_matches("/*");
    cwd.starts_with(prefix)
}

/// Simple glob-style pattern matching for remote URLs.
/// Supports * as wildcard for any number of characters.
fn match_glob_pattern(text: &str, pattern: &str) -> bool {
    if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
        glob_pattern.matches(text)
    } else {
        text.contains(pattern)
    }
}

/// Try to get the origin remote URL for a directory (if it's a git repo).
pub fn get_remote_url(cwd: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(cwd)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8(output.stdout).ok()?.trim().to_string())
    } else {
        None
    }
}

/// Extract the hostname from a git remote URL.
/// Handles both HTTPS (https://github.com/...) and SSH (git@github.com:...) formats.
pub fn extract_host_from_url(url: &str) -> Option<String> {
    // HTTPS format: https://github.com/user/repo.git
    if url.starts_with("https://") || url.starts_with("http://") {
        return url
            .split("://")
            .nth(1)
            .and_then(|rest| rest.split('/').next())
            .map(|h| h.to_string());
    }

    // SSH format: git@github.com:user/repo.git
    if url.contains('@') && url.contains(':') {
        return url
            .split('@')
            .nth(1)
            .and_then(|rest| rest.split(':').next())
            .map(|h| h.to_string());
    }

    // SSH format: ssh://git@github.com/user/repo.git
    if url.starts_with("ssh://") {
        return url
            .split("://")
            .nth(1)
            .and_then(|rest| rest.split('@').nth(1))
            .and_then(|rest| rest.split('/').next())
            .map(|h| h.to_string());
    }

    None
}

/// Build a full resolve context from a directory path.
pub fn build_context(cwd: &Path) -> ResolveContext {
    let remote_url = get_remote_url(cwd);
    let host = remote_url
        .as_ref()
        .and_then(|url| extract_host_from_url(url));

    ResolveContext {
        cwd: Some(cwd.to_path_buf()),
        host,
        remote_url,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::Profile;

    fn test_profiles() -> ProfileStore {
        let mut store = ProfileStore::new();
        store.set("personal", Profile::new("Vipin", "vipin@personal.dev"));
        store.set("work", Profile::new("Vipin Sharma", "vipin@company.com"));
        store.set("oss", Profile::new("Vipin", "vipin@oss.dev"));
        store
    }

    #[test]
    fn test_directory_rule_resolution() {
        let profiles = test_profiles();
        let mut rules = RuleStore::new();
        rules.add_directory_rule("/home/vipin/work/**", "work");
        rules.set_default("personal");

        let context = ResolveContext {
            cwd: Some(PathBuf::from("/home/vipin/work/project")),
            host: None,
            remote_url: None,
        };

        let result = resolve(&context, &rules, &profiles).unwrap();
        assert_eq!(result.profile_name, "work");
        assert!(matches!(result.reason, ResolveReason::DirectoryRule(_)));
    }

    #[test]
    fn test_remote_rule_resolution() {
        let profiles = test_profiles();
        let mut rules = RuleStore::new();
        rules.add_remote_rule("*bitbucket.org*", "work");
        rules.set_default("personal");

        let context = ResolveContext {
            cwd: None,
            host: Some("bitbucket.org".into()),
            remote_url: Some("https://bitbucket.org/company/repo.git".into()),
        };

        let result = resolve(&context, &rules, &profiles).unwrap();
        assert_eq!(result.profile_name, "work");
        assert!(matches!(result.reason, ResolveReason::RemoteRule(_)));
    }

    #[test]
    fn test_host_rule_resolution() {
        let profiles = test_profiles();
        let mut rules = RuleStore::new();
        rules.add_host_rule("github.com", "personal");
        rules.set_default("oss");

        let context = ResolveContext {
            cwd: None,
            host: Some("github.com".into()),
            remote_url: None,
        };

        let result = resolve(&context, &rules, &profiles).unwrap();
        assert_eq!(result.profile_name, "personal");
        assert!(matches!(result.reason, ResolveReason::HostDefault(_)));
    }

    #[test]
    fn test_global_default_fallback() {
        let profiles = test_profiles();
        let mut rules = RuleStore::new();
        rules.set_default("personal");

        let context = ResolveContext::default();
        let result = resolve(&context, &rules, &profiles).unwrap();
        assert_eq!(result.profile_name, "personal");
        assert_eq!(result.reason, ResolveReason::GlobalDefault);
    }

    #[test]
    fn test_no_profile_resolved() {
        let profiles = test_profiles();
        let rules = RuleStore::new(); // No rules at all

        let context = ResolveContext::default();
        assert!(resolve(&context, &rules, &profiles).is_err());
    }

    #[test]
    fn test_extract_host_https() {
        assert_eq!(
            extract_host_from_url("https://github.com/user/repo.git"),
            Some("github.com".into())
        );
    }

    #[test]
    fn test_extract_host_ssh() {
        assert_eq!(
            extract_host_from_url("git@github.com:user/repo.git"),
            Some("github.com".into())
        );
    }

    #[test]
    fn test_extract_host_ssh_protocol() {
        assert_eq!(
            extract_host_from_url("ssh://git@github.com/user/repo.git"),
            Some("github.com".into())
        );
    }

    #[test]
    fn test_priority_directory_over_host() {
        let profiles = test_profiles();
        let mut rules = RuleStore::new();
        rules.add_directory_rule("/home/vipin/work/**", "work");
        rules.add_host_rule("github.com", "personal");

        let context = ResolveContext {
            cwd: Some(PathBuf::from("/home/vipin/work/project")),
            host: Some("github.com".into()),
            remote_url: None,
        };

        let result = resolve(&context, &rules, &profiles).unwrap();
        assert_eq!(result.profile_name, "work");
    }

    #[test]
    fn test_rule_store_yaml_roundtrip() {
        let mut rules = RuleStore::new();
        rules.add_directory_rule("~/work/**", "work");
        rules.add_remote_rule("*bitbucket*", "work");
        rules.add_host_rule("github.com", "personal");
        rules.set_default("personal");

        let yaml = serde_yaml::to_string(&rules).unwrap();
        let loaded: RuleStore = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(loaded.directory.len(), 1);
        assert_eq!(loaded.remote.len(), 1);
        assert_eq!(loaded.host.len(), 1);
        assert_eq!(loaded.default.as_deref(), Some("personal"));
    }
}
