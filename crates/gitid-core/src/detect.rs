//! Deep auto-detection of existing Git identities, SSH keys, and config.
//!
//! Scans the developer's machine and reconstructs existing identity profiles
//! by clustering signals from multiple sources:
//!
//! 1. `~/.ssh/` — all key pairs, email from comments
//! 2. `~/.ssh/config` — Host aliases (github.com-personal, etc.)
//! 3. `~/.gitconfig` — includeIf blocks, directory→identity mappings
//! 4. Common project directories — walk repos, read .git/config
//! 5. OS keychain — stored credential entries
//!
//! Signals are clustered by email into distinct identities with confidence scores.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// =============================================================================
// Public types
// =============================================================================

/// An SSH key found on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedSshKey {
    pub path: String,
    pub pub_path: String,
    pub key_type: String,
    pub comment: String,
    pub fingerprint: Option<String>,
}

/// An identity detected from git config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedIdentity {
    pub source: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub signing_key: Option<String>,
}

/// An SSH config host alias (e.g., github.com-personal).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshHostAlias {
    pub alias: String,
    pub hostname: String,
    pub identity_file: Option<String>,
    pub user: Option<String>,
}

/// A repo found on disk with its current identity config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedRepo {
    pub path: String,
    pub name: String,
    pub remote_url: Option<String>,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
    pub ssh_command: Option<String>,
}

/// Where a signal came from — for transparency in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InferenceSource {
    SshKeyComment { key_path: String },
    GitConfigGlobal,
    GitConfigIncludeIf { pattern: String },
    SshConfigHost { alias: String },
    RepoLocalConfig { repo_path: String },
}

impl std::fmt::Display for InferenceSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InferenceSource::SshKeyComment { key_path } => write!(f, "SSH key: {}", key_path),
            InferenceSource::GitConfigGlobal => write!(f, "~/.gitconfig"),
            InferenceSource::GitConfigIncludeIf { pattern } => write!(f, "includeIf: {}", pattern),
            InferenceSource::SshConfigHost { alias } => write!(f, "SSH config: {}", alias),
            InferenceSource::RepoLocalConfig { repo_path } => write!(f, "repo: {}", repo_path),
        }
    }
}

/// A suggested profile built by clustering signals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedProfile {
    pub suggested_name: String,
    pub name: String,
    pub email: String,
    pub ssh_key: Option<String>,
    pub hosts: Vec<String>,
    pub directory_pattern: Option<String>,
    pub repos: Vec<String>,
    pub confidence: f32,
    pub sources: Vec<String>,
}

/// Full detection results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub global_identity: DetectedIdentity,
    pub conditional_identities: Vec<DetectedIdentity>,
    pub ssh_keys: Vec<DetectedSshKey>,
    pub ssh_host_aliases: Vec<SshHostAlias>,
    pub scanned_repos: Vec<ScannedRepo>,
    pub credential_helper: Option<String>,
    pub suggested_profiles: Vec<SuggestedProfile>,
}

// =============================================================================
// Internal signal type for clustering
// =============================================================================

#[derive(Debug, Clone)]
struct IdentitySignal {
    email: Option<String>,
    name: Option<String>,
    ssh_key: Option<String>,
    host: Option<String>,
    directory: Option<String>,
    repo_path: Option<String>,
    source: InferenceSource,
}

// =============================================================================
// Main entry point
// =============================================================================

