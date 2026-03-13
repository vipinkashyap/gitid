//! gitid — CLI tool for managing GitID profiles and rules.
//!
//! Commands:
//!   init                    Interactive first-time setup
//!   install                 Register as git credential helper
//!   profile list|add|edit|remove|show   Manage identity profiles
//!   rule add|list|remove    Manage matching rules
//!   status [path]           Show active profile for a directory
//!   use <profile>           Set override for current repo
//!   doctor                  Verify configuration health
//!   key generate|import|test   SSH key operations
//!   token set|test          Token management

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use console::style;
use dialoguer::{Confirm, Input};
use gitid_core::{
    config_writer, guard, keychain, learn, profile::Profile, resolver, ssh, store, team,
};
use std::path::{Path, PathBuf};
use tabled::{Table, Tabled};

#[derive(Parser)]
#[command(
    name = "gitid",
    about = "Multi-profile Git identity manager",
    version,
    long_about = "Clone any repo. GitID knows who you are.\n\n\
                  GitID manages SSH keys, git config, credential tokens, and signing keys \
                  as unified profiles that switch automatically based on your directory, \
                  remote URL, or host."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive first-time setup
    Init,

    /// Register GitID as the global git credential helper
    Install,

    /// Manage identity profiles
    #[command(subcommand)]
    Profile(ProfileCommands),

    /// Manage matching rules
    #[command(subcommand)]
    Rule(RuleCommands),

    /// Show which profile is active for a directory
    Status {
        /// Path to check (defaults to current directory)
        path: Option<PathBuf>,
    },

    /// Set a profile override for the current repo
    Use {
        /// Profile name to use
        profile: String,
    },

    /// Apply a profile to all repos in a directory
    Apply {
        /// Profile name
        profile: String,
        /// Directory containing repos
        directory: PathBuf,
    },

    /// Verify configuration health
    Doctor,

    /// SSH key operations
    #[command(subcommand)]
    Key(KeyCommands),

    /// Token management
    #[command(subcommand)]
    Token(TokenCommands),

    /// Clone a repo with the correct identity pre-applied
    Clone {
        /// Git remote URL to clone
        url: String,
        /// Optional local directory name
        directory: Option<String>,
    },

    /// Generate shell completions
    Completions {
        /// Shell type
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Print the active profile name (for shell prompts)
    #[command(name = "prompt")]
    Prompt {
        /// Output format: name, full, json
        #[arg(long, default_value = "name")]
        format: String,
    },

    /// Output shell hook script (add `eval "$(gitid shell-init)"` to .zshrc)
    #[command(name = "shell-init")]
    ShellInit {
        /// Shell type (zsh, bash, fish)
        #[arg(long, default_value = "zsh")]
        shell: String,
    },

    /// Identity guard — pre-commit hook to prevent wrong-email commits
    #[command(subcommand)]
    Guard(GuardCommands),

    /// Suggest new rules based on usage patterns
    Suggest {
        /// Minimum number of matching events to suggest a rule
        #[arg(long, default_value = "3")]
        min_evidence: usize,
        /// Apply a suggestion by index (from previous suggest output)
        #[arg(long)]
        apply: Option<usize>,
    },

    /// Check team identity constraints (.gitid.toml)
    Team {
        #[command(subcommand)]
        sub: Option<TeamCommands>,
    },
}

#[derive(Subcommand)]
enum GuardCommands {
    /// Install the identity guard (global pre-commit hook)
    Install,
    /// Uninstall the identity guard
    Uninstall,
    /// Check if the current repo's email matches the expected profile
    Check {
        /// Output as JSON (for use by the hook script)
        #[arg(long)]
        json: bool,
    },
    /// Fix a mismatch by applying the correct profile
    Fix,
    /// Show guard status
    Status,
}

#[derive(Subcommand)]
enum TeamCommands {
    /// Check the current repo against .gitid.toml constraints
    Check,
    /// Generate a sample .gitid.toml file
    Init {
        /// Team name
        #[arg(long)]
        team: String,
        /// Required email domain
        #[arg(long)]
        domain: String,
    },
}

#[derive(Subcommand)]
enum ProfileCommands {
    /// List all profiles
    List,
    /// Add a new profile (interactive)
    Add {
        /// Profile name
        name: String,
    },
    /// Show profile details
    Show {
        /// Profile name
        name: String,
    },
    /// Edit an existing profile (interactive)
    Edit {
        /// Profile name
        name: String,
    },
    /// Remove a profile
    Remove {
        /// Profile name
        name: String,
    },
}

#[derive(Subcommand)]
enum RuleCommands {
    /// Add a new rule
    Add {
        /// Rule type: dir, remote, or host
        rule_type: String,
        /// Pattern or value (e.g., "~/work/**", "*bitbucket*", "github.com")
        pattern: String,
        /// Profile to associate
        #[arg(long)]
        profile: String,
    },
    /// List all rules
    List,
    /// Remove a rule by type and index
    Remove {
        /// Rule type: dir, remote, or host
        rule_type: String,
        /// Rule index (from `rule list`)
        index: usize,
    },
}

#[derive(Subcommand)]
enum KeyCommands {
    /// Generate a new SSH key pair
    Generate {
        /// Profile name
        profile: String,
        /// Key type (ed25519, rsa)
        #[arg(long, default_value = "ed25519")]
        key_type: String,
    },
    /// Import an existing SSH key
    Import {
        /// Profile name
        profile: String,
        /// Path to the private key
        path: PathBuf,
    },
    /// Test SSH connection for a profile
    Test {
        /// Profile name
        profile: String,
    },
}

#[derive(Subcommand)]
enum TokenCommands {
    /// Store a token in the OS keychain (interactive)
    Set {
        /// Profile name
        profile: String,
        /// Host (e.g., github.com)
        host: String,
    },
    /// Test if a stored token is valid
    Test {
        /// Profile name
        profile: String,
        /// Host (e.g., github.com)
        host: String,
    },
}

// --- Table display types ---

#[derive(Tabled)]
struct ProfileRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Email")]
    email: String,
    #[tabled(rename = "SSH Key")]
    ssh_key: String,
    #[tabled(rename = "Hosts")]
    hosts: String,
}

