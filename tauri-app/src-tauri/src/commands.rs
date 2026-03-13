//! Tauri IPC commands — bridge between the React frontend and gitid-core.
//!
//! Each #[tauri::command] function is callable from TypeScript via invoke().

use gitid_core::{config_writer, detect, guard, keychain, learn, profile::Profile, resolver, ssh, store};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tauri::Manager;

// =============================================================================
// Shared types for the frontend
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileDto {
    pub name: String,
    pub email: String,
    pub ssh_key: Option<String>,
    pub signing_key: Option<String>,
    pub signing_format: Option<String>,
    pub hosts: Vec<String>,
    pub username: Option<String>,
}

impl From<&Profile> for ProfileDto {
    fn from(p: &Profile) -> Self {
        Self {
            name: p.name.clone(),
            email: p.email.clone(),
            ssh_key: p.ssh_key.clone(),
            signing_key: p.signing_key.clone(),
            signing_format: p.signing_format.clone(),
            hosts: p.hosts.clone(),
            username: p.username.clone(),
        }
    }
}

impl From<ProfileDto> for Profile {
    fn from(dto: ProfileDto) -> Self {
        Profile {
            name: dto.name,
            email: dto.email,
            ssh_key: dto.ssh_key,
            signing_key: dto.signing_key,
            signing_format: dto.signing_format,
            hosts: dto.hosts,
            username: dto.username,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDto {
    pub id: usize,
    pub rule_type: String,
    pub pattern: String,
    pub profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesDto {
    pub rules: Vec<RuleDto>,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusDto {
    pub directory: String,
    pub profile_name: Option<String>,
    pub reason: Option<String>,
    pub profile: Option<ProfileDto>,
    pub remote_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorCheck {
    pub name: String,
    pub status: String,
    pub message: String,
    pub fix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedRepo {
    pub path: String,
    pub name: String,
    pub remote_url: Option<String>,
    pub current_profile: Option<String>,
    pub current_email: Option<String>,
}

// =============================================================================
// Profile commands
// =============================================================================

#[tauri::command]
pub fn get_profiles() -> Result<BTreeMap<String, ProfileDto>, String> {
    let s = store::load_profiles().map_err(|e| e.to_string())?;
    Ok(s.profiles.iter().map(|(k, v)| (k.clone(), ProfileDto::from(v))).collect())
}

#[tauri::command]
pub fn get_profile(name: String) -> Result<ProfileDto, String> {
    let s = store::load_profiles().map_err(|e| e.to_string())?;
    s.get(&name).map(ProfileDto::from).ok_or_else(|| format!("Profile '{}' not found", name))
}

#[tauri::command]
pub fn create_profile(name: String, profile: ProfileDto) -> Result<(), String> {
    let mut s = store::load_profiles().map_err(|e| e.to_string())?;
    if s.contains(&name) {
        return Err(format!("Profile '{}' already exists", name));
    }
    s.set(&name, Profile::from(profile));
    store::save_profiles(&s).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_profile(name: String, profile: ProfileDto) -> Result<(), String> {
    let mut s = store::load_profiles().map_err(|e| e.to_string())?;
    if !s.contains(&name) {
        return Err(format!("Profile '{}' not found", name));
    }
    s.set(&name, Profile::from(profile));
    store::save_profiles(&s).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_profile(name: String) -> Result<(), String> {
    let mut s = store::load_profiles().map_err(|e| e.to_string())?;
    s.remove(&name).ok_or_else(|| format!("Profile '{}' not found", name))?;
    store::save_profiles(&s).map_err(|e| e.to_string())
}

// =============================================================================
// Rule commands
// =============================================================================

#[tauri::command]
pub fn get_rules() -> Result<RulesDto, String> {
    let rs = store::load_rules().map_err(|e| e.to_string())?;
    let mut rules = Vec::new();
    let mut id = 0;

    for rule in &rs.directory {
        rules.push(RuleDto { id, rule_type: "directory".into(), pattern: rule.path.clone(), profile: rule.profile.clone() });
        id += 1;
    }
    for rule in &rs.remote {
        rules.push(RuleDto { id, rule_type: "remote".into(), pattern: rule.pattern.clone(), profile: rule.profile.clone() });
        id += 1;
    }
    for rule in &rs.host {
        rules.push(RuleDto { id, rule_type: "host".into(), pattern: rule.host.clone(), profile: rule.profile.clone() });
        id += 1;
    }

    Ok(RulesDto { rules, default: rs.default })
}

#[tauri::command]
pub fn add_rule(rule_type: String, pattern: String, profile: String) -> Result<(), String> {
    let profiles = store::load_profiles().map_err(|e| e.to_string())?;
    if !profiles.contains(&profile) {
        return Err(format!("Profile '{}' not found", profile));
    }
    let mut rules = store::load_rules().map_err(|e| e.to_string())?;
    match rule_type.as_str() {
        "directory" => rules.add_directory_rule(&pattern, &profile),
        "remote" => rules.add_remote_rule(&pattern, &profile),
        "host" => rules.add_host_rule(&pattern, &profile),
        _ => return Err(format!("Unknown rule type: {}", rule_type)),
    }
    store::save_rules(&rules).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_rule(rule_type: String, index: usize) -> Result<(), String> {
    let mut rules = store::load_rules().map_err(|e| e.to_string())?;
    let removed = match rule_type.as_str() {
        "directory" => rules.remove_directory_rule(index).is_some(),
        "remote" => rules.remove_remote_rule(index).is_some(),
        "host" => rules.remove_host_rule(index).is_some(),
        _ => return Err(format!("Unknown rule type: {}", rule_type)),
    };
    if !removed {
        return Err(format!("No {} rule at index {}", rule_type, index));
    }
    store::save_rules(&rules).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_default_profile(profile: String) -> Result<(), String> {
    let profiles = store::load_profiles().map_err(|e| e.to_string())?;
    if !profiles.contains(&profile) {
        return Err(format!("Profile '{}' not found", profile));
    }
    let mut rules = store::load_rules().map_err(|e| e.to_string())?;
    rules.set_default(&profile);
    store::save_rules(&rules).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reorder_rules(rule_type: String, new_order: Vec<usize>) -> Result<(), String> {
    let mut rules = store::load_rules().map_err(|e| e.to_string())?;
    match rule_type.as_str() {
        "directory" => {
            let orig = rules.directory.clone();
            rules.directory = new_order.iter().filter_map(|&i| orig.get(i).cloned()).collect();
        }
        "remote" => {
            let orig = rules.remote.clone();
            rules.remote = new_order.iter().filter_map(|&i| orig.get(i).cloned()).collect();
        }
        "host" => {
            let orig = rules.host.clone();
            rules.host = new_order.iter().filter_map(|&i| orig.get(i).cloned()).collect();
        }
        _ => return Err(format!("Unknown rule type: {}", rule_type)),
    }
    store::save_rules(&rules).map_err(|e| e.to_string())
}

// =============================================================================
// Status & diagnostics
// =============================================================================

#[tauri::command]
pub fn get_status(path: Option<String>) -> Result<StatusDto, String> {
    let cwd = match path {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir().map_err(|e| e.to_string())?,
    };
    let profiles = store::load_profiles().map_err(|e| e.to_string())?;
    let rules = store::load_rules().map_err(|e| e.to_string())?;
    let context = resolver::build_context(&cwd);

    match resolver::resolve(&context, &rules, &profiles) {
        Ok(result) => {
            let profile = profiles.get(&result.profile_name).map(ProfileDto::from);
            Ok(StatusDto {
                directory: cwd.to_string_lossy().to_string(),
                profile_name: Some(result.profile_name),
                reason: Some(result.reason.to_string()),
                profile,
                remote_url: context.remote_url,
            })
        }
        Err(_) => Ok(StatusDto {
            directory: cwd.to_string_lossy().to_string(),
            profile_name: None,
            reason: None,
            profile: None,
            remote_url: context.remote_url,
        }),
    }
}

#[tauri::command]
pub fn run_doctor() -> Result<Vec<DoctorCheck>, String> {
    let mut checks = Vec::new();

    // Credential helper
    checks.push(if config_writer::is_credential_helper_installed() {
        DoctorCheck { name: "Credential Helper".into(), status: "ok".into(), message: "Registered".into(), fix: None }
    } else {
        DoctorCheck { name: "Credential Helper".into(), status: "error".into(), message: "Not registered".into(), fix: Some("Click Install below".into()) }
    });

    // Binary
    checks.push(if which::which("git-credential-gitid").is_ok() {
        DoctorCheck { name: "Credential Binary".into(), status: "ok".into(), message: "Found in PATH".into(), fix: None }
    } else {
        DoctorCheck { name: "Credential Binary".into(), status: "error".into(), message: "Not in PATH".into(), fix: Some("cargo install git-credential-gitid".into()) }
    });

    // Profiles
    let profiles = store::load_profiles().map_err(|e| e.to_string())?;
    checks.push(if profiles.profiles.is_empty() {
        DoctorCheck { name: "Profiles".into(), status: "error".into(), message: "None configured".into(), fix: Some("Create a profile".into()) }
    } else {
        DoctorCheck { name: "Profiles".into(), status: "ok".into(), message: format!("{} configured", profiles.profiles.len()), fix: None }
    });

    // SSH keys
    for (name, profile) in &profiles.profiles {
        if let Some(ref key_path) = profile.ssh_key {
            let expanded = shellexpand::tilde(key_path);
            let path = std::path::Path::new(expanded.as_ref());
            checks.push(if path.exists() {
                DoctorCheck { name: format!("SSH [{}]", name), status: "ok".into(), message: "Key exists".into(), fix: None }
            } else {
                DoctorCheck { name: format!("SSH [{}]", name), status: "error".into(), message: "Key not found".into(), fix: Some("Generate or import key".into()) }
            });
        }
    }

    // Rules
    let rules = store::load_rules().map_err(|e| e.to_string())?;
    checks.push(if rules.total_rules() == 0 && rules.default.is_none() {
        DoctorCheck { name: "Rules".into(), status: "warning".into(), message: "None configured".into(), fix: Some("Add rules".into()) }
    } else {
        DoctorCheck { name: "Rules".into(), status: "ok".into(), message: format!("{} rule(s)", rules.total_rules()), fix: None }
    });

    // Tokens
    for (name, profile) in &profiles.profiles {
        for host in &profile.hosts {
            checks.push(if keychain::has_token(name, host) {
                DoctorCheck { name: format!("Token [{}@{}]", name, host), status: "ok".into(), message: "Stored".into(), fix: None }
            } else {
                DoctorCheck { name: format!("Token [{}@{}]", name, host), status: "warning".into(), message: "Not stored".into(), fix: Some(format!("gitid token set {} {}", name, host)) }
            });
        }
    }

    Ok(checks)
}

#[tauri::command]
pub fn install_credential_helper() -> Result<(), String> {
    config_writer::install_credential_helper().map_err(|e| e.to_string())
}

// =============================================================================
// Repo detection
// =============================================================================

#[tauri::command]
pub fn scan_repos(directory: String) -> Result<Vec<DetectedRepo>, String> {
    let dir = PathBuf::from(&directory);
    if !dir.exists() {
        return Err(format!("Directory not found: {}", directory));
    }
    let mut repos = Vec::new();
    let entries = std::fs::read_dir(&dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() && path.join(".git").exists() {
            repos.push(DetectedRepo {
                path: path.to_string_lossy().to_string(),
                name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                remote_url: resolver::get_remote_url(&path),
                current_profile: config_writer::read_local_config(&path, "gitid.profile"),
                current_email: config_writer::read_local_config(&path, "user.email"),
            });
        }
    }
    repos.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(repos)
}

#[tauri::command]
pub fn test_ssh_connection(profile_name: String) -> Result<Vec<(String, bool)>, String> {
    let profiles = store::load_profiles().map_err(|e| e.to_string())?;
    let profile = profiles.get(&profile_name).ok_or_else(|| format!("Profile '{}' not found", profile_name))?;
    let key_path_str = profile.ssh_key.as_ref().ok_or("No SSH key configured")?;
    let expanded = shellexpand::tilde(key_path_str);
    let key_path = std::path::Path::new(expanded.as_ref());
    let mut results = Vec::new();
    for host in &profile.hosts {
        let success = ssh::test_connection(host, key_path).unwrap_or(false);
        results.push((host.clone(), success));
    }
    Ok(results)
}

// =============================================================================
// Detection / import
// =============================================================================

#[tauri::command]
pub fn detect_setup() -> Result<detect::DetectionResult, String> {
    Ok(detect::detect_existing_setup())
}

#[tauri::command]
pub fn import_suggested_profile(
    name: String,
    profile: ProfileDto,
    directory_pattern: Option<String>,
) -> Result<(), String> {
    // Create the profile
    let mut s = store::load_profiles().map_err(|e| e.to_string())?;
    s.set(&name, Profile::from(profile));
    store::save_profiles(&s).map_err(|e| e.to_string())?;

    // If a directory pattern was provided, also create a rule
    if let Some(pattern) = directory_pattern {
        let mut rules = store::load_rules().map_err(|e| e.to_string())?;
        rules.add_directory_rule(&pattern, &name);
        // Set as default if it's the first profile
        if rules.default.is_none() {
            rules.set_default(&name);
        }
        store::save_rules(&rules).map_err(|e| e.to_string())?;
    }

    Ok(())
}

// =============================================================================
// Guard commands
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardStatusDto {
    pub installed: bool,
    pub verdict: String,
    pub profile: Option<String>,
    pub expected_email: Option<String>,
    pub actual_email: Option<String>,
}

#[tauri::command]
pub fn guard_status() -> Result<GuardStatusDto, String> {
    let installed = guard::is_installed();
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let verdict = guard::check(&cwd);

    let (verdict_str, profile, expected, actual) = match &verdict {
        guard::GuardVerdict::Ok { profile, email } => {
            ("ok".to_string(), Some(profile.clone()), Some(email.clone()), None)
        }
        guard::GuardVerdict::Mismatch { profile, expected_email, actual_email } => {
            ("mismatch".to_string(), Some(profile.clone()), Some(expected_email.clone()), Some(actual_email.clone()))
        }
        guard::GuardVerdict::NoProfile => ("no_profile".to_string(), None, None, None),
        guard::GuardVerdict::NotARepo => ("not_a_repo".to_string(), None, None, None),
    };

    Ok(GuardStatusDto {
        installed,
        verdict: verdict_str,
        profile,
        expected_email: expected,
        actual_email: actual,
    })
}

#[tauri::command]
pub fn guard_install() -> Result<(), String> {
    guard::install().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn guard_uninstall() -> Result<(), String> {
    guard::uninstall().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn guard_fix() -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    guard::fix_mismatch(&cwd).map_err(|e| e.to_string())
}

// =============================================================================
// Suggest / learn commands
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionDto {
    pub index: usize,
    pub rule_type: String,
    pub profile: String,
    pub pattern: String,
    pub evidence_count: usize,
    pub reason: String,
}

#[tauri::command]
pub fn get_suggestions(min_evidence: Option<usize>) -> Result<Vec<SuggestionDto>, String> {
    let min = min_evidence.unwrap_or(3);
    let suggestions = learn::suggest(min).map_err(|e| e.to_string())?;
    Ok(suggestions
        .into_iter()
        .enumerate()
        .map(|(i, s)| SuggestionDto {
            index: i,
            rule_type: s.rule_type.to_string(),
            profile: s.profile,
            pattern: s.pattern,
            evidence_count: s.evidence_count,
            reason: s.reason,
        })
        .collect())
}

#[tauri::command]
pub fn get_activity_count() -> Result<usize, String> {
    learn::event_count().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn apply_suggestion(rule_type: String, pattern: String, profile: String) -> Result<(), String> {
    let mut rules = store::load_rules().map_err(|e| e.to_string())?;
    match rule_type.as_str() {
        "directory" => rules.add_directory_rule(&pattern, &profile),
        "remote" => rules.add_remote_rule(&pattern, &profile),
        "host" => rules.add_host_rule(&pattern, &profile),
        "default" => rules.set_default(&profile),
        _ => return Err(format!("Unknown rule type: {}", rule_type)),
    }
    store::save_rules(&rules).map_err(|e| e.to_string())
}

// =============================================================================
// CLI Installation — install/check CLI binary from the bundled app
// =============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct CliStatusDto {
    pub installed: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}

#[tauri::command]
pub fn check_cli_installed() -> CliStatusDto {
    match which::which("gitid") {
        Ok(path) => {
            let version = std::process::Command::new(&path)
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string());
            CliStatusDto {
                installed: true,
                path: Some(path.display().to_string()),
                version,
            }
        }
        Err(_) => CliStatusDto {
            installed: false,
            path: None,
            version: None,
        },
    }
}

#[tauri::command]
pub fn install_cli(app_handle: tauri::AppHandle) -> Result<String, String> {
    // Determine the install directory
    let install_dir = dirs::home_dir()
        .ok_or("Cannot find home directory")?
        .join(".local")
        .join("bin");

    std::fs::create_dir_all(&install_dir)
        .map_err(|e| format!("Failed to create {}: {}", install_dir.display(), e))?;

    // Get the path to the bundled CLI sidecar
    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Cannot find resource dir: {}", e))?;

    let bundled_cli = resource_dir.join("gitid-cli");
    let bundled_cred = resource_dir.join("git-credential-gitid");

    let target_cli = install_dir.join("gitid");
    let target_cred = install_dir.join("git-credential-gitid");

    if bundled_cli.exists() {
        std::fs::copy(&bundled_cli, &target_cli)
            .map_err(|e| format!("Failed to copy gitid: {}", e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&target_cli, std::fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("Failed to set permissions: {}", e))?;
        }
    } else {
        return Err(
            "CLI binary not bundled. Install manually: cargo install --path crates/gitid-cli"
                .to_string(),
        );
    }

    if bundled_cred.exists() {
        std::fs::copy(&bundled_cred, &target_cred)
            .map_err(|e| format!("Failed to copy git-credential-gitid: {}", e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&target_cred, std::fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("Failed to set permissions: {}", e))?;
        }
    }

    Ok(install_dir.display().to_string())
}
