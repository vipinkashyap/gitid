//! Team config — `.gitid.toml` support.
//!
//! A `.gitid.toml` file in a repository root lets teams declare
//! identity constraints: which email patterns are allowed, which
//! SSH keys are expected, and optional profile mappings.
//!
//! This file is meant to be committed to the repo so every team
//! member automatically gets the right identity enforced.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Team identity configuration from `.gitid.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamConfig {
    /// Team / org name (informational)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,

    /// Identity constraints
    #[serde(default)]
    pub identity: IdentityConstraints,

    /// Optional mapping hints (profile name suggestions)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<TeamProfileHint>,
}

/// Constraints on what identity is allowed for commits in this repo.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IdentityConstraints {
    /// Allowed email patterns (glob-style).
    /// e.g. ["*@company.com", "*@subsidiary.com"]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_emails: Vec<String>,

    /// Required email domain (shorthand for ["*@domain"])
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_domain: Option<String>,

    /// Allowed SSH key fingerprints (optional — for high-security teams)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_ssh_fingerprints: Vec<String>,

    /// Whether signing is required
    #[serde(default)]
    pub require_signing: bool,

    /// Allowed signing key IDs
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_signing_keys: Vec<String>,
}

/// A hint that maps a GitID profile name to expected identity values.
/// Teams can suggest what profiles should look like.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamProfileHint {
    /// Suggested profile name pattern (e.g. "work-*", "company")
    pub name_pattern: String,
    /// Expected email pattern
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email_pattern: Option<String>,
    /// Description for the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Result of validating current identity against team config.
#[derive(Debug, Clone)]
pub struct TeamValidation {
    /// Whether the identity passes all constraints.
    pub passed: bool,
    /// Individual check results.
    pub checks: Vec<TeamCheck>,
}

/// A single check result.
#[derive(Debug, Clone)]
pub struct TeamCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

impl TeamConfig {
    /// Try to load a `.gitid.toml` from a repository.
    /// Walks up from the given path to find the repo root.
    pub fn load(repo_path: &Path) -> Option<Self> {
        let config_path = find_team_config(repo_path)?;
        let content = fs::read_to_string(&config_path).ok()?;
        toml_parse(&content)
    }

    /// Validate an email + optional SSH fingerprint against this config.
    pub fn validate(&self, email: &str, ssh_fingerprint: Option<&str>) -> TeamValidation {
        let mut checks = Vec::new();

        // Check required_domain
        if let Some(ref domain) = self.identity.required_domain {
            let passes = email.ends_with(&format!("@{}", domain));
            checks.push(TeamCheck {
                name: "required_domain".into(),
                passed: passes,
                message: if passes {
                    format!("Email {} matches required domain @{}", email, domain)
                } else {
                    format!("Email {} does not match required domain @{}", email, domain)
                },
            });
        }

        // Check allowed_emails
        if !self.identity.allowed_emails.is_empty() {
            let passes = self
                .identity
                .allowed_emails
                .iter()
                .any(|pattern| glob_match_email(email, pattern));
            checks.push(TeamCheck {
                name: "allowed_emails".into(),
                passed: passes,
                message: if passes {
                    format!("Email {} matches an allowed pattern", email)
                } else {
                    format!(
                        "Email {} does not match any allowed pattern: {:?}",
                        email, self.identity.allowed_emails
                    )
                },
            });
        }

        // Check SSH fingerprint
        if !self.identity.allowed_ssh_fingerprints.is_empty() {
            if let Some(fp) = ssh_fingerprint {
                let passes = self
                    .identity
                    .allowed_ssh_fingerprints
                    .contains(&fp.to_string());
                checks.push(TeamCheck {
                    name: "ssh_fingerprint".into(),
                    passed: passes,
                    message: if passes {
                        "SSH key fingerprint is in the allowed list".into()
                    } else {
                        format!("SSH key fingerprint {} is not in the allowed list", fp)
                    },
                });
            } else {
                checks.push(TeamCheck {
                    name: "ssh_fingerprint".into(),
                    passed: false,
                    message: "No SSH key configured but team requires specific keys".into(),
                });
            }
        }

        // Check signing requirement
        if self.identity.require_signing {
            // We can only check that the setting exists, not the actual key here
            checks.push(TeamCheck {
                name: "require_signing".into(),
                passed: false, // Will be overridden by caller if signing is configured
                message: "Team requires commit signing".into(),
            });
        }

        let passed = checks.iter().all(|c| c.passed);

        TeamValidation { passed, checks }
    }

    /// Check if this config has any constraints defined.
    pub fn has_constraints(&self) -> bool {
        self.identity.required_domain.is_some()
            || !self.identity.allowed_emails.is_empty()
            || !self.identity.allowed_ssh_fingerprints.is_empty()
            || self.identity.require_signing
    }