#[derive(Tabled)]
struct RuleRow {
    #[tabled(rename = "#")]
    index: usize,
    #[tabled(rename = "Type")]
    rule_type: String,
    #[tabled(rename = "Pattern")]
    pattern: String,
    #[tabled(rename = "Profile")]
    profile: String,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init => cmd_init(),
        Commands::Install => cmd_install(),
        Commands::Profile(sub) => match sub {
            ProfileCommands::List => cmd_profile_list(),
            ProfileCommands::Add { name } => cmd_profile_add(&name),
            ProfileCommands::Show { name } => cmd_profile_show(&name),
            ProfileCommands::Edit { name } => cmd_profile_edit(&name),
            ProfileCommands::Remove { name } => cmd_profile_remove(&name),
        },
        Commands::Rule(sub) => match sub {
            RuleCommands::Add {
                rule_type,
                pattern,
                profile,
            } => cmd_rule_add(&rule_type, &pattern, &profile),
            RuleCommands::List => cmd_rule_list(),
            RuleCommands::Remove { rule_type, index } => cmd_rule_remove(&rule_type, index),
        },
        Commands::Status { path } => cmd_status(path.as_deref()),
        Commands::Use { profile } => cmd_use(&profile),
        Commands::Apply { profile, directory } => cmd_apply(&profile, &directory),
        Commands::Doctor => cmd_doctor(),
        Commands::Key(sub) => match sub {
            KeyCommands::Generate { profile, key_type } => cmd_key_generate(&profile, &key_type),
            KeyCommands::Import { profile, path } => cmd_key_import(&profile, &path),
            KeyCommands::Test { profile } => cmd_key_test(&profile),
        },
        Commands::Token(sub) => match sub {
            TokenCommands::Set { profile, host } => cmd_token_set(&profile, &host),
            TokenCommands::Test { profile, host } => cmd_token_test(&profile, &host),
        },
        Commands::Clone { url, directory } => cmd_clone(&url, directory.as_deref()),
        Commands::Completions { shell } => cmd_completions(shell),
        Commands::Prompt { format } => cmd_prompt(&format),
        Commands::ShellInit { shell } => cmd_shell_init(&shell),
        Commands::Guard(sub) => match sub {
            GuardCommands::Install => cmd_guard_install(),
            GuardCommands::Uninstall => cmd_guard_uninstall(),
            GuardCommands::Check { json } => cmd_guard_check(json),
            GuardCommands::Fix => cmd_guard_fix(),
            GuardCommands::Status => cmd_guard_status(),
        },
        Commands::Suggest { min_evidence, apply } => cmd_suggest(min_evidence, apply),
        Commands::Team { sub } => match sub {
            Some(TeamCommands::Check) => cmd_team_check(),
            Some(TeamCommands::Init { team, domain }) => cmd_team_init(&team, &domain),
            None => cmd_team_check(),
        },
    };

    if let Err(e) = result {
        eprintln!("{} {}", style("error:").red().bold(), e);
        std::process::exit(1);
    }
}

// =============================================================================
// Command implementations
// =============================================================================

fn cmd_init() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", style("Welcome to GitID!").bold().cyan());
    println!("Let's set up your first identity profile.\n");

    // Check if already initialized
    let profiles = store::load_profiles()?;
    if !profiles.profiles.is_empty() {
        println!("{}", style("GitID is already configured with profiles.").yellow());
        if !Confirm::new()
            .with_prompt("Add another profile?")
            .default(true)
            .interact()?
        {
            return Ok(());
        }
    }

    // Get profile name
    let name: String = Input::new()
        .with_prompt("Profile name")
        .default("personal".into())
        .interact_text()?;

    cmd_profile_add(&name)?;

    // Ask about rules
    println!("\n{}", style("Setting up rules...").bold());

    if Confirm::new()
        .with_prompt("Add a directory rule? (e.g., ~/work/** → this profile)")
        .default(true)
        .interact()?
    {
        let dir: String = Input::new()
            .with_prompt("Directory pattern")
            .default("~/projects/**".into())
            .interact_text()?;

        let mut rules = store::load_rules()?;
        rules.add_directory_rule(dir, &name);
        rules.set_default(&name);
        store::save_rules(&rules)?;
        println!("  {} Directory rule added", style("✓").green());
    }

    // Install credential helper
    if Confirm::new()
        .with_prompt("Register GitID as your git credential helper?")
        .default(true)
        .interact()?
    {
        cmd_install()?;
    }

    println!("\n{}", style("GitID is ready!").bold().green());
    println!("Run {} to see your current profile.", style("gitid status").cyan());

    Ok(())
}

fn cmd_install() -> Result<(), Box<dyn std::error::Error>> {
    config_writer::install_credential_helper()?;
    println!(
        "{} Registered as git credential helper (credential.helper = gitid)",
        style("✓").green(),
    );

    // Check that the binary is in PATH
    if which::which("git-credential-gitid").is_err() {
        println!("{}", style("⚠ Warning: git-credential-gitid not found in PATH").yellow());
        println!("  Make sure the binary is installed and accessible.");
    }

    Ok(())
}

fn cmd_profile_list() -> Result<(), Box<dyn std::error::Error>> {
    let profiles = store::load_profiles()?;

    if profiles.profiles.is_empty() {
        println!("No profiles configured. Run {} to get started.", style("gitid init").cyan());
        return Ok(());
    }

    let rows: Vec<ProfileRow> = profiles
        .profiles
        .iter()
        .map(|(name, p)| ProfileRow {
            name: name.clone(),
            email: p.email.clone(),
            ssh_key: p
                .ssh_key
                .as_deref()
                .unwrap_or("-")
                .to_string(),
            hosts: if p.hosts.is_empty() {
                "-".into()
            } else {
                p.hosts.join(", ")
            },
        })
        .collect();

    println!("{}", Table::new(rows));
    Ok(())
}