/// Run the full machine scan and return clustered results.
pub fn detect_existing_setup() -> DetectionResult {
    let global_identity = detect_global_git_config();
    let conditional_identities = detect_include_if_configs();
    let ssh_keys = detect_ssh_keys();
    let ssh_host_aliases = detect_ssh_config();
    let scanned_repos = scan_common_directories();
    let credential_helper = read_git_config_global("credential.helper");

    // Collect all signals
    let mut signals: Vec<IdentitySignal> = Vec::new();

    // Signals from global config
    if let Some(ref email) = global_identity.email {
        signals.push(IdentitySignal {
            email: Some(email.clone()),
            name: global_identity.name.clone(),
            ssh_key: None,
            host: None,
            directory: None,
            repo_path: None,
            source: InferenceSource::GitConfigGlobal,
        });
    }

    // Signals from includeIf
    for cond in &conditional_identities {
        if let Some(ref email) = cond.email {
            let dir = if cond.source.starts_with("includeIf: ") {
                Some(cond.source["includeIf: ".len()..].to_string())
            } else {
                None
            };
            signals.push(IdentitySignal {
                email: Some(email.clone()),
                name: cond.name.clone(),
                ssh_key: None,
                host: None,
                directory: dir,
                repo_path: None,
                source: InferenceSource::GitConfigIncludeIf {
                    pattern: cond.source.clone(),
                },
            });
        }
    }

    // Signals from SSH key comments
    for key in &ssh_keys {
        let comment = key.comment.trim().to_string();
        if comment.contains('@') {
            signals.push(IdentitySignal {
                email: Some(comment),
                name: None,
                ssh_key: Some(key.path.clone()),
                host: None,
                directory: None,
                repo_path: None,
                source: InferenceSource::SshKeyComment {
                    key_path: key.path.clone(),
                },
            });
        }
    }

    // Signals from SSH config host aliases
    for alias in &ssh_host_aliases {
        signals.push(IdentitySignal {
            email: None,
            name: None,
            ssh_key: alias.identity_file.clone(),
            host: Some(alias.hostname.clone()),
            directory: None,
            repo_path: None,
            source: InferenceSource::SshConfigHost {
                alias: alias.alias.clone(),
            },
        });
    }

    // Signals from scanned repos
    for repo in &scanned_repos {
        if let Some(ref email) = repo.user_email {
            // Extract SSH key path from core.sshCommand if present
            let ssh_key = repo.ssh_command.as_ref().and_then(|cmd| {
                // Parse "ssh -i /path/to/key ..." → "/path/to/key"
                cmd.split_whitespace()
                    .skip_while(|&s| s != "-i")
                    .nth(1)
                    .map(|s| {
                        // Convert to ~ relative
                        if let Some(home) = dirs::home_dir() {
                            if let Ok(rel) = Path::new(s).strip_prefix(&home) {
                                return format!("~/{}", rel.display());
                            }
                        }
                        s.to_string()
                    })
            });

            signals.push(IdentitySignal {
                email: Some(email.clone()),
                name: repo.user_name.clone(),
                ssh_key,
                host: repo.remote_url.as_ref().and_then(|u| extract_host(u)),
                directory: None,
                repo_path: Some(repo.path.clone()),
                source: InferenceSource::RepoLocalConfig {
                    repo_path: repo.path.clone(),
                },
            });
        }
    }

    // Cluster signals into suggested profiles
    let suggested_profiles = cluster_signals(signals, &ssh_keys);

    DetectionResult {
        global_identity,
        conditional_identities,
        ssh_keys,
        ssh_host_aliases,
        scanned_repos,
        credential_helper,
        suggested_profiles,
    }
}

// =============================================================================
// Scanners
// =============================================================================

fn detect_global_git_config() -> DetectedIdentity {
    DetectedIdentity {
        source: "global (~/.gitconfig)".into(),
        name: read_git_config_global("user.name"),
        email: read_git_config_global("user.email"),
        signing_key: read_git_config_global("user.signingkey"),
    }
}

fn read_git_config_global(key: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--global", key])
        .output()
        .ok()?;
    if output.status.success() {
        let val = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    } else {
        None
    }
}

