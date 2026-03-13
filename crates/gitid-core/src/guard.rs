//! Identity guard — pre-commit hook that prevents wrong-email commits.
//!
//! The guard installs a global pre-commit hook via `core.hooksPath`.
//! Before every commit, it checks that the repo's `user.email` matches
//! the email expected by the resolved GitID profile. On mismatch it
//! can block, warn, or auto-fix depending on configuration.

use crate::error::{Error, Result};
use crate::store;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Guard check outcome.
#[derive(Debug, Clone, PartialEq)]
pub enum GuardVerdict {
    /// Email matches the expected profile — proceed.
    Ok {
        profile: String,
        email: String,
    },
    /// Email mismatch detected.
    Mismatch {
        profile: String,
        expected_email: String,
        actual_email: String,
    },
    /// No profile resolved — nothing to enforce.
    NoProfile,
    /// Not inside a git repository.
    NotARepo,
}

/// Where the global hooks directory lives.
fn hooks_dir() -> Result<PathBuf> {
    let dir = store::config_dir()?.join("hooks");
    Ok(dir)
}

/// Check whether the identity guard is installed.
pub fn is_installed() -> bool {
    // Check if core.hooksPath is set to our hooks dir
    let output = Command::new("git")
        .args(["config", "--global", "core.hooksPath"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if let Ok(our_dir) = hooks_dir() {
                path == our_dir.to_string_lossy()
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Install the identity guard (global pre-commit hook).
///
/// Creates `~/.config/gitid/hooks/pre-commit` and sets
/// `core.hooksPath` globally. Any existing repo-level hooks
/// are still run via a pass-through mechanism.
pub fn install() -> Result<()> {
    let dir = hooks_dir()?;
    fs::create_dir_all(&dir).map_err(|e| Error::Io {
        path: dir.clone(),
        source: e,
    })?;

    let hook_path = dir.join("pre-commit");
    let script = generate_hook_script();

    fs::write(&hook_path, script).map_err(|e| Error::Io {
        path: hook_path.clone(),
        source: e,
    })?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&hook_path, perms).map_err(|e| Error::Io {
            path: hook_path.clone(),
            source: e,
        })?;
    }

    // Set core.hooksPath globally
    let status = Command::new("git")
        .args([
            "config",
            "--global",
            "core.hooksPath",
            &dir.to_string_lossy(),
        ])
        .status()
        .map_err(|e| Error::CommandFailed {
            command: "git config --global core.hooksPath".into(),
            source: e,
        })?;

    if !status.success() {
        return Err(Error::GitConfigFailed {
            key: "core.hooksPath".into(),
            stderr: "failed to set".into(),
        });
    }

    Ok(())
}

/// Uninstall the identity guard (removes core.hooksPath and hook files).
pub fn uninstall() -> Result<()> {
    // Unset core.hooksPath
    let _ = Command::new("git")
        .args(["config", "--global", "--unset", "core.hooksPath"])
        .status();

    // Remove the hooks directory
    let dir = hooks_dir()?;
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| Error::Io {
            path: dir,
            source: e,
        })?;
    }

    Ok(())
}

/// Run the guard check for a given repo directory.
/// Returns what the hook would decide.
pub fn check(repo_path: &Path) -> GuardVerdict {
    use crate::resolver;

    // Check we're in a repo
    if !repo_path.join(".git").exists() {
        // Try to find .git up the tree
        let mut dir = repo_path.to_path_buf();
        let mut found = false;
        while dir.pop() {
            if dir.join(".git").exists() {
                found = true;
                break;
            }
        }
        if !found {
            return GuardVerdict::NotARepo;
        }
    }

    // Load profiles and rules
    let profiles = match store::load_profiles() {
        Ok(p) => p,
        Err(_) => return GuardVerdict::NoProfile,
    };
    let rules = match store::load_rules() {
        Ok(r) => r,
        Err(_) => return GuardVerdict::NoProfile,
    };

    // Resolve expected profile
    let context = resolver::build_context(repo_path);
    let result = match resolver::resolve(&context, &rules, &profiles) {
        Ok(r) => r,
        Err(_) => return GuardVerdict::NoProfile,
    };

    let expected_profile = match profiles.get(&result.profile_name) {
        Some(p) => p,
        None => return GuardVerdict::NoProfile,
    };

    // Read the actual email configured in the repo
    let actual_email = read_repo_email(repo_path).unwrap_or_default();

    if actual_email.is_empty() || actual_email == expected_profile.email {
        GuardVerdict::Ok {
            profile: result.profile_name,
            email: expected_profile.email.clone(),
        }
    } else {
        GuardVerdict::Mismatch {
            profile: result.profile_name,
            expected_email: expected_profile.email.clone(),
            actual_email,
        }
    }
}

