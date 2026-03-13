//! Error types for gitid-core.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for gitid operations.
pub type Result<T> = std::result::Result<T, Error>;

/// All possible errors in gitid-core.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not determine config directory (HOME not set?)")]
    NoConfigDir,

    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse YAML at {path}: {source}")]
    YamlParse {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },

    #[error("Failed to serialize YAML: {0}")]
    YamlSerialize(#[source] serde_yaml::Error),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("No profile could be resolved for the current context")]
    NoProfileResolved,

    #[error("Profile already exists: {0}")]
    ProfileAlreadyExists(String),

    #[error("SSH key not found: {0}")]
    SshKeyNotFound(PathBuf),

    #[error("SSH key already exists: {0}")]
    SshKeyExists(PathBuf),

    #[error("SSH key generation failed: {0}")]
    SshKeyGenFailed(String),

    #[error("SSH key has incorrect permissions at {path} (mode {mode:o}, expected 600)")]
    SshKeyPermissions { path: PathBuf, mode: u32 },

    #[error("Keychain error ({operation}): {detail}")]
    Keychain { operation: String, detail: String },

    #[error("Git config error for key '{key}': {stderr}")]
    GitConfigFailed { key: String, stderr: String },

    #[error("Command '{command}' failed: {source}")]
    CommandFailed {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Credential helper protocol error: {0}")]
    CredentialProtocol(String),
}
