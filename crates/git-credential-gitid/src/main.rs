//! git-credential-gitid — Git credential helper with automatic profile resolution.
//!
//! This binary implements the standard Git credential helper protocol:
//! https://git-scm.com/docs/git-credential#IOFMT
//!
//! Git invokes this as:
//!   git-credential-gitid get    — request credentials
//!   git-credential-gitid store  — store credentials after successful auth
//!   git-credential-gitid erase  — remove credentials after failed auth
//!
//! The key differentiator: on `get`, this helper also injects per-profile
//! SSH keys and identity (user.name, user.email) into the repo's local config.

use gitid_core::{config_writer, keychain, resolver, store};
use std::collections::HashMap;
use std::env;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: git-credential-gitid <get|store|erase>");
        std::process::exit(1);
    }

    let operation = &args[1];
    let result = match operation.as_str() {
        "get" => handle_get(),
        "store" => handle_store(),
        "erase" => handle_erase(),
        _ => {
            eprintln!("Unknown operation: {}", operation);
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("gitid: {}", e);
        std::process::exit(1);
    }
}

/// Parse the credential helper input from stdin.
/// Format is key=value pairs, one per line, terminated by a blank line.
fn parse_input() -> HashMap<String, String> {
    let stdin = io::stdin();
    let mut fields = HashMap::new();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.is_empty() {
            break;
        }

        if let Some((key, value)) = line.split_once('=') {
            fields.insert(key.to_string(), value.to_string());
        }
    }

    fields
}

/// Get the current working directory, trying GIT_WORK_TREE first,
/// then the parent process's CWD, then falling back to the actual CWD.
fn get_working_dir() -> PathBuf {
    // Try GIT_WORK_TREE environment variable
    if let Ok(work_tree) = env::var("GIT_WORK_TREE") {
        return PathBuf::from(work_tree);
    }

    // Try GIT_DIR and go up one level
    if let Ok(git_dir) = env::var("GIT_DIR") {
        let path = PathBuf::from(&git_dir);
        if let Some(parent) = path.parent() {
            if parent.exists() && parent != path {
                return parent.to_path_buf();
            }
        }
    }

    // Fall back to actual CWD
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Handle the `get` operation: resolve profile and return credentials.
///
/// This is the critical path. On every `get`:
/// 1. Determine the current directory context
/// 2. Resolve which profile to use
/// 3. Return credentials from the keychain
/// 4. ALSO inject SSH key and identity into repo config (the differentiator)
fn handle_get() -> Result<(), Box<dyn std::error::Error>> {
    let fields = parse_input();
    let host = fields.get("protocol").and_then(|_| fields.get("host"));
    let path = fields.get("path");

    let host_str = match host {
        Some(h) => h.clone(),
        None => return Ok(()), // No host = nothing we can do
    };

    // Load config
    let profiles = store::load_profiles()?;
    let rules = store::load_rules()?;

    // Build context for resolution
    let cwd = get_working_dir();
    let remote_url = path
        .map(|p| format!("https://{}/{}", host_str, p))
        .or_else(|| resolver::get_remote_url(&cwd));

    let context = resolver::ResolveContext {
        cwd: Some(cwd.clone()),
        host: Some(host_str.clone()),
        remote_url,
    };

    // Resolve profile
    let result = match resolver::resolve(&context, &rules, &profiles) {
        Ok(r) => r,
        Err(_) => return Ok(()), // No profile resolved, let git try other helpers
    };

    let profile = match profiles.get(&result.profile_name) {
        Some(p) => p,
        None => return Ok(()),
    };

    // --- THE DIFFERENTIATOR ---
    // Apply SSH key and identity to the repo's local config.
    // This is what makes GitID special: even SSH operations get the right key
    // automatically, and every repo gets the right name/email without manual setup.
    if config_writer::is_git_repo(&cwd) {
        // Best-effort: don't fail the credential get if config writing fails
        let _ = config_writer::apply_profile_to_repo(profile, &cwd);
    }

    // Return credentials for HTTPS auth
    if let Ok(Some(token)) = keychain::get_token(&result.profile_name, &host_str) {
        let stdout = io::stdout();
        let mut out = stdout.lock();

        // Use the profile's username, or fall back to common defaults
        let username = profile.username.as_deref().unwrap_or("x-access-token"); // GitHub PAT format

        writeln!(out, "username={}", username)?;
        writeln!(out, "password={}", token)?;
        writeln!(out, "quit=true")?;
    }

    Ok(())
}

/// Handle the `store` operation: save credentials after successful auth.
fn handle_store() -> Result<(), Box<dyn std::error::Error>> {
    let fields = parse_input();
    let host = match fields.get("host") {
        Some(h) => h.clone(),
        None => return Ok(()),
    };
    let password = match fields.get("password") {
        Some(p) => p.clone(),
        None => return Ok(()),
    };

    // Resolve which profile this should be stored under
    let profiles = store::load_profiles()?;
    let rules = store::load_rules()?;
    let cwd = get_working_dir();

    let context = resolver::ResolveContext {
        cwd: Some(cwd),
        host: Some(host.clone()),
        remote_url: None,
    };

    if let Ok(result) = resolver::resolve(&context, &rules, &profiles) {
        keychain::store_token(&result.profile_name, &host, &password)?;
    }

    Ok(())
}

/// Handle the `erase` operation: remove credentials after failed auth.
fn handle_erase() -> Result<(), Box<dyn std::error::Error>> {
    let fields = parse_input();
    let host = match fields.get("host") {
        Some(h) => h.clone(),
        None => return Ok(()),
    };

    // Try to remove token for the resolved profile
    let profiles = store::load_profiles()?;
    let rules = store::load_rules()?;
    let cwd = get_working_dir();

    let context = resolver::ResolveContext {
        cwd: Some(cwd),
        host: Some(host.clone()),
        remote_url: None,
    };

    if let Ok(result) = resolver::resolve(&context, &rules, &profiles) {
        keychain::delete_token(&result.profile_name, &host)?;
    }

    Ok(())
}
