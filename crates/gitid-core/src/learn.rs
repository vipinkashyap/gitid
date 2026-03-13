//! Pattern learning — activity log and rule suggestions.
//!
//! Every time a profile is resolved, the event is logged.
//! `gitid suggest` analyzes the log and proposes new rules
//! that would match frequently-used patterns.

use crate::error::{Error, Result};
use crate::resolver::{DirectoryRule, HostRule, RemoteRule, RuleStore};
use crate::store;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// A single resolve event logged for pattern learning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveEvent {
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Which profile was resolved
    pub profile: String,
    /// The directory where resolution happened
    pub directory: Option<String>,
    /// The remote URL (if in a git repo)
    pub remote_url: Option<String>,
    /// The host extracted from the remote
    pub host: Option<String>,
    /// How the profile was resolved
    pub reason: String,
}

/// A suggested rule from the learning engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// What kind of rule to create
    pub rule_type: SuggestionType,
    /// The profile this rule would apply
    pub profile: String,
    /// The pattern or value for the rule
    pub pattern: String,
    /// How many log entries support this suggestion
    pub evidence_count: usize,
    /// Human-readable explanation
    pub reason: String,
}

/// Types of rules the learning engine can suggest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SuggestionType {
    Directory,
    Remote,
    Host,
    Default,
}

impl std::fmt::Display for SuggestionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SuggestionType::Directory => write!(f, "directory"),
            SuggestionType::Remote => write!(f, "remote"),
            SuggestionType::Host => write!(f, "host"),
            SuggestionType::Default => write!(f, "default"),
        }
    }
}

/// Path to the activity log file.
fn activity_log_path() -> Result<PathBuf> {
    Ok(store::config_dir()?.join("activity.jsonl"))
}

/// Log a resolve event (append to JSONL activity log).
pub fn log_event(event: &ResolveEvent) -> Result<()> {
    let path = activity_log_path()?;
    let mut line = serde_json::to_string(event).map_err(|e| Error::Io {
        path: path.clone(),
        source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
    })?;
    line.push('\n');

    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| Error::Io {
            path: path.clone(),
            source: e,
        })?;
    file.write_all(line.as_bytes()).map_err(|e| Error::Io {
        path: path.clone(),
        source: e,
    })?;

    Ok(())
}

/// Read all events from the activity log.
pub fn read_events() -> Result<Vec<ResolveEvent>> {
    let path = activity_log_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&path).map_err(|e| Error::Io {
        path: path.clone(),
        source: e,
    })?;

    let events: Vec<ResolveEvent> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    Ok(events)
}

/// Clear the activity log.
pub fn clear_log() -> Result<()> {
    let path = activity_log_path()?;
    if path.exists() {
        fs::write(&path, "").map_err(|e| Error::Io { path, source: e })?;
    }
    Ok(())
}

/// Get the number of events in the log.
pub fn event_count() -> Result<usize> {
    Ok(read_events()?.len())
}

/// Analyze the activity log and suggest new rules.
///
/// Compares observed patterns against existing rules and
/// suggests rules that would match frequent uncovered patterns.
pub fn suggest(min_evidence: usize) -> Result<Vec<Suggestion>> {
    let events = read_events()?;
    let rules = store::load_rules()?;
    let mut suggestions = Vec::new();

    // --- Directory pattern suggestions ---
    suggestions.extend(suggest_directory_rules(&events, &rules, min_evidence));

    // --- Host suggestions ---
    suggestions.extend(suggest_host_rules(&events, &rules, min_evidence));

    // --- Remote URL suggestions ---
    suggestions.extend(suggest_remote_rules(&events, &rules, min_evidence));

    // --- Default profile suggestion ---
    suggestions.extend(suggest_default(&events, &rules, min_evidence));

    // Sort by evidence count (most evidence first)
    suggestions.sort_by(|a, b| b.evidence_count.cmp(&a.evidence_count));

    Ok(suggestions)
}