fn cmd_profile_add(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut profiles = store::load_profiles()?;

    if profiles.contains(name) {
        return Err(
            format!("Profile '{}' already exists. Use 'gitid profile edit {}'.", name, name)
                .into(),
        );
    }

    let git_name: String = Input::new()
        .with_prompt("Git user.name")
        .interact_text()?;

    let email: String = Input::new()
        .with_prompt("Git user.email")
        .interact_text()?;

    let ssh_key: String = Input::new()
        .with_prompt("SSH private key path (leave empty to skip)")
        .default(String::new())
        .interact_text()?;

    let username: String = Input::new()
        .with_prompt("HTTPS username (e.g., GitHub username, leave empty to skip)")
        .default(String::new())
        .interact_text()?;

    let hosts_input: String = Input::new()
        .with_prompt("Associated hosts (comma-separated, e.g., github.com,gitlab.com)")
        .default(String::new())
        .interact_text()?;

    let mut profile = Profile::new(git_name, email);

    if !ssh_key.is_empty() {
        profile = profile.with_ssh_key(ssh_key);
    }
    if !username.is_empty() {
        profile = profile.with_username(username);
    }
    if !hosts_input.is_empty() {
        let hosts: Vec<String> = hosts_input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        profile = profile.with_hosts(hosts);
    }

    profiles.set(name, profile);
    store::save_profiles(&profiles)?;

    println!("{} Profile '{}' created", style("✓").green(), style(name).bold());
    Ok(())
}

fn cmd_profile_show(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = store::load_profiles()?;
    let profile = profiles
        .get(name)
        .ok_or_else(|| format!("Profile '{}' not found", name))?;

    println!("{}", style(format!("Profile: {}", name)).bold());
    println!("  Name:        {}", profile.name);
    println!("  Email:       {}", profile.email);
    println!("  SSH Key:     {}", profile.ssh_key.as_deref().unwrap_or("(none)"));
    println!("  Signing Key: {}", profile.signing_key.as_deref().unwrap_or("(none)"));
    println!("  Username:    {}", profile.username.as_deref().unwrap_or("(none)"));
    println!(
        "  Hosts:       {}",
        if profile.hosts.is_empty() { "(none)".into() } else { profile.hosts.join(", ") }
    );

    // Validate SSH key if set
    if let Some(ref key_path) = profile.ssh_key {
        let expanded = shellexpand::tilde(key_path);
        let path = Path::new(expanded.as_ref());
        if path.exists() {
            println!("  SSH Status:  {}", style("key exists ✓").green());
            if let Ok(info) = ssh::get_key_info(path) {
                if let Some(fp) = info.fingerprint {
                    println!("  Fingerprint: {}", fp);
                }
            }
        } else {
            println!("  SSH Status:  {}", style("key NOT FOUND ✗").red());
        }
    }

    Ok(())
}

fn cmd_profile_edit(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut profiles = store::load_profiles()?;
    let existing = profiles
        .get(name)
        .ok_or_else(|| format!("Profile '{}' not found", name))?
        .clone();

    println!("{}", style(format!("Editing profile: {}", name)).bold());
    println!("Press Enter to keep the current value.\n");

    let git_name: String = Input::new()
        .with_prompt("Git user.name")
        .default(existing.name.clone())
        .interact_text()?;

    let email: String = Input::new()
        .with_prompt("Git user.email")
        .default(existing.email.clone())
        .interact_text()?;

    let ssh_key: String = Input::new()
        .with_prompt("SSH private key path")
        .default(existing.ssh_key.clone().unwrap_or_default())
        .interact_text()?;

    let username: String = Input::new()
        .with_prompt("HTTPS username")
        .default(existing.username.clone().unwrap_or_default())
        .interact_text()?;

    let hosts_input: String = Input::new()
        .with_prompt("Associated hosts (comma-separated)")
        .default(existing.hosts.join(", "))
        .interact_text()?;

    let mut profile = Profile::new(git_name, email);
    if !ssh_key.is_empty() {
        profile = profile.with_ssh_key(ssh_key);
    }
    if !username.is_empty() {
        profile = profile.with_username(username);
    }
    if !hosts_input.is_empty() {
        let hosts: Vec<String> = hosts_input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        profile = profile.with_hosts(hosts);
    }

    // Preserve signing config if not changed
    profile.signing_key = existing.signing_key;
    profile.signing_format = existing.signing_format;

    profiles.set(name, profile);
    store::save_profiles(&profiles)?;

    println!("{} Profile '{}' updated", style("✓").green(), style(name).bold());
    Ok(())
}

fn cmd_profile_remove(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut profiles = store::load_profiles()?;

    if !profiles.contains(name) {
        return Err(format!("Profile '{}' not found", name).into());
    }

    if Confirm::new()
        .with_prompt(format!("Remove profile '{}'?", name))
        .default(false)
        .interact()?
    {
        profiles.remove(name);
        store::save_profiles(&profiles)?;
        println!("{} Profile '{}' removed", style("✓").green(), name);
    } else {
        println!("Cancelled.");
    }

    Ok(())
}

fn cmd_rule_add(
    rule_type: &str,
    pattern: &str,
    profile_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = store::load_profiles()?;
    if !profiles.contains(profile_name) {
        return Err(format!("Profile '{}' not found", profile_name).into());
    }

    let mut rules = store::load_rules()?;

    match rule_type {
        "dir" | "directory" => {
            rules.add_directory_rule(pattern, profile_name);
            println!(
                "{} Directory rule added: {} → {}",
                style("✓").green(),
                style(pattern).cyan(),
                style(profile_name).bold(),
            );
        }
        "remote" | "url" => {
            rules.add_remote_rule(pattern, profile_name);
            println!(
                "{} Remote URL rule added: {} → {}",
                style("✓").green(),
                style(pattern).cyan(),
                style(profile_name).bold(),
            );
        }
        "host" => {
            rules.add_host_rule(pattern, profile_name);
            println!(
                "{} Host rule added: {} → {}",
                style("✓").green(),
                style(pattern).cyan(),
                style(profile_name).bold(),
            );
        }
        _ => {
            return Err(format!(
                "Unknown rule type '{}'. Use: dir, remote, or host",
                rule_type
            )
            .into());
        }
    }

    store::save_rules(&rules)?;
    Ok(())
}

