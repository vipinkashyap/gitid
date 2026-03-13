//! Integration tests for gitid-core.
//!
//! These tests create real Git repositories in temp directories and verify
//! that profile resolution, config writing, guard checks, and learning
//! all work end-to-end.

use gitid_core::{config_writer, learn, profile::Profile, profile::ProfileStore, resolver};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

// =============================================================================
// Helpers
// =============================================================================

/// Create a real git repository in a temp directory.
fn init_git_repo(dir: &Path) {
    Command::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(dir)
        .output()
        .expect("git init failed");

    // Set a default identity so git doesn't complain
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(dir)
        .output()
        .expect("git config name failed");

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(dir)
        .output()
        .expect("git config email failed");
}

/// Create a git repo with a remote URL set.
fn init_git_repo_with_remote(dir: &Path, remote_url: &str) {
    init_git_repo(dir);
    Command::new("git")
        .args(["remote", "add", "origin", remote_url])
        .current_dir(dir)
        .output()
        .expect("git remote add failed");
}

/// Read a local git config value.
fn read_git_config(dir: &Path, key: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--local", key])
        .current_dir(dir)
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Create a test profile.
fn work_profile() -> Profile {
    Profile::new("Work Dev", "dev@company.com").with_hosts(vec!["github.com".to_string()])
}

fn personal_profile() -> Profile {
    Profile::new("Personal", "me@personal.com").with_hosts(vec!["github.com".to_string()])
}

/// Create a ProfileStore with work and personal profiles.
fn test_profiles() -> ProfileStore {
    let mut store = ProfileStore::new();
    store.set("work", work_profile());
    store.set("personal", personal_profile());
    store
}

// =============================================================================
// Profile tests
// =============================================================================

#[test]
fn test_profile_creation_and_fields() {
    let p = Profile::new("John Doe", "john@example.com")
        .with_ssh_key("/home/john/.ssh/id_ed25519")
        .with_hosts(vec!["github.com".to_string(), "gitlab.com".to_string()]);

    assert_eq!(p.name, "John Doe");
    assert_eq!(p.email, "john@example.com");
    assert_eq!(p.ssh_key.as_deref(), Some("/home/john/.ssh/id_ed25519"));
    assert_eq!(p.hosts.len(), 2);
    assert!(p.matches_host("github.com"));
    assert!(p.matches_host("gitlab.com"));
    assert!(!p.matches_host("bitbucket.org"));
}

// =============================================================================
// Resolver tests
// =============================================================================

#[test]
fn test_directory_rule_resolution() {
    let tmp = TempDir::new().unwrap();
    let work_dir = tmp.path().join("work").join("project");
    fs::create_dir_all(&work_dir).unwrap();
    init_git_repo(&work_dir);

    let profiles = test_profiles();
    let mut rules = resolver::RuleStore::default();
    let canonical_work = fs::canonicalize(tmp.path().join("work")).unwrap();
    let work_pattern = format!("{}/**", canonical_work.display());
    rules.add_directory_rule(&work_pattern, "work");

    let context = resolver::build_context(&work_dir);
    let result = resolver::resolve(&context, &rules, &profiles);

    assert_eq!(
        result.ok().map(|r| r.profile_name),
        Some("work".to_string())
    );
}

#[test]
fn test_remote_rule_resolution() {
    let tmp = TempDir::new().unwrap();
    let repo_dir = tmp.path().join("repo");
    fs::create_dir_all(&repo_dir).unwrap();
    init_git_repo_with_remote(&repo_dir, "git@github.com:mycompany/project.git");

    let profiles = test_profiles();
    let mut rules = resolver::RuleStore::default();
    rules.add_remote_rule("*github.com*mycompany*", "work");

    let context = resolver::build_context(&repo_dir);
    let result = resolver::resolve(&context, &rules, &profiles);

    assert_eq!(
        result.ok().map(|r| r.profile_name),
        Some("work".to_string())
    );
}

#[test]
fn test_host_rule_resolution() {
    let tmp = TempDir::new().unwrap();
    let repo_dir = tmp.path().join("repo");
    fs::create_dir_all(&repo_dir).unwrap();
    init_git_repo_with_remote(&repo_dir, "git@gitlab.internal.com:team/project.git");

    let profiles = test_profiles();
    let mut rules = resolver::RuleStore::default();
    rules.add_host_rule("gitlab.internal.com", "work");

    let context = resolver::build_context(&repo_dir);
    let result = resolver::resolve(&context, &rules, &profiles);

    assert_eq!(
        result.ok().map(|r| r.profile_name),
        Some("work".to_string())
    );
}

#[test]
fn test_default_fallback() {
    let tmp = TempDir::new().unwrap();
    let repo_dir = tmp.path().join("random-repo");
    fs::create_dir_all(&repo_dir).unwrap();
    init_git_repo(&repo_dir);

    let profiles = test_profiles();
    let mut rules = resolver::RuleStore::default();
    rules.set_default("personal");

    let context = resolver::build_context(&repo_dir);
    let result = resolver::resolve(&context, &rules, &profiles);

    assert_eq!(
        result.ok().map(|r| r.profile_name),
        Some("personal".to_string())
    );
}

#[test]
fn test_no_match_returns_error() {
    let tmp = TempDir::new().unwrap();
    let repo_dir = tmp.path().join("repo");
    fs::create_dir_all(&repo_dir).unwrap();
    init_git_repo(&repo_dir);

    let profiles = test_profiles();
    let rules = resolver::RuleStore::default();
    let context = resolver::build_context(&repo_dir);
    let result = resolver::resolve(&context, &rules, &profiles);

    assert!(result.is_err());
}

#[test]
fn test_priority_order_directory_beats_remote() {
    let tmp = TempDir::new().unwrap();
    let repo_dir = tmp.path().join("work").join("project");
    fs::create_dir_all(&repo_dir).unwrap();
    init_git_repo_with_remote(&repo_dir, "git@github.com:personal/project.git");

    let profiles = test_profiles();
    let mut rules = resolver::RuleStore::default();
    let canonical_work = fs::canonicalize(tmp.path().join("work")).unwrap();
    let work_pattern = format!("{}/**", canonical_work.display());
    rules.add_directory_rule(&work_pattern, "work");
    rules.add_remote_rule("*github.com*personal*", "personal");

    let context = resolver::build_context(&repo_dir);
    let result = resolver::resolve(&context, &rules, &profiles);

    // Directory rule should win (higher priority)
    assert_eq!(
        result.ok().map(|r| r.profile_name),
        Some("work".to_string())
    );
}

// =============================================================================
// Config writer tests
// =============================================================================

#[test]
fn test_apply_profile_to_repo() {
    let tmp = TempDir::new().unwrap();
    let repo_dir = tmp.path().join("repo");
    fs::create_dir_all(&repo_dir).unwrap();
    init_git_repo(&repo_dir);

    let profile = work_profile();
    config_writer::apply_profile_to_repo(&profile, &repo_dir).unwrap();

    assert_eq!(
        read_git_config(&repo_dir, "user.name").as_deref(),
        Some("Work Dev")
    );
    assert_eq!(
        read_git_config(&repo_dir, "user.email").as_deref(),
        Some("dev@company.com")
    );
}

#[test]
fn test_apply_profile_with_ssh_key() {
    let tmp = TempDir::new().unwrap();
    let repo_dir = tmp.path().join("repo");
    fs::create_dir_all(&repo_dir).unwrap();
    init_git_repo(&repo_dir);

    let profile = Profile::new("Dev", "dev@company.com").with_ssh_key("/home/dev/.ssh/work_key");
    config_writer::apply_profile_to_repo(&profile, &repo_dir).unwrap();

    let ssh_command = read_git_config(&repo_dir, "core.sshCommand");
    assert!(ssh_command.is_some());
    assert!(ssh_command.unwrap().contains("work_key"));
}

#[test]
fn test_repo_profile_override() {
    let tmp = TempDir::new().unwrap();
    let repo_dir = tmp.path().join("repo");
    fs::create_dir_all(&repo_dir).unwrap();
    init_git_repo(&repo_dir);

    config_writer::set_repo_profile_override(&repo_dir, "work").unwrap();

    let override_value = read_git_config(&repo_dir, "gitid.profile");
    assert_eq!(override_value.as_deref(), Some("work"));
}

// =============================================================================
// Store tests (YAML persistence)
// =============================================================================

#[test]
fn test_profile_store_roundtrip() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();

    // Write profiles
    let profiles_path = config_dir.join("profiles.yaml");
    let mut profiles = std::collections::BTreeMap::new();
    profiles.insert("work".to_string(), work_profile());
    profiles.insert("personal".to_string(), personal_profile());

    let yaml = serde_yaml::to_string(&profiles).unwrap();
    fs::write(&profiles_path, &yaml).unwrap();

    // Read back
    let content = fs::read_to_string(&profiles_path).unwrap();
    let loaded: std::collections::BTreeMap<String, Profile> =
        serde_yaml::from_str(&content).unwrap();

    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded["work"].email, "dev@company.com");
    assert_eq!(loaded["personal"].email, "me@personal.com");
}

