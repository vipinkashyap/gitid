//! # gitid-core
//!
//! Core library for GitID — the multi-profile Git identity manager.
//!
//! This crate provides:
//! - **Profile management**: Create, store, and retrieve Git identity profiles
//! - **Rule-based resolution**: Automatically select profiles based on directory, URL, or host
//! - **SSH key management**: Generate, validate, and inject SSH keys per profile
//! - **Config writing**: Apply identity settings to git repos automatically
//! - **Keychain integration**: Cross-platform secure token storage
//!
//! ## Quick Example
//!
//! ```no_run
//! use gitid_core::{store, resolver, profile::Profile};
//! use std::path::Path;
//!
//! // Load profiles and rules
//! let profiles = store::load_profiles().unwrap();
//! let rules = store::load_rules().unwrap();
//!
//! // Resolve which profile to use for the current directory
//! let context = resolver::build_context(Path::new("."));
//! let result = resolver::resolve(&context, &rules, &profiles).unwrap();
//!
//! println!("Using profile: {} ({})", result.profile_name, result.reason);
//! ```

pub mod config_writer;
pub mod detect;
pub mod error;
pub mod guard;
pub mod keychain;
pub mod learn;
pub mod profile;
pub mod resolver;
pub mod ssh;
pub mod store;
pub mod team;

// Re-export commonly used types at the crate root
pub use error::{Error, Result};
