//! Git config manipulation — sets local and global git config values.
//!
//! Handles writing:
//! - Per-repo identity (user.name, user.email, core.sshCommand)
//! - Global credential helper registration
//! - Commit signing configuration

use crate::error::{Error, Result};
use crate::profile::Profile;
use crate::ssh;
use std::path::Path;
use std::process::Command;

/// Apply a profile's identity settings to a repo's local git config.
/// This sets user.name, user.email, core.sshCommand, and optionally signing config.
pub fn apply_profile_to_repo(profile: &Profile, repo_path: &Path) -> Result<()> {
    // Set user.name
    git_config_local(repo_path, "user.name", &profile.name)?;

    // Set user.email
    git_config_local(repo_path, "user.email", &profile.email)?;

    // Set SSH command if an SSH key is configured
    if let Some(ref ssh_key) = profile.ssh_key {
        let expanded = shellexpand::tilde(ssh_key);
        let key_path = Path::new(expanded.as_ref());
        let ssh_cmd = ssh::ssh_command_for_key(key_path);
        git_config_local(repo_path, "core.sshCommand", &ssh_cmd)?;
    }

    // Set signing config if configured
    if let Some(ref signing_key) = profile.signing_key {
        let expanded = shellexpand::tilde(signing_key);
        git_config_local(repo_path, "user.signingkey", expanded.as_ref())?;
        git_config_local(repo_path, "commit.gpgsign", "true")?;

        if let Some(ref format) = profile.signing_format {
            git_config_local(repo_path, "gpg.format", format)?;
        }
    }

    Ok(())
}

/// Set a repo-level gitid.profile override.
pub fn set_repo_profile_override(repo_path: &Path, profile_name: &str) -> Result<()> {
    git_config_local(repo_path, "gitid.profile", profile_name)
}

/// Remove the repo-level gitid.profile override.
pub fn remove_repo_profile_override(repo_path: &Path) -> Result<()> {
    git_config_unset_local(repo_path, "gitid.profile")
}

/// Register gitid as the global credential helper.
pub fn install_credential_helper() -> Result<()> {
    git_config_global("credential.helper", "gitid")
}

/// Remove gitid as the global credential helper.
pub fn uninstall_credential_helper() -> Result<()> {
    git_config_unset_global("credential.helper")
}

/// Check if gitid is currently registered as the credential helper.
pub fn is_credential_helper_installed() -> bool {
    let output = Command::new("git")
        .args(["config", "--global", "credential.helper"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let val = String::from_utf8_lossy(&out.stdout);
            val.trim() == "gitid"
        }
        _ => false,
    }
}

/// Read a git config value from a repo's local config.
pub fn read_local_config(repo_path: &Path, key: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--local", key])
        .current_dir(repo_path)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Read a git config value from the global config.
pub fn read_global_config(key: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--global", key])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Check if a path is inside a git repository.
pub fn is_git_repo(path: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get the root of the git repository containing the given path.
pub fn repo_root(path: &Path) -> Option<std::path::PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .output()
        .ok()?;

    if output.status.success() {
        Some(
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .into(),
        )
    } else {
        None
    }
}

// --- Internal helpers ---

fn git_config_local(repo_path: &Path, key: &str, value: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["config", "--local", key, value])
        .current_dir(repo_path)
        .output()
        .map_err(|e| Error::CommandFailed {
            command: format!("git config --local {} {}", key, value),
            source: e,
        })?;

    if output.status.success() {
        Ok(())
    } else {
        Err(Error::GitConfigFailed {
            key: key.into(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

fn git_config_unset_local(repo_path: &Path, key: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["config", "--local", "--unset", key])
        .current_dir(repo_path)
        .output()
        .map_err(|e| Error::CommandFailed {
            command: format!("git config --local --unset {}", key),
            source: e,
        })?;

    // Exit code 5 means the key wasn't set — that's fine
    if output.status.success() || output.status.code() == Some(5) {
        Ok(())
    } else {
        Err(Error::GitConfigFailed {
            key: key.into(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

fn git_config_global(key: &str, value: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["config", "--global", key, value])
        .output()
        .map_err(|e| Error::CommandFailed {
            command: format!("git config --global {} {}", key, value),
            source: e,
        })?;

    if output.status.success() {
        Ok(())
    } else {
        Err(Error::GitConfigFailed {
            key: key.into(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

fn git_config_unset_global(key: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["config", "--global", "--unset", key])
        .output()
        .map_err(|e| Error::CommandFailed {
            command: format!("git config --global --unset {}", key),
            source: e,
        })?;

    if output.status.success() || output.status.code() == Some(5) {
        Ok(())
    } else {
        Err(Error::GitConfigFailed {
            key: key.into(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}