// =============================================================================
// Learning tests
// =============================================================================

#[test]
fn test_activity_log_write_and_read() {
    let tmp = TempDir::new().unwrap();
    let log_path = tmp.path().join("activity.jsonl");

    // Create a few events
    let event1 = learn::ResolveEvent {
        timestamp: "2025-01-01T00:00:00Z".to_string(),
        directory: Some("/home/user/work/project1".to_string()),
        remote_url: Some("github.com/company/project1".to_string()),
        host: Some("github.com".to_string()),
        profile: "work".to_string(),
        reason: "directory rule".to_string(),
    };

    let event2 = learn::ResolveEvent {
        timestamp: "2025-01-01T01:00:00Z".to_string(),
        directory: Some("/home/user/work/project2".to_string()),
        remote_url: Some("github.com/company/project2".to_string()),
        host: Some("github.com".to_string()),
        profile: "work".to_string(),
        reason: "directory rule".to_string(),
    };

    // Write events
    let line1 = serde_json::to_string(&event1).unwrap();
    let line2 = serde_json::to_string(&event2).unwrap();
    fs::write(&log_path, format!("{}\n{}\n", line1, line2)).unwrap();

    // Read back
    let content = fs::read_to_string(&log_path).unwrap();
    let events: Vec<learn::ResolveEvent> = content
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].profile, "work");
    assert_eq!(
        events[1].directory.as_deref(),
        Some("/home/user/work/project2")
    );
}