/// Suggest directory rules from patterns in the log.
fn suggest_directory_rules(
    events: &[ResolveEvent],
    rules: &RuleStore,
    min_evidence: usize,
) -> Vec<Suggestion> {
    // Group directories by profile, find common parents
    let mut dir_by_profile: HashMap<String, Vec<PathBuf>> = HashMap::new();

    for event in events {
        if let Some(ref dir) = event.directory {
            dir_by_profile
                .entry(event.profile.clone())
                .or_default()
                .push(PathBuf::from(dir));
        }
    }

    let mut suggestions = Vec::new();

    for (profile, dirs) in &dir_by_profile {
        // Find common parent directories
        let parent_counts = find_common_parents(dirs);

        for (parent, count) in parent_counts {
            if count < min_evidence {
                continue;
            }

            // Build the glob pattern
            let pattern = format!("{}/**", parent.display());

            // Check if this pattern already exists
            let already_covered = rules.directory.iter().any(|r| {
                r.profile == *profile && (r.path == pattern || parent_matches(&r.path, &pattern))
            });

            if !already_covered {
                // Tilde-compress the pattern for readability
                let display_pattern = tilde_compress(&pattern);
                suggestions.push(Suggestion {
                    rule_type: SuggestionType::Directory,
                    profile: profile.clone(),
                    pattern: display_pattern.clone(),
                    evidence_count: count,
                    reason: format!(
                        "Found {} resolves to '{}' under {}",
                        count, profile, display_pattern
                    ),
                });
            }
        }
    }

    suggestions
}

/// Suggest host rules from patterns in the log.
fn suggest_host_rules(
    events: &[ResolveEvent],
    rules: &RuleStore,
    min_evidence: usize,
) -> Vec<Suggestion> {
    // Count profile usage per host
    let mut host_profile: HashMap<(String, String), usize> = HashMap::new();

    for event in events {
        if let Some(ref host) = event.host {
            *host_profile
                .entry((host.clone(), event.profile.clone()))
                .or_default() += 1;
        }
    }

    let mut suggestions = Vec::new();

    for ((host, profile), count) in &host_profile {
        if *count < min_evidence {
            continue;
        }

        let already_covered = rules
            .host
            .iter()
            .any(|r| r.host == *host && r.profile == *profile);

        if !already_covered {
            suggestions.push(Suggestion {
                rule_type: SuggestionType::Host,
                profile: profile.clone(),
                pattern: host.clone(),
                evidence_count: *count,
                reason: format!("Used '{}' for {} {} times", profile, host, count),
            });
        }
    }

    suggestions
}

/// Suggest remote URL rules from patterns in the log.
fn suggest_remote_rules(
    events: &[ResolveEvent],
    rules: &RuleStore,
    min_evidence: usize,
) -> Vec<Suggestion> {
    // Group remote URLs by profile, find common patterns (org-level)
    let mut url_by_profile: HashMap<String, Vec<String>> = HashMap::new();

    for event in events {
        if let Some(ref url) = event.remote_url {
            url_by_profile
                .entry(event.profile.clone())
                .or_default()
                .push(url.clone());
        }
    }

    let mut suggestions = Vec::new();

    for (profile, urls) in &url_by_profile {
        // Extract org-level patterns
        let org_patterns = extract_org_patterns(urls);

        for (pattern, count) in org_patterns {
            if count < min_evidence {
                continue;
            }

            let already_covered = rules
                .remote
                .iter()
                .any(|r| r.profile == *profile && r.pattern == pattern);

            if !already_covered {
                suggestions.push(Suggestion {
                    rule_type: SuggestionType::Remote,
                    profile: profile.clone(),
                    pattern: pattern.clone(),
                    evidence_count: count,
                    reason: format!(
                        "Used '{}' for remotes matching {} ({} times)",
                        profile, pattern, count
                    ),
                });
            }
        }
    }

    suggestions
}