fn detect_include_if_configs() -> Vec<DetectedIdentity> {
    let mut identities = Vec::new();
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return identities,
    };
    let gitconfig_path = home.join(".gitconfig");
    let content = match fs::read_to_string(&gitconfig_path) {
        Ok(c) => c,
        Err(_) => return identities,
    };

    let mut current_dir: Option<String> = None;
    let mut current_path: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[includeIf") {
            if let Some(start) = trimmed.find("gitdir:") {
                let rest = &trimmed[start + 7..];
                if let Some(end) = rest.find('"') {
                    current_dir = Some(rest[..end].to_string());
                }
            }
        } else if trimmed.starts_with("path") && current_dir.is_some() {
            if let Some(eq) = trimmed.find('=') {
                current_path = Some(trimmed[eq + 1..].trim().to_string());
            }
        }

        if let (Some(ref dir), Some(ref path)) = (&current_dir, &current_path) {
            let resolved = if path.starts_with('~') {
                home.join(&path[2..])
            } else {
                PathBuf::from(path)
            };

            if let Ok(included) = fs::read_to_string(&resolved) {
                let mut name = None;
                let mut email = None;
                for iline in included.lines() {
                    let itrimmed = iline.trim();
                    if itrimmed.starts_with("name") {
                        if let Some(eq) = itrimmed.find('=') {
                            name = Some(itrimmed[eq + 1..].trim().to_string());
                        }
                    } else if itrimmed.starts_with("email") {
                        if let Some(eq) = itrimmed.find('=') {
                            email = Some(itrimmed[eq + 1..].trim().to_string());
                        }
                    }
                }
                if name.is_some() || email.is_some() {
                    identities.push(DetectedIdentity {
                        source: format!("includeIf: {}", dir),
                        name,
                        email,
                        signing_key: None,
                    });
                }
            }
            current_dir = None;
            current_path = None;
        }
    }

    identities
}

fn detect_ssh_keys() -> Vec<DetectedSshKey> {
    let mut keys = Vec::new();
    let ssh_dir = match dirs::home_dir() {
        Some(h) => h.join(".ssh"),
        None => return keys,
    };
    if !ssh_dir.exists() {
        return keys;
    }

    let entries = match fs::read_dir(&ssh_dir) {
        Ok(e) => e,
        Err(_) => return keys,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let pub_path = path.with_extension("pub");

        if !path.is_file() || !pub_path.exists() {
            continue;
        }

        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if name.ends_with(".pub")
            || name == "config"
            || name == "known_hosts"
            || name == "known_hosts.old"
            || name == "authorized_keys"
            || name.starts_with(".")
        {
            continue;
        }

        let (key_type, comment) = match fs::read_to_string(&pub_path) {
            Ok(content) => {
                let parts: Vec<&str> = content.trim().splitn(3, ' ').collect();
                (
                    parts.first().unwrap_or(&"unknown").to_string(),
                    parts.get(2).unwrap_or(&"").to_string(),
                )
            }
            Err(_) => ("unknown".into(), String::new()),
        };

        let fingerprint = Command::new("ssh-keygen")
            .args(["-l", "-f", &path.to_string_lossy()])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            });

        let display_path = to_tilde_path(&path);
        let display_pub = format!("{}.pub", display_path);

        keys.push(DetectedSshKey {
            path: display_path,
            pub_path: display_pub,
            key_type,
            comment,
            fingerprint,
        });
    }

    keys
}