fn cmd_rule_list() -> Result<(), Box<dyn std::error::Error>> {
    let rules = store::load_rules()?;

    if rules.total_rules() == 0 && rules.default.is_none() {
        println!("No rules configured. Run {} to add rules.", style("gitid rule add").cyan());
        return Ok(());
    }

    let mut rows: Vec<RuleRow> = Vec::new();
    let mut idx = 0;

    for rule in &rules.directory {
        rows.push(RuleRow {
            index: idx,
            rule_type: "directory".into(),
            pattern: rule.path.clone(),
            profile: rule.profile.clone(),
        });
        idx += 1;
    }

    for rule in &rules.remote {
        rows.push(RuleRow {
            index: idx,
            rule_type: "remote".into(),
            pattern: rule.pattern.clone(),
            profile: rule.profile.clone(),
        });
        idx += 1;
    }

    for rule in &rules.host {
        rows.push(RuleRow {
            index: idx,
            rule_type: "host".into(),
            pattern: rule.host.clone(),
            profile: rule.profile.clone(),
        });
        idx += 1;
    }

    println!("{}", style("Rules (highest priority first):").bold());
    println!("{}", Table::new(rows));

    if let Some(ref default) = rules.default {
        println!("\nGlobal default: {}", style(default).bold());
    }

    Ok(())
}

fn cmd_rule_remove(rule_type: &str, index: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut rules = store::load_rules()?;

    let removed = match rule_type {
        "dir" | "directory" => rules
            .remove_directory_rule(index)
            .map(|r| format!("directory: {} → {}", r.path, r.profile)),
        "remote" | "url" => rules
            .remove_remote_rule(index)
            .map(|r| format!("remote: {} → {}", r.pattern, r.profile)),
        "host" => rules
            .remove_host_rule(index)
            .map(|r| format!("host: {} → {}", r.host, r.profile)),
        _ => {
            return Err(
                format!(
                    "Unknown rule type '{}'. Use: dir, remote, or host",
                    rule_type,
                )
                .into(),
            )
        }
    };

    match removed {
        Some(desc) => {
            store::save_rules(&rules)?;
            println!("{} Removed rule: {}", style("✓").green(), desc);
        }
        None => {
            return Err(format!("No {} rule at index {}", rule_type, index).into());
        }
    }

    Ok(())
}

fn cmd_status(path: Option<&Path>) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = match path {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir()?,
    };

    let profiles = store::load_profiles()?;
    let rules = store::load_rules()?;
    let context = resolver::build_context(&cwd);

    match resolver::resolve(&context, &rules, &profiles) {
        Ok(result) => {
            let profile = profiles.get(&result.profile_name).unwrap();

            println!("{}", style("GitID Status").bold());
            println!("  Directory:   {}", cwd.display());
            println!("  Profile:     {}", style(&result.profile_name).bold().green());
            println!("  Matched by:  {}", result.reason);
            println!("  Name:        {}", profile.name);
            println!("  Email:       {}", profile.email);
            if let Some(ref key) = profile.ssh_key {
                println!("  SSH Key:     {}", key);
            }
            if let Some(ref remote) = context.remote_url {
                println!("  Remote:      {}", remote);
            }
        }
        Err(_) => {
            println!("{}", style("No profile resolved for this directory.").yellow());
            println!("  Directory: {}", cwd.display());
            println!("\nRun {} to set up rules.", style("gitid init").cyan());
        }
    }

    Ok(())
}

fn cmd_use(profile_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = store::load_profiles()?;
    if !profiles.contains(profile_name) {
        return Err(format!("Profile '{}' not found", profile_name).into());
    }

    let cwd = std::env::current_dir()?;
    if !config_writer::is_git_repo(&cwd) {
        return Err("Not inside a git repository".into());
    }

    config_writer::set_repo_profile_override(&cwd, profile_name)?;

    let profile = profiles.get(profile_name).unwrap();
    config_writer::apply_profile_to_repo(profile, &cwd)?;

    println!(
        "{} Using profile '{}' for this repo",
        style("✓").green(),
        style(profile_name).bold(),
    );
    println!("  Name:  {}", profile.name);
    println!("  Email: {}", profile.email);

    Ok(())
}

fn cmd_apply(profile_name: &str, directory: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = store::load_profiles()?;
    let profile = profiles
        .get(profile_name)
        .ok_or_else(|| format!("Profile '{}' not found", profile_name))?;

    if !directory.exists() {
        return Err(format!("Directory not found: {}", directory.display()).into());
    }

    let mut applied = 0;
    let mut errors = 0;

    // Find all git repos in the directory
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.join(".git").exists() {
            match config_writer::apply_profile_to_repo(profile, &path) {
                Ok(()) => {
                    println!(
                        "  {} {}",
                        style("✓").green(),
                        path.file_name().unwrap().to_string_lossy()
                    );
                    applied += 1;
                }
                Err(e) => {
                    eprintln!(
                        "  {} {} — {}",
                        style("✗").red(),
                        path.file_name().unwrap().to_string_lossy(),
                        e
                    );
                    errors += 1;
                }
            }
        }
    }

    println!(
        "\nApplied '{}' to {} repo(s){}",
        profile_name,
        applied,
        if errors > 0 {
            format!(" ({} errors)", errors)
        } else {
            String::new()
        }
    );

    Ok(())
}

