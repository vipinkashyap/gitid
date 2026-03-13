//! Cross-platform keychain abstraction for storing Git tokens.
//!
//! Uses the OS keychain (macOS Keychain, Windows Credential Manager,
//! Linux Secret Service) via the `keyring` crate.
//!
//! Tokens are stored with the service name "gitid" and keyed by
//! "<profile_name>:<host>", e.g. "work:github.com".

use crate::error::{Error, Result};

const SERVICE_NAME: &str = "gitid";

/// Build the keychain key for a profile + host combination.
fn keychain_key(profile_name: &str, host: &str) -> String {
    format!("{}:{}", profile_name, host)
}

/// Store a token (PAT, OAuth token, etc.) in the OS keychain.
pub fn store_token(profile_name: &str, host: &str, token: &str) -> Result<()> {
    let key = keychain_key(profile_name, host);
    let entry = keyring::Entry::new(SERVICE_NAME, &key).map_err(|e| Error::Keychain {
        operation: "create entry".into(),
        detail: e.to_string(),
    })?;
    entry.set_password(token).map_err(|e| Error::Keychain {
        operation: "store token".into(),
        detail: e.to_string(),
    })?;
    Ok(())
}

/// Retrieve a token from the OS keychain.
pub fn get_token(profile_name: &str, host: &str) -> Result<Option<String>> {
    let key = keychain_key(profile_name, host);
    let entry = keyring::Entry::new(SERVICE_NAME, &key).map_err(|e| Error::Keychain {
        operation: "create entry".into(),
        detail: e.to_string(),
    })?;

    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(Error::Keychain {
            operation: "get token".into(),
            detail: e.to_string(),
        }),
    }
}

/// Delete a token from the OS keychain.
pub fn delete_token(profile_name: &str, host: &str) -> Result<()> {
    let key = keychain_key(profile_name, host);
    let entry = keyring::Entry::new(SERVICE_NAME, &key).map_err(|e| Error::Keychain {
        operation: "create entry".into(),
        detail: e.to_string(),
    })?;

    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
        Err(e) => Err(Error::Keychain {
            operation: "delete token".into(),
            detail: e.to_string(),
        }),
    }
}

/// Check if a token exists in the keychain without retrieving it.
pub fn has_token(profile_name: &str, host: &str) -> bool {
    matches!(get_token(profile_name, host), Ok(Some(_)))
}

/// Test if a GitHub/GitLab token is valid by making an API call.
pub fn test_token(host: &str, token: &str) -> Result<bool> {
    // We use git ls-remote as a lightweight auth test
    let test_url = match host {
        "github.com" => "https://github.com",
        "gitlab.com" => "https://gitlab.com",
        "bitbucket.org" => "https://bitbucket.org",
        _ => return Ok(false), // Can't test unknown hosts
    };

    let output = std::process::Command::new("git")
        .args(["ls-remote", test_url])
        .env("GIT_ASKPASS", "echo")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env(
            "GIT_CONFIG_VALUE_0",
            format!(
                "!f() {{ echo \"username=token\"; echo \"password={}\"; }}; f",
                token
            ),
        )
        .env("GIT_CONFIG_KEY_0", "credential.helper")
        .env("GIT_CONFIG_COUNT", "1")
        .output()
        .map_err(|e| Error::CommandFailed {
            command: "git ls-remote".into(),
            source: e,
        })?;

    Ok(output.status.success())
}