// =============================================================================
// Git repo detection tests
// =============================================================================

#[test]
fn test_is_git_repo() {
    let tmp = TempDir::new().unwrap();

    // Not a repo
    assert!(!config_writer::is_git_repo(tmp.path()));

    // Make it a repo
    init_git_repo(tmp.path());
    assert!(config_writer::is_git_repo(tmp.path()));
}

#[test]
fn test_repo_root_detection() {
    let tmp = TempDir::new().unwrap();
    init_git_repo(tmp.path());

    // Create a subdirectory
    let sub = tmp.path().join("src").join("lib");
    fs::create_dir_all(&sub).unwrap();

    // repo_root should find the root from a subdirectory
    let root = config_writer::repo_root(&sub);
    assert!(root.is_some());
    // Canonicalize both paths for comparison
    let expected = fs::canonicalize(tmp.path()).unwrap();
    let actual = fs::canonicalize(root.unwrap()).unwrap();
    assert_eq!(actual, expected);
}

// =============================================================================
// Team constraints tests
// =============================================================================

#[test]
fn test_team_config_parse() {
    let toml_content = r#"
[identity]
required_domain = "company.com"
require_signing = false

[[profiles]]
name_pattern = "Work Profile"
email_pattern = "*@company.com"
"#;

    let config: gitid_core::team::TeamConfig = toml::from_str(toml_content).unwrap();
    assert_eq!(
        config.identity.required_domain.as_deref(),
        Some("company.com")
    );
    assert!(!config.identity.require_signing);
    assert_eq!(config.profiles.len(), 1);
    assert_eq!(config.profiles[0].name_pattern, "Work Profile");
}