fn cmd_doctor() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", style("GitID Doctor").bold());
    println!("{}\n", style("Checking configuration health...").dim());

    let mut issues = 0;

    // 1. Check credential helper
    print!("  Credential helper... ");
    if config_writer::is_credential_helper_installed() {
        println!("{}", style("✓ registered").green());
    } else {
        println!("{}", style("✗ not registered").red());
        println!("    Run: gitid install");
        issues += 1;
    }

    // 2. Check binary in PATH
    print!("  git-credential-gitid in PATH... ");
    if which::which("git-credential-gitid").is_ok() {
        println!("{}", style("✓ found").green());
    } else {
        println!("{}", style("✗ not found").red());
        println!("    Install with: cargo install git-credential-gitid");
        issues += 1;
    }

    // 3. Check profiles
    let profiles = store::load_profiles()?;
    print!("  Profiles... ");
    if profiles.profiles.is_empty() {
        println!("{}", style("✗ none configured").red());
        println!("    Run: gitid init");
        issues += 1;
    } else {
        println!("{} ({} profile(s))", style("✓").green(), profiles.profiles.len());
    }

    // 4. Check SSH keys for each profile
    for (name, profile) in &profiles.profiles {
        if let Some(ref key_path) = profile.ssh_key {
            print!("  SSH key [{}]... ", name);
            match ssh::validate_key(Path::new(&shellexpand::tilde(key_path).to_string())) {
                Ok(()) => println!("{}", style("✓ valid").green()),
                Err(e) => {
                    println!("{}", style(format!("✗ {}", e)).red());
                    issues += 1;
                }
            }
        }
    }

    // 5. Check rules
    let rules = store::load_rules()?;
    print!("  Rules... ");
    if rules.total_rules() == 0 && rules.default.is_none() {
        println!("{}", style("⚠ no rules configured").yellow());
        println!("    Run: gitid rule add dir ~/work/** --profile <name>");
        issues += 1;
    } else {
        println!(
            "{} ({} rule(s), default: {})",
            style("✓").green(),
            rules.total_rules(),
            rules.default.as_deref().unwrap_or("none")
        );
    }

    // 6. Check token availability for each profile's hosts
    for (name, profile) in &profiles.profiles {
        for host in &profile.hosts {
            print!("  Token [{}@{}]... ", name, host);
            if keychain::has_token(name, host) {
                println!("{}", style("✓ stored").green());
            } else {
                println!("{}", style("⚠ not stored").yellow());
                println!("    Run: gitid token set {} {}", name, host);
            }
        }
    }

    // Summary
    println!();
    if issues == 0 {
        println!("{}", style("All checks passed! GitID is healthy.").bold().green());
    } else {
        println!(
            "{}",
            style(format!("{} issue(s) found. See above for fixes.", issues))
                .bold()
                .yellow()
        );
    }

    Ok(())
}

fn cmd_key_generate(profile_name: &str, key_type: &str) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = store::load_profiles()?;
    let profile = profiles
        .get(profile_name)
        .ok_or_else(|| format!("Profile '{}' not found", profile_name))?;

    let default_path = format!("~/.ssh/id_{}_{}", key_type, profile_name);
    let key_path: String = Input::new()
        .with_prompt("Key file path")
        .default(default_path)
        .interact_text()?;

    let expanded = shellexpand::tilde(&key_path);
    let path = PathBuf::from(expanded.as_ref());

    let info = ssh::generate_key(&profile.email, &path, key_type)?;

    println!("{} SSH key generated", style("✓").green());
    println!("  Private: {}", info.private_key.display());
    println!("  Public:  {}", info.public_key.display());
    if let Some(fp) = info.fingerprint {
        println!("  Fingerprint: {}", fp);
    }

    // Ask if we should update the profile
    if Confirm::new()
        .with_prompt("Update profile SSH key path?")
        .default(true)
        .interact()?
    {
        let mut profiles = store::load_profiles()?;
        if let Some(p) = profiles.profiles.get_mut(profile_name) {
            p.ssh_key = Some(key_path);
            store::save_profiles(&profiles)?;
            println!("{} Profile updated", style("✓").green());
        }
    }

    println!(
        "\n{} Add this public key to your Git hosting provider:",
        style("Next step:").bold()
    );
    if let Ok(pub_content) = std::fs::read_to_string(&info.public_key) {
        println!("\n{}", pub_content.trim());
    }

    Ok(())
}

fn cmd_key_import(profile_name: &str, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        return Err(format!("Key file not found: {}", path.display()).into());
    }

    let mut profiles = store::load_profiles()?;
    if !profiles.contains(profile_name) {
        return Err(format!("Profile '{}' not found", profile_name).into());
    }

    let info = ssh::get_key_info(path)?;

    if let Some(p) = profiles.profiles.get_mut(profile_name) {
        p.ssh_key = Some(path.to_string_lossy().to_string());
        store::save_profiles(&profiles)?;
    }

    println!("{} SSH key imported for profile '{}'", style("✓").green(), profile_name);
    if let Some(fp) = info.fingerprint {
        println!("  Fingerprint: {}", fp);
    }

    Ok(())
}

fn cmd_key_test(profile_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = store::load_profiles()?;
    let profile = profiles
        .get(profile_name)
        .ok_or_else(|| format!("Profile '{}' not found", profile_name))?;

    let key_path = profile
        .ssh_key
        .as_ref()
        .ok_or("No SSH key configured for this profile")?;

    let expanded = shellexpand::tilde(key_path);
    let path = Path::new(expanded.as_ref());

    for host in &profile.hosts {
        print!("  Testing {} ... ", host);
        match ssh::test_connection(host, path) {
            Ok(true) => println!("{}", style("✓ authenticated").green()),
            Ok(false) => println!("{}", style("✗ failed").red()),
            Err(e) => println!("{}", style(format!("✗ error: {}", e)).red()),
        }
    }

    if profile.hosts.is_empty() {
        println!(
            "{}",
            style("No hosts configured for this profile. Add hosts with `gitid profile edit`.")
                .yellow()
        );
    }

    Ok(())
}

fn cmd_token_set(profile_name: &str, host: &str) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = store::load_profiles()?;
    if !profiles.contains(profile_name) {
        return Err(format!("Profile '{}' not found", profile_name).into());
    }

    let token: String = dialoguer::Password::new()
        .with_prompt(format!("Token for {}@{}", profile_name, host))
        .interact()?;

    keychain::store_token(profile_name, host, &token)?;

    println!(
        "{} Token stored in OS keychain for {}@{}",
        style("✓").green(),
        profile_name,
        host
    );

    Ok(())
}