/// Parse ~/.ssh/config for Host aliases.
fn detect_ssh_config() -> Vec<SshHostAlias> {
    let mut aliases = Vec::new();
    let ssh_config = match dirs::home_dir() {
        Some(h) => h.join(".ssh/config"),
        None => return aliases,
    };
    let content = match fs::read_to_string(&ssh_config) {
        Ok(c) => c,
        Err(_) => return aliases,
    };

    let mut current_alias: Option<String> = None;
    let mut current_hostname: Option<String> = None;
    let mut current_identity: Option<String> = None;
    let mut current_user: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.to_lowercase().starts_with("host ")
            && !trimmed.to_lowercase().starts_with("hostname")
        {
            // Save previous entry
            if let (Some(alias), Some(hostname)) = (current_alias.take(), current_hostname.take()) {
                aliases.push(SshHostAlias {
                    alias,
                    hostname,
                    identity_file: current_identity.take(),
                    user: current_user.take(),
                });
            }
            current_alias = Some(trimmed[5..].trim().to_string());
            current_hostname = None;
            current_identity = None;
            current_user = None;
        } else if let Some((key, value)) = parse_ssh_config_line(trimmed) {
            match key.to_lowercase().as_str() {
                "hostname" => current_hostname = Some(value),
                "identityfile" => current_identity = Some(value),
                "user" => current_user = Some(value),
                _ => {}
            }
        }
    }

    // Don't forget the last entry
    if let (Some(alias), Some(hostname)) = (current_alias, current_hostname) {
        aliases.push(SshHostAlias {
            alias,
            hostname,
            identity_file: current_identity,
            user: current_user,
        });
    }

    // Filter out wildcard hosts and the literal hostname matches
    aliases
        .into_iter()
        .filter(|a| !a.alias.contains('*') && a.alias != a.hostname)
        .collect()
}

fn parse_ssh_config_line(line: &str) -> Option<(String, String)> {
    // Handle both "Key Value" and "Key=Value" formats
    let trimmed = line.trim();
    if let Some(eq) = trimmed.find('=') {
        Some((
            trimmed[..eq].trim().to_string(),
            trimmed[eq + 1..].trim().to_string(),
        ))
    } else {
        let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
        if parts.len() == 2 {
            Some((parts[0].to_string(), parts[1].trim().to_string()))
        } else {
            None
        }
    }
}

/// Walk common project directories to find repos and read their identity config.
fn scan_common_directories() -> Vec<ScannedRepo> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Vec::new(),
    };

    // Common developer directories to scan
    let search_dirs: Vec<PathBuf> = [
        "work",
        "Work",
        "projects",
        "Projects",
        "Developer",
        "dev",
        "repos",
        "Repos",
        "src",
        "code",
        "Code",
        "oss",
        "personal",
        "hobby",
        "github",
        "gitlab",
        "bitbucket",
    ]
    .iter()
    .map(|d| home.join(d))
    .filter(|d| d.exists() && d.is_dir())
    .collect();

    let mut repos = Vec::new();

    for dir in &search_dirs {
        scan_directory_for_repos(dir, &mut repos, 2); // max depth 2
    }

    // Also check home directory itself for repos (less common but possible)
    if let Ok(entries) = fs::read_dir(&home) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join(".git").exists() {
                if let Some(repo) = read_repo_identity(&path) {
                    repos.push(repo);
                }
            }
        }
    }

    // Deduplicate by path
    repos.sort_by(|a, b| a.path.cmp(&b.path));
    repos.dedup_by(|a, b| a.path == b.path);
    repos
}

fn scan_directory_for_repos(dir: &Path, repos: &mut Vec<ScannedRepo>, depth: usize) {
    if depth == 0 {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        if path.join(".git").exists() {
            if let Some(repo) = read_repo_identity(&path) {
                repos.push(repo);
            }
        } else if depth > 1 {
            // Recurse one level deeper
            scan_directory_for_repos(&path, repos, depth - 1);
        }
    }
}

fn read_repo_identity(repo_path: &Path) -> Option<ScannedRepo> {
    let name = repo_path.file_name()?.to_string_lossy().to_string();

    let user_name = git_config_local(repo_path, "user.name");
    let user_email = git_config_local(repo_path, "user.email");
    let ssh_command = git_config_local(repo_path, "core.sshCommand");
    let remote_url = git_remote_url(repo_path);

    // Only include repos that have at least some identity config or a remote
    if user_email.is_some() || remote_url.is_some() {
        Some(ScannedRepo {
            path: to_tilde_path(repo_path),
            name,
            remote_url,
            user_name,
            user_email,
            ssh_command,
        })
    } else {
        None
    }
}

fn git_config_local(repo_path: &Path, key: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--local", key])
        .current_dir(repo_path)
        .output()
        .ok()?;
    if output.status.success() {
        let val = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    } else {
        None
    }
}

