//! SSH key management — generation, testing, and validation.

use crate::error::{Error, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Information about an SSH key pair.
#[derive(Debug, Clone)]
pub struct SshKeyInfo {
    /// Path to the private key
    pub private_key: PathBuf,
    /// Path to the public key
    pub public_key: PathBuf,
    /// Key type (e.g., "ed25519", "rsa")
    pub key_type: Option<String>,
    /// Key fingerprint
    pub fingerprint: Option<String>,
}

/// Generate a new SSH key pair for a profile.
pub fn generate_key(email: &str, key_path: &Path, key_type: &str) -> Result<SshKeyInfo> {
    // Ensure the parent directory exists
    if let Some(parent) = key_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| Error::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
    }

    // Don't overwrite existing keys
    if key_path.exists() {
        return Err(Error::SshKeyExists(key_path.to_path_buf()));
    }

    let output = Command::new("ssh-keygen")
        .args([
            "-t",
            key_type,
            "-C",
            email,
            "-f",
            &key_path.to_string_lossy(),
            "-N",
            "", // empty passphrase (user can add later)
        ])
        .output()
        .map_err(|e| Error::CommandFailed {
            command: "ssh-keygen".into(),
            source: e,
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(Error::SshKeyGenFailed(stderr));
    }

    get_key_info(key_path)
}

/// Get information about an existing SSH key.
pub fn get_key_info(private_key_path: &Path) -> Result<SshKeyInfo> {
    let public_key = private_key_path.with_extension("pub");

    let fingerprint = get_fingerprint(private_key_path).ok();
    let key_type = get_key_type(&public_key).ok();

    Ok(SshKeyInfo {
        private_key: private_key_path.to_path_buf(),
        public_key,
        key_type,
        fingerprint,
    })
}

/// Get the fingerprint of an SSH key.
fn get_fingerprint(key_path: &Path) -> Result<String> {
    let output = Command::new("ssh-keygen")
        .args(["-l", "-f", &key_path.to_string_lossy()])
        .output()
        .map_err(|e| Error::CommandFailed {
            command: "ssh-keygen".into(),
            source: e,
        })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(Error::SshKeyGenFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

/// Detect the key type from a public key file.
fn get_key_type(public_key_path: &Path) -> Result<String> {
    if !public_key_path.exists() {
        return Err(Error::SshKeyNotFound(public_key_path.to_path_buf()));
    }

    let content = fs::read_to_string(public_key_path).map_err(|e| Error::Io {
        path: public_key_path.to_path_buf(),
        source: e,
    })?;

    // Public key format: "ssh-ed25519 AAAA... comment"
    if let Some(key_type) = content.split_whitespace().next() {
        Ok(key_type
            .trim_start_matches("ssh-")
            .trim_start_matches("ecdsa-")
            .to_string())
    } else {
        Ok("unknown".into())
    }
}

/// Test SSH connectivity to a host using a specific key.
pub fn test_connection(host: &str, key_path: &Path) -> Result<bool> {
    let output = Command::new("ssh")
        .args([
            "-T",
            "-i",
            &key_path.to_string_lossy(),
            "-o",
            "IdentitiesOnly=yes",
            "-o",
            "StrictHostKeyChecking=accept-new",
            "-o",
            "ConnectTimeout=10",
            &format!("git@{}", host),
        ])
        .output()
        .map_err(|e| Error::CommandFailed {
            command: "ssh".into(),
            source: e,
        })?;

    // GitHub returns exit code 1 with "Hi <user>!" message on success
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stdout, stderr);

    Ok(combined.contains("successfully authenticated")
        || combined.contains("Hi ")
        || combined.contains("Welcome to")
        || combined.contains("logged in as")
        || output.status.success())
}

/// Validate that an SSH key file exists and has correct permissions.
pub fn validate_key(key_path: &Path) -> Result<()> {
    let lossy = key_path.to_string_lossy();
    let expanded = shellexpand::tilde(&lossy);
    let path = Path::new(expanded.as_ref());

    if !path.exists() {
        return Err(Error::SshKeyNotFound(path.to_path_buf()));
    }

    // Check that the public key also exists
    let pub_path = path.with_extension("pub");
    if !pub_path.exists() {
        return Err(Error::SshKeyNotFound(pub_path));
    }

    // On Unix, check permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta = fs::metadata(path).map_err(|e| Error::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        let mode = meta.permissions().mode();
        if mode & 0o077 != 0 {
            return Err(Error::SshKeyPermissions {
                path: path.to_path_buf(),
                mode,
            });
        }
    }

    Ok(())
}

/// Get the SSH command string for a specific key, suitable for core.sshCommand.
pub fn ssh_command_for_key(key_path: &Path) -> String {
    format!(
        "ssh -i {} -o IdentitiesOnly=yes",
        key_path.to_string_lossy()
    )
}