fn cmd_token_test(profile_name: &str, host: &str) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = store::load_profiles()?;
    if !profiles.contains(profile_name) {
        return Err(format!("Profile '{}' not found", profile_name).into());
    }

    match keychain::get_token(profile_name, host)? {
        Some(token) => {
            print!("  Testing token for {}@{} ... ", profile_name, host);
            match keychain::test_token(host, &token) {
                Ok(true) => println!("{}", style("✓ valid").green()),
                Ok(false) => println!("{}", style("✗ invalid or host not supported").red()),
                Err(e) => println!("{}", style(format!("✗ error: {}", e)).red()),
            }
        }
        None => {
            println!(
                "{} No token stored for {}@{}",
                style("✗").red(),
                profile_name,
                host
            );
            println!("  Run: gitid token set {} {}", profile_name, host);
        }
    }

    Ok(())
}

// =============================================================================
// Clone with identity
// =============================================================================

fn cmd_clone(url: &str, directory: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = store::load_profiles()?;
    let rules = store::load_rules()?;

    // Extract host from the clone URL to resolve a profile
    let host = resolver::extract_host_from_url(url);
    let context = resolver::ResolveContext {
        cwd: Some(std::env::current_dir()?),
        host: host.clone(),
        remote_url: Some(url.to_string()),
    };

    let resolved = resolver::resolve(&context, &rules, &profiles).ok();

    if let Some(ref result) = resolved {
        println!(
            "{} Will use profile: {} ({})",
            style("→").cyan(),
            style(&result.profile_name).bold(),
            result.reason
        );
    }

    // Build the git clone command
    let mut cmd = std::process::Command::new("git");
    cmd.arg("clone");

    // If we have a resolved profile with an SSH key, inject it
    if let Some(ref result) = resolved {
        if let Some(profile) = profiles.get(&result.profile_name) {
            if let Some(ref ssh_key) = profile.ssh_key {
                let expanded = shellexpand::tilde(ssh_key);
                let ssh_cmd = format!("ssh -i {} -o IdentitiesOnly=yes", expanded);
                cmd.env("GIT_SSH_COMMAND", &ssh_cmd);
            }
        }
    }

    cmd.arg(url);
    if let Some(dir) = directory {
        cmd.arg(dir);
    }

    println!("{} Cloning {}...\n", style("→").cyan(), style(url).underlined());

    let status = cmd.status().map_err(|e| {
        format!("Failed to run git clone: {}", e)
    })?;

    if !status.success() {
        return Err("git clone failed".into());
    }

    // Determine the cloned directory
    let clone_dir = if let Some(dir) = directory {
        PathBuf::from(dir)
    } else {
        // Infer from URL: last path segment minus .git
        let name = url
            .rsplit('/')
            .next()
            .unwrap_or("repo")
            .trim_end_matches(".git");
        PathBuf::from(name)
    };

    // Apply the resolved profile to the freshly cloned repo
    if let Some(ref result) = resolved {
        if let Some(profile) = profiles.get(&result.profile_name) {
            config_writer::apply_profile_to_repo(profile, &clone_dir)?;
            config_writer::set_repo_profile_override(&clone_dir, &result.profile_name)?;

            println!(
                "\n{} Identity applied to {}:",
                style("✓").green(),
                clone_dir.display()
            );
            println!("  Name:  {}", profile.name);
            println!("  Email: {}", profile.email);
            if let Some(ref key) = profile.ssh_key {
                println!("  SSH:   {}", key);
            }
        }
    }

    Ok(())
}

// =============================================================================
// Shell completions
// =============================================================================

fn cmd_completions(shell: Shell) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "gitid", &mut std::io::stdout());
    Ok(())
}

// =============================================================================
// Prompt (for shell PS1 integration)
// =============================================================================

fn cmd_prompt(format: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    let profiles = store::load_profiles()?;
    let rules = store::load_rules()?;
    let context = resolver::build_context(&cwd);

    match resolver::resolve(&context, &rules, &profiles) {
        Ok(result) => match format {
            "name" => print!("{}", result.profile_name),
            "full" => {
                let profile = profiles.get(&result.profile_name);
                let email = profile.map(|p| p.email.as_str()).unwrap_or("");
                print!("{}:{}", result.profile_name, email);
            }
            "json" => {
                let profile = profiles.get(&result.profile_name);
                let json = serde_json::json!({
                    "profile": result.profile_name,
                    "reason": result.reason.to_string(),
                    "name": profile.map(|p| p.name.as_str()),
                    "email": profile.map(|p| p.email.as_str()),
                });
                print!("{}", json);
            }
            _ => print!("{}", result.profile_name),
        },
        Err(_) => {
            // No profile resolved — output nothing (keeps prompt clean)
        }
    }

    Ok(())
}

// =============================================================================
// Shell init — outputs the hook script
// =============================================================================

fn cmd_shell_init(shell: &str) -> Result<(), Box<dyn std::error::Error>> {
    match shell {
        "zsh" => print!("{}", ZSH_INIT_SCRIPT),
        "bash" => print!("{}", BASH_INIT_SCRIPT),
        "fish" => print!("{}", FISH_INIT_SCRIPT),
        _ => return Err(format!("Unsupported shell: {}. Use zsh, bash, or fish.", shell).into()),
    }
    Ok(())
}

// =============================================================================
// Guard commands
// =============================================================================

fn cmd_guard_install() -> Result<(), Box<dyn std::error::Error>> {
    guard::install()?;
    println!("{} Identity guard installed (global pre-commit hook)", style("✓").green());
    println!("  Every commit will now be checked against your GitID profiles.");
    println!("  To skip temporarily: GITID_GUARD_SKIP=1 git commit ...");
    Ok(())
}

fn cmd_guard_uninstall() -> Result<(), Box<dyn std::error::Error>> {
    guard::uninstall()?;
    println!("{} Identity guard uninstalled", style("✓").green());
    Ok(())
}