    /// Generate a sample `.gitid.toml` for a given domain.
    pub fn sample(team_name: &str, domain: &str) -> String {
        format!(
            r#"# GitID Team Configuration
# Commit this file to your repo to enforce identity constraints.

team = "{team}"

[identity]
required_domain = "{domain}"
# allowed_emails = ["*@{domain}", "bot+*@{domain}"]
require_signing = false

# [[profiles]]
# name_pattern = "work"
# email_pattern = "*@{domain}"
# description = "Use your {team} work profile"
"#,
            team = team_name,
            domain = domain,
        )
    }
}

/// Walk up from a path to find `.gitid.toml`.
fn find_team_config(start: &Path) -> Option<PathBuf> {
    let mut dir = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };

    loop {
        let candidate = dir.join(".gitid.toml");
        if candidate.exists() {
            return Some(candidate);
        }
        // Stop at repo root (don't go above .git)
        if dir.join(".git").exists() {
            return None;
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Simple TOML parser (we parse manually to avoid adding a toml dependency).
/// Handles the subset we need: strings, arrays of strings, bools.
fn toml_parse(content: &str) -> Option<TeamConfig> {
    let mut config = TeamConfig::default();
    let mut current_section = String::new();
    let mut in_profiles_array = false;
    let mut current_profile = TeamProfileHint {
        name_pattern: String::new(),
        email_pattern: None,
        description: None,
    };

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Section headers
        if trimmed.starts_with("[[") && trimmed.ends_with("]]") {
            // Save previous profile if any
            if in_profiles_array && !current_profile.name_pattern.is_empty() {
                config.profiles.push(current_profile.clone());
            }
            let section = &trimmed[2..trimmed.len() - 2];
            if section == "profiles" {
                in_profiles_array = true;
                current_profile = TeamProfileHint {
                    name_pattern: String::new(),
                    email_pattern: None,
                    description: None,
                };
            }
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            // Save previous profile if any
            if in_profiles_array && !current_profile.name_pattern.is_empty() {
                config.profiles.push(current_profile.clone());
                in_profiles_array = false;
            }
            current_section = trimmed[1..trimmed.len() - 1].to_string();
            continue;
        }

        // Key = value pairs
        if let Some((key, value)) = trimmed.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            if in_profiles_array {
                match key {
                    "name_pattern" => current_profile.name_pattern = parse_toml_string(value),
                    "email_pattern" => {
                        current_profile.email_pattern = Some(parse_toml_string(value))
                    }
                    "description" => current_profile.description = Some(parse_toml_string(value)),
                    _ => {}
                }
                continue;
            }

            match current_section.as_str() {
                "" => {
                    // Top-level
                    if key == "team" {
                        config.team = Some(parse_toml_string(value));
                    }
                }
                "identity" => match key {
                    "required_domain" => {
                        config.identity.required_domain = Some(parse_toml_string(value));
                    }
                    "allowed_emails" => {
                        config.identity.allowed_emails = parse_toml_string_array(value);
                    }
                    "allowed_ssh_fingerprints" => {
                        config.identity.allowed_ssh_fingerprints = parse_toml_string_array(value);
                    }
                    "require_signing" => {
                        config.identity.require_signing = value.trim() == "true";
                    }
                    "allowed_signing_keys" => {
                        config.identity.allowed_signing_keys = parse_toml_string_array(value);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    // Save last profile
    if in_profiles_array && !current_profile.name_pattern.is_empty() {
        config.profiles.push(current_profile);
    }

    Some(config)
}

/// Parse a TOML string value (strip quotes).
fn parse_toml_string(value: &str) -> String {
    let v = value.trim();
    if (v.starts_with('"') && v.ends_with('"')) || (v.starts_with('\'') && v.ends_with('\'')) {
        v[1..v.len() - 1].to_string()
    } else {
        v.to_string()
    }
}

/// Parse a TOML array of strings like ["a", "b", "c"].
fn parse_toml_string_array(value: &str) -> Vec<String> {
    let v = value.trim();
    if !v.starts_with('[') || !v.ends_with(']') {
        return vec![];
    }
    let inner = &v[1..v.len() - 1];
    inner
        .split(',')
        .map(|s| parse_toml_string(s.trim()))
        .filter(|s| !s.is_empty())
        .collect()
}

/// Glob-style email matching.
/// Supports * as wildcard for any number of characters.
fn glob_match_email(email: &str, pattern: &str) -> bool {
    if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
        glob_pattern.matches(email)
    } else {
        email == pattern
    }
}