fn git_remote_url(repo_path: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(repo_path)
        .output()
        .ok()?;
    if output.status.success() {
        let val = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    } else {
        None
    }
}

// =============================================================================
// Signal clustering
// =============================================================================

/// Cluster identity signals into suggested profiles, grouped by email.
fn cluster_signals(
    signals: Vec<IdentitySignal>,
    all_keys: &[DetectedSshKey],
) -> Vec<SuggestedProfile> {
    // Group signals by email (lowercased)
    let mut clusters: HashMap<String, Vec<IdentitySignal>> = HashMap::new();

    for signal in signals {
        if let Some(ref email) = signal.email {
            let key = email.to_lowercase();
            clusters.entry(key).or_default().push(signal);
        }
    }

    // Build a name→key map from key filenames for fallback matching
    let mut name_to_key: HashMap<String, String> = HashMap::new();
    for key in all_keys {
        let filename = Path::new(&key.path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        let parts: Vec<&str> = filename.split('_').collect();
        if parts.len() >= 3 {
            let label = parts[2..].join("_");
            name_to_key.insert(label, key.path.clone());
        }
    }

    let mut profiles: Vec<SuggestedProfile> = Vec::new();

    for (email, signals) in &clusters {
        // Collect all data from signals
        let mut names: Vec<String> = Vec::new();
        let mut ssh_keys: Vec<String> = Vec::new();
        let mut hosts: Vec<String> = Vec::new();
        let mut dirs: Vec<String> = Vec::new();
        let mut repo_paths: Vec<String> = Vec::new();
        let mut sources: Vec<String> = Vec::new();

        for signal in signals {
            if let Some(ref n) = signal.name {
                if !names.contains(n) {
                    names.push(n.clone());
                }
            }
            if let Some(ref k) = signal.ssh_key {
                if !ssh_keys.contains(k) {
                    ssh_keys.push(k.clone());
                }
            }
            if let Some(ref h) = signal.host {
                if !hosts.contains(h) {
                    hosts.push(h.clone());
                }
            }
            if let Some(ref d) = signal.directory {
                if !dirs.contains(d) {
                    dirs.push(d.clone());
                }
            }
            if let Some(ref r) = signal.repo_path {
                if !repo_paths.contains(r) {
                    repo_paths.push(r.clone());
                }
            }
            sources.push(signal.source.to_string());
        }

        // Infer name
        let name = names.first().cloned().unwrap_or_default();

        // Pick the best SSH key
        let ssh_key = if !ssh_keys.is_empty() {
            Some(ssh_keys[0].clone())
        } else {
            // Try matching by naming convention
            guess_ssh_key_for_email(email, &name_to_key, all_keys)
        };

        // Infer a directory pattern from repos
        let directory_pattern = infer_directory_pattern(&dirs, &repo_paths);

        // Generate a profile name
        let suggested_name = infer_profile_name(email, &dirs, &repo_paths);

        // Confidence score (0.0 to 1.0)
        let confidence =
            compute_confidence(signals.len(), ssh_key.is_some(), !repo_paths.is_empty());

        profiles.push(SuggestedProfile {
            suggested_name,
            name,
            email: email.clone(),
            ssh_key,
            hosts,
            directory_pattern,
            repos: repo_paths,
            confidence,
            sources,
        });
    }

    // Sort by confidence descending
    profiles.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    profiles
}

fn guess_ssh_key_for_email(
    email: &str,
    name_to_key: &HashMap<String, String>,
    all_keys: &[DetectedSshKey],
) -> Option<String> {
    let email_lower = email.to_lowercase();

    // Check if any key's comment matches this email
    for key in all_keys {
        if key.comment.to_lowercase().trim() == email_lower {
            return Some(key.path.clone());
        }
    }

    // Check naming conventions
    if email_lower.contains("work")
        || email_lower.contains("company")
        || email_lower.contains("corp")
    {
        if let Some(k) = name_to_key.get("work") {
            return Some(k.clone());
        }
    }
    if email_lower.contains("personal") {
        if let Some(k) = name_to_key.get("personal") {
            return Some(k.clone());
        }
    }

    // If there's only one key, use it
    if all_keys.len() == 1 {
        return Some(all_keys[0].path.clone());
    }

    None
}

fn infer_directory_pattern(dirs: &[String], repo_paths: &[String]) -> Option<String> {
    // If we have an explicit directory from includeIf, use it
    if let Some(dir) = dirs.first() {
        let pattern = dir.trim_end_matches('/');
        return Some(format!("{}/**", pattern));
    }

    // Infer from repo paths — find common parent
    if repo_paths.len() >= 2 {
        if let Some(common) = find_common_parent(repo_paths) {
            return Some(format!("{}/**", common));
        }
    }

    None
}

fn find_common_parent(paths: &[String]) -> Option<String> {
    if paths.is_empty() {
        return None;
    }

    let first = &paths[0];
    let parts: Vec<&str> = first.split('/').collect();

    for depth in (1..parts.len()).rev() {
        let prefix: String = parts[..depth].join("/");
        if paths.iter().all(|p| p.starts_with(&prefix)) {
            // Don't return home directory as the common parent
            if prefix == "~" || prefix.ends_with("/home") {
                continue;
            }
            return Some(prefix);
        }
    }

    None
}

fn infer_profile_name(email: &str, dirs: &[String], repo_paths: &[String]) -> String {
    let email_lower = email.to_lowercase();

    // Check email domain hints
    if email_lower.contains("work")
        || email_lower.contains("company")
        || email_lower.contains("corp")
    {
        return "work".into();
    }
    if email_lower.contains("oss") || email_lower.contains("opensource") {
        return "oss".into();
    }

    // Check directory hints
    let all_paths: Vec<&str> = dirs
        .iter()
        .chain(repo_paths.iter())
        .map(|s| s.as_str())
        .collect();
    for path in &all_paths {
        let lower = path.to_lowercase();
        if lower.contains("/work/") || lower.contains("/work") {
            return "work".into();
        }
        if lower.contains("/oss/") || lower.contains("/opensource/") {
            return "oss".into();
        }
        if lower.contains("/personal/") || lower.contains("/hobby/") {
            return "personal".into();
        }
    }

    // Fall back to email-based naming
    if let Some(domain) = email_lower.split('@').nth(1) {
        if domain.contains("gmail") || domain.contains("outlook") || domain.contains("proton") {
            return "personal".into();
        }
        // Use company name from domain
        let company = domain.split('.').next().unwrap_or("work");
        return company.to_string();
    }

    "default".into()
}

fn compute_confidence(signal_count: usize, has_ssh_key: bool, has_repos: bool) -> f32 {
    let mut score: f32 = 0.0;

    // More signals = higher confidence
    score += (signal_count as f32 * 0.15).min(0.45);

    // Having an SSH key match is a strong signal
    if has_ssh_key {
        score += 0.3;
    }

    // Having repos using this identity confirms it's active
    if has_repos {
        score += 0.25;
    }

    score.min(1.0)
}

// =============================================================================
// Helpers
// =============================================================================

fn to_tilde_path(path: &Path) -> String {
    if let Some(home) = dirs::home_dir() {
        if let Ok(rel) = path.strip_prefix(&home) {
            return format!("~/{}", rel.display());
        }
    }
    path.to_string_lossy().to_string()
}

fn extract_host(url: &str) -> Option<String> {
    if url.starts_with("https://") || url.starts_with("http://") {
        url.split("://")
            .nth(1)
            .and_then(|r| r.split('/').next())
            .map(String::from)
    } else if url.contains('@') && url.contains(':') {
        url.split('@')
            .nth(1)
            .and_then(|r| r.split(':').next())
            .map(String::from)
    } else {
        None
    }
}