fn cmd_guard_check(json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    let verdict = guard::check(&cwd);

    if json {
        let json_val = match &verdict {
            guard::GuardVerdict::Ok { profile, email } => {
                serde_json::json!({
                    "verdict": "ok",
                    "profile": profile,
                    "email": email,
                })
            }
            guard::GuardVerdict::Mismatch {
                profile,
                expected_email,
                actual_email,
            } => {
                serde_json::json!({
                    "verdict": "mismatch",
                    "profile": profile,
                    "expected_email": expected_email,
                    "actual_email": actual_email,
                })
            }
            guard::GuardVerdict::NoProfile => {
                serde_json::json!({ "verdict": "no_profile" })
            }
            guard::GuardVerdict::NotARepo => {
                serde_json::json!({ "verdict": "not_a_repo" })
            }
        };
        println!("{}", json_val);
    } else {
        match &verdict {
            guard::GuardVerdict::Ok { profile, email } => {
                println!(
                    "{} Email matches profile '{}' ({})",
                    style("✓").green(),
                    style(profile).bold(),
                    email
                );
            }
            guard::GuardVerdict::Mismatch {
                profile,
                expected_email,
                actual_email,
            } => {
                println!("{} Email mismatch!", style("✗").red());
                println!("  Expected: {} (profile: {})", expected_email, profile);
                println!("  Actual:   {}", actual_email);
                println!("\n  Fix with: {}", style("gitid guard fix").cyan());
            }
            guard::GuardVerdict::NoProfile => {
                println!(
                    "{} No GitID profile resolved for this directory",
                    style("⚠").yellow()
                );
            }
            guard::GuardVerdict::NotARepo => {
                println!("{} Not inside a git repository", style("⚠").yellow());
            }
        }
    }

    Ok(())
}

fn cmd_guard_fix() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    guard::fix_mismatch(&cwd)?;
    println!("{} Applied the correct profile identity to this repo", style("✓").green());
    Ok(())
}

fn cmd_guard_status() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", style("Identity Guard Status").bold());
    if guard::is_installed() {
        println!("  Hook:    {}", style("✓ installed (global pre-commit)").green());
    } else {
        println!("  Hook:    {}", style("✗ not installed").red());
        println!("  Install: {}", style("gitid guard install").cyan());
    }

    // Also show check result for current dir
    let cwd = std::env::current_dir()?;
    let verdict = guard::check(&cwd);
    match &verdict {
        guard::GuardVerdict::Ok { profile, email } => {
            println!(
                "  Current: {} {} ({})",
                style("✓").green(),
                profile,
                email
            );
        }
        guard::GuardVerdict::Mismatch {
            expected_email,
            actual_email,
            ..
        } => {
            println!(
                "  Current: {} expected {} but got {}",
                style("✗").red(),
                expected_email,
                actual_email
            );
        }
        guard::GuardVerdict::NoProfile => {
            println!("  Current: no profile resolved");
        }
        guard::GuardVerdict::NotARepo => {
            println!("  Current: not in a git repo");
        }
    }

    Ok(())
}

// =============================================================================
// Suggest command (pattern learning)
// =============================================================================

fn cmd_suggest(
    min_evidence: usize,
    apply_index: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_count = learn::event_count()?;

    if event_count == 0 {
        println!(
            "{} No activity logged yet. Use GitID for a while, then run this again.",
            style("⚠").yellow()
        );
        println!(
            "  Activity is logged each time a profile is resolved ({} events so far).",
            event_count
        );
        return Ok(());
    }

    let suggestions = learn::suggest(min_evidence)?;

    if suggestions.is_empty() {
        println!(
            "{} No new rules to suggest (based on {} events, min evidence: {}).",
            style("✓").green(),
            event_count,
            min_evidence
        );
        println!("  Your rules already cover your usage patterns well!");
        return Ok(());
    }

    // If --apply was given, apply a specific suggestion
    if let Some(idx) = apply_index {
        if idx >= suggestions.len() {
            return Err(format!(
                "Suggestion index {} out of range (0..{})",
                idx,
                suggestions.len() - 1
            )
            .into());
        }

        let s = &suggestions[idx];
        let mut rules = store::load_rules()?;

        match s.rule_type {
            learn::SuggestionType::Directory => {
                rules.add_directory_rule(&s.pattern, &s.profile);
            }
            learn::SuggestionType::Remote => {
                rules.add_remote_rule(&s.pattern, &s.profile);
            }
            learn::SuggestionType::Host => {
                rules.add_host_rule(&s.pattern, &s.profile);
            }
            learn::SuggestionType::Default => {
                rules.set_default(&s.profile);
            }
        }

        store::save_rules(&rules)?;
        println!(
            "{} Applied: {} rule {} → {}",
            style("✓").green(),
            s.rule_type,
            style(&s.pattern).cyan(),
            style(&s.profile).bold()
        );
        return Ok(());
    }

    // Display suggestions
    println!(
        "{} Suggestions based on {} logged events:\n",
        style("💡").bold(),
        event_count
    );

    for (i, s) in suggestions.iter().enumerate() {
        println!(
            "  {} {} {} → {}",
            style(format!("[{}]", i)).dim(),
            style(format!("{:>9}", s.rule_type.to_string())).cyan(),
            style(&s.pattern).bold(),
            style(&s.profile).green()
        );
        println!(
            "      {} (evidence: {} events)",
            style(&s.reason).dim(),
            s.evidence_count
        );
    }

    println!(
        "\n  Apply a suggestion with: {}",
        style("gitid suggest --apply <index>").cyan()
    );

    Ok(())
}

// =============================================================================
// Team config commands
// =============================================================================