/// Read the user.email from a repo (local or global).
fn read_repo_email(repo_path: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["config", "user.email"])
        .current_dir(repo_path)
        .output()
        .ok()?;

    if output.status.success() {
        let email = String::from_utf8(output.stdout).ok()?.trim().to_string();
        if !email.is_empty() {
            return Some(email);
        }
    }
    None
}

/// Fix a mismatch by applying the correct profile to the repo.
pub fn fix_mismatch(repo_path: &Path) -> Result<()> {
    use crate::{config_writer, resolver};

    let profiles = store::load_profiles()?;
    let rules = store::load_rules()?;
    let context = resolver::build_context(repo_path);
    let result = resolver::resolve(&context, &rules, &profiles)?;

    if let Some(profile) = profiles.get(&result.profile_name) {
        config_writer::apply_profile_to_repo(profile, repo_path)?;
    }

    Ok(())
}

/// Generate the pre-commit hook shell script.
fn generate_hook_script() -> String {
    r#"#!/usr/bin/env bash
# GitID Identity Guard — pre-commit hook
# Ensures the current repo's email matches the expected GitID profile.
# Installed by: gitid guard install

set -euo pipefail

# Skip if gitid is not installed
if ! command -v gitid &>/dev/null; then
    exit 0
fi

# Skip if GITID_GUARD_SKIP is set (escape hatch)
if [[ -n "${GITID_GUARD_SKIP:-}" ]]; then
    exit 0
fi

# Get the expected profile info as JSON
GUARD_JSON=$(gitid guard check --json 2>/dev/null || echo '{"verdict":"error"}')
VERDICT=$(echo "$GUARD_JSON" | grep -o '"verdict":"[^"]*"' | head -1 | cut -d'"' -f4)

case "$VERDICT" in
    ok|no_profile|not_a_repo|error)
        # All good or nothing to enforce
        ;;
    mismatch)
        EXPECTED=$(echo "$GUARD_JSON" | grep -o '"expected_email":"[^"]*"' | cut -d'"' -f4)
        ACTUAL=$(echo "$GUARD_JSON" | grep -o '"actual_email":"[^"]*"' | cut -d'"' -f4)
        PROFILE=$(echo "$GUARD_JSON" | grep -o '"profile":"[^"]*"' | cut -d'"' -f4)

        echo ""
        echo "⚠️  GitID Identity Guard"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "  Expected: $EXPECTED (profile: $PROFILE)"
        echo "  Actual:   $ACTUAL"
        echo ""

        # In non-interactive mode, just warn
        if [[ ! -t 0 ]]; then
            echo "  ⚠ Non-interactive — proceeding with warning"
            exit 0
        fi

        echo "  [s]witch to correct email and commit"
        echo "  [c]ontinue with current email anyway"
        echo "  [a]bort commit"
        echo ""
        read -r -p "  Choice [s/c/a]: " choice </dev/tty

        case "$choice" in
            s|S)
                gitid guard fix 2>/dev/null
                echo "  ✓ Switched to $EXPECTED — continuing commit"
                ;;
            c|C)
                echo "  → Continuing with $ACTUAL"
                ;;
            *)
                echo "  ✗ Commit aborted"
                exit 1
                ;;
        esac
        ;;
esac

# Run repo-level pre-commit hook if it exists
REPO_HOOK="$(git rev-parse --show-toplevel 2>/dev/null)/.git/hooks/pre-commit"
if [[ -x "$REPO_HOOK" ]]; then
    exec "$REPO_HOOK" "$@"
fi
"#
    .to_string()
}