/// Suggest a default profile if one isn't set.
fn suggest_default(
    events: &[ResolveEvent],
    rules: &RuleStore,
    min_evidence: usize,
) -> Vec<Suggestion> {
    if rules.default.is_some() {
        return vec![];
    }

    // Find the most-used profile
    let mut profile_counts: HashMap<String, usize> = HashMap::new();
    for event in events {
        *profile_counts.entry(event.profile.clone()).or_default() += 1;
    }

    if let Some((profile, count)) = profile_counts.into_iter().max_by_key(|(_, c)| *c) {
        if count >= min_evidence {
            return vec![Suggestion {
                rule_type: SuggestionType::Default,
                profile: profile.clone(),
                pattern: profile.clone(),
                evidence_count: count,
                reason: format!(
                    "'{}' is the most-used profile ({} resolves) — good default candidate",
                    profile, count
                ),
            }];
        }
    }

    vec![]
}

// ── Helpers ──────────────────────────────────────────────────

/// Find common parent directories from a set of paths.
/// Returns (parent, count) pairs where count = number of paths under that parent.
fn find_common_parents(dirs: &[PathBuf]) -> Vec<(PathBuf, usize)> {
    let mut parent_counts: HashMap<PathBuf, usize> = HashMap::new();

    for dir in dirs {
        // Walk up the directory tree, counting each ancestor
        let mut current = dir.clone();
        // We want parents 2-3 levels up from home, not home itself
        while current.pop() {
            let depth = current.components().count();
            // Skip very shallow paths (/, /home, /Users, /home/user)
            if depth <= 3 {
                break;
            }
            *parent_counts.entry(current.clone()).or_default() += 1;
        }
    }

    // Filter to parents that are meaningfully specific
    // and remove parents that are subsets of others with same count
    let mut results: Vec<(PathBuf, usize)> = parent_counts.into_iter().collect();
    results.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.0.cmp(&a.0)));

    // Keep the most specific (deepest) parent for each count tier
    let mut filtered = Vec::new();
    for (path, count) in &results {
        let dominated = filtered.iter().any(|(existing, ec): &(PathBuf, usize)| {
            *ec >= *count && path.starts_with(existing)
        });
        if !dominated {
            filtered.push((path.clone(), *count));
        }
    }

    filtered
}

/// Extract organization-level URL patterns from remote URLs.
/// e.g. "https://github.com/my-company/repo1" → "*github.com/my-company/*"
fn extract_org_patterns(urls: &[String]) -> Vec<(String, usize)> {
    let mut org_counts: HashMap<String, usize> = HashMap::new();

    for url in urls {
        if let Some(pattern) = url_to_org_pattern(url) {
            *org_counts.entry(pattern).or_default() += 1;
        }
    }

    org_counts.into_iter().collect()
}

/// Convert a remote URL to an org-level glob pattern.
fn url_to_org_pattern(url: &str) -> Option<String> {
    // HTTPS: https://github.com/org/repo → *github.com/org/*
    if url.starts_with("https://") || url.starts_with("http://") {
        let parts: Vec<&str> = url.split("://").nth(1)?.split('/').collect();
        if parts.len() >= 2 {
            return Some(format!("*{}/{}/*", parts[0], parts[1]));
        }
    }

    // SSH: git@github.com:org/repo → *github.com:org/*
    if url.contains('@') && url.contains(':') {
        let after_at = url.split('@').nth(1)?;
        let parts: Vec<&str> = after_at.split(':').collect();
        if parts.len() >= 2 {
            let org = parts[1].split('/').next()?;
            return Some(format!("*{}:{}/*", parts[0], org));
        }
    }

    None
}

/// Check if one directory pattern is a parent of another.
fn parent_matches(existing: &str, candidate: &str) -> bool {
    let existing_prefix = existing.trim_end_matches("/**").trim_end_matches("/*");
    let candidate_prefix = candidate.trim_end_matches("/**").trim_end_matches("/*");
    candidate_prefix.starts_with(existing_prefix)
}

/// Compress an absolute path by replacing $HOME with ~.
fn tilde_compress(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        if path.starts_with(home_str.as_ref()) {
            return format!("~{}", &path[home_str.len()..]);
        }
    }
    path.to_string()
}