fn cmd_team_check() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    let team_config = team::TeamConfig::load(&cwd);

    match team_config {
        None => {
            println!(
                "{} No .gitid.toml found in this repository.",
                style("⚠").yellow()
            );
            println!(
                "  Create one with: {}",
                style("gitid team init --team <name> --domain <domain>").cyan()
            );
            return Ok(());
        }
        Some(config) => {
            println!("{}", style("Team Identity Check").bold());
            if let Some(ref name) = config.team {
                println!("  Team: {}", style(name).bold());
            }

            if !config.has_constraints() {
                println!(
                    "  {}",
                    style("No identity constraints defined in .gitid.toml").dim()
                );
                return Ok(());
            }

            // Get the current identity
            let profiles = store::load_profiles()?;
            let rules = store::load_rules()?;
            let context = resolver::build_context(&cwd);

            let (email, _ssh_fp) = match resolver::resolve(&context, &rules, &profiles) {
                Ok(result) => {
                    let profile = profiles.get(&result.profile_name);
                    let email = profile
                        .map(|p| p.email.clone())
                        .unwrap_or_default();
                    // TODO: get SSH fingerprint if key is set
                    (email, None::<String>)
                }
                Err(_) => {
                    // Try reading from git config directly
                    let output = std::process::Command::new("git")
                        .args(["config", "user.email"])
                        .current_dir(&cwd)
                        .output();
                    let email = output
                        .ok()
                        .filter(|o| o.status.success())
                        .and_then(|o| String::from_utf8(o.stdout).ok())
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default();
                    (email, None)
                }
            };

            if email.is_empty() {
                println!(
                    "  {} Could not determine current email — configure a profile first.",
                    style("✗").red()
                );
                return Ok(());
            }

            println!("  Email: {}\n", style(&email).bold());

            let validation = config.validate(&email, _ssh_fp.as_deref());

            for check in &validation.checks {
                let icon = if check.passed {
                    style("✓").green()
                } else {
                    style("✗").red()
                };
                println!("  {} {}: {}", icon, check.name, check.message);
            }

            println!();
            if validation.passed {
                println!(
                    "  {}",
                    style("All team identity checks passed!").bold().green()
                );
            } else {
                println!(
                    "  {}",
                    style("Some team identity checks failed.").bold().red()
                );
                println!(
                    "  Switch to the correct profile with: {}",
                    style("gitid use <profile>").cyan()
                );
            }
        }
    }

    Ok(())
}

fn cmd_team_init(team_name: &str, domain: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    let target = cwd.join(".gitid.toml");

    if target.exists() {
        return Err(format!(".gitid.toml already exists at {}", target.display()).into());
    }

    let content = team::TeamConfig::sample(team_name, domain);
    std::fs::write(&target, &content)?;

    println!(
        "{} Created .gitid.toml for team '{}'",
        style("✓").green(),
        style(team_name).bold()
    );
    println!("  File: {}", target.display());
    println!(
        "  Commit this file to enforce identity constraints for all team members."
    );

    Ok(())
}

const ZSH_INIT_SCRIPT: &str = r#"
# GitID shell integration for zsh
# Add to .zshrc: eval "$(gitid shell-init --shell zsh)"

# --- Prompt integration ---
# Shows the active GitID profile in your prompt.
# Customize _gitid_prompt_info to change the format.
_gitid_prompt_info() {
  local profile
  profile=$(gitid prompt 2>/dev/null)
  if [[ -n "$profile" ]]; then
    echo " %F{cyan}[${profile}]%f"
  fi
}

# Append to RPROMPT (right prompt) — or customize as you like
if [[ -z "$GITID_NO_PROMPT" ]]; then
  RPROMPT='$(_gitid_prompt_info)'"${RPROMPT}"
fi

# --- Auto-apply on cd ---
# When you cd into a git repo, GitID shows which profile is active
# and applies the identity to the repo config.
_gitid_chpwd_hook() {
  if [[ -d ".git" ]] || git rev-parse --git-dir &>/dev/null 2>&1; then
    local profile
    profile=$(gitid prompt 2>/dev/null)
    if [[ -n "$profile" ]]; then
      # Silently apply the profile (sets user.name, user.email, core.sshCommand)
      gitid use "$profile" &>/dev/null 2>&1
      # Show a subtle notification
      if [[ -z "$GITID_QUIET" ]]; then
        echo "\033[0;36m→ gitid: ${profile}\033[0m"
      fi
    fi
  fi
}

# Register the hook
autoload -Uz add-zsh-hook
add-zsh-hook chpwd _gitid_chpwd_hook

# --- Completions ---
# Enable zsh completions for gitid
if (( $+commands[gitid] )); then
  eval "$(gitid completions zsh)"
fi

# --- Convenience aliases ---
alias gc='gitid clone'
alias gs='gitid status'
alias gp='gitid profile list'
"#;

const BASH_INIT_SCRIPT: &str = r#"
# GitID shell integration for bash
# Add to .bashrc: eval "$(gitid shell-init --shell bash)"

_gitid_prompt_info() {
  local profile
  profile=$(gitid prompt 2>/dev/null)
  if [[ -n "$profile" ]]; then
    echo " [\033[0;36m${profile}\033[0m]"
  fi
}

if [[ -z "$GITID_NO_PROMPT" ]]; then
  PROMPT_COMMAND="_gitid_prompt_command;${PROMPT_COMMAND}"
  _gitid_prompt_command() {
    PS1="${PS1%\$ }$(_gitid_prompt_info)\$ "
  }
fi

_gitid_cd_hook() {
  builtin cd "$@" || return
  if [[ -d ".git" ]] || git rev-parse --git-dir &>/dev/null 2>&1; then
    local profile
    profile=$(gitid prompt 2>/dev/null)
    if [[ -n "$profile" ]]; then
      gitid use "$profile" &>/dev/null 2>&1
      if [[ -z "$GITID_QUIET" ]]; then
        echo -e "\033[0;36m→ gitid: ${profile}\033[0m"
      fi
    fi
  fi
}
alias cd='_gitid_cd_hook'

if command -v gitid &>/dev/null; then
  eval "$(gitid completions bash)"
fi

alias gc='gitid clone'
alias gs='gitid status'
alias gp='gitid profile list'
"#;

const FISH_INIT_SCRIPT: &str = r#"
# GitID shell integration for fish
# Add to config.fish: gitid shell-init --shell fish | source

function _gitid_prompt_info
  set -l profile (gitid prompt 2>/dev/null)
  if test -n "$profile"
    set_color cyan
    echo -n " [$profile]"
    set_color normal
  end
end

function _gitid_on_cd --on-variable PWD
  if test -d ".git"; or git rev-parse --git-dir &>/dev/null
    set -l profile (gitid prompt 2>/dev/null)
    if test -n "$profile"
      gitid use "$profile" &>/dev/null
      if not set -q GITID_QUIET
        set_color cyan
        echo "→ gitid: $profile"
        set_color normal
      end
    end
  end
end

if command -v gitid &>/dev/null
  gitid completions fish | source
end

alias gc 'gitid clone'
alias gs 'gitid status'
alias gp 'gitid profile list'
"#;
