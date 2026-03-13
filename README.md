# GitID

**Multi-profile Git identity manager.** Automatically switches your name, email, SSH key, and credentials based on which repo you're in.

If you juggle work, personal, and open-source Git accounts on the same machine, GitID makes sure you never commit with the wrong identity again.

---

## Why GitID?

Every developer with multiple Git accounts has committed with the wrong email. Git's `includeIf` directive helps, but it only covers directory-based switching, requires manual config editing, and doesn't handle SSH keys or credentials. GitID replaces all of that with a single tool.

**What makes it different from the 20 other "git profile switcher" tools:**

- **Auto-detection** — scans your SSH keys, gitconfig, and repos to suggest profiles on first run
- **5-tier rule engine** — matches by directory path, remote URL, hostname, repo override, or default
- **Identity Guard** — pre-commit hook that blocks commits when the identity doesn't match
- **Pattern learning** — watches your activity and suggests new rules automatically
- **Team constraints** — commit a `.gitid.toml` to enforce identity rules across your org
- **Credential helper** — plugs into Git's credential system to serve the right token per profile
- **Desktop app + CLI** — Tauri desktop GUI for setup, CLI for daily use and shell integration

## Installation

### CLI (recommended)

```bash
# Install script (macOS / Linux)
curl -fsSL https://gitid.dev/install | sh

# Or build from source
cargo install --path crates/gitid-cli
cargo install --path crates/git-credential-gitid
```

### Desktop App

Download from [Releases](https://github.com/vipinkashyap/gitid/releases) — available for macOS (.dmg) and Linux (.AppImage).

Or build from source:

```bash
cd tauri-app
npm install
cargo tauri build
```

## Quick Start

```bash
# 1. Auto-detect your existing identities
gitid init

# 2. Check what profiles were created
gitid profile list

# 3. Add a rule to map your work directory
gitid rule add directory "~/work/**" --profile work

# 4. Enable the identity guard
gitid guard install

# 5. Add shell integration (add to ~/.zshrc or ~/.bashrc)
eval "$(gitid shell-hook)"
```

That's it. When you `cd` into a repo, GitID resolves the right profile and applies it. The guard catches any mismatches at commit time.

## How It Works

GitID resolves profiles using a 5-tier priority system:

| Priority | Rule Type | Example |
|----------|-----------|---------|
| 1 | Repo override | `gitid use work` in a specific repo |
| 2 | Directory rule | `~/work/**` → work profile |
| 3 | Remote rule | `github.com/mycompany/*` → work profile |
| 4 | Host rule | `gitlab.internal.com` → work profile |
| 5 | Default | Fallback when nothing else matches |

Rules are checked top-to-bottom within each type. First match wins.

## CLI Reference

```
gitid init                          Interactive first-time setup
gitid status [path]                 Show active profile for a directory
gitid doctor                        Verify configuration health

gitid profile list                  List all profiles
gitid profile add <name>            Create a new profile interactively
gitid profile show <name>           Display profile details
gitid profile edit <name>           Edit a profile
gitid profile remove <name>         Delete a profile

gitid rule add <type> <pattern>     Add a rule (directory, remote, host)
gitid rule list                     Show all rules with priorities
gitid rule remove <type> <index>    Remove a rule

gitid guard install                 Install pre-commit identity guard
gitid guard uninstall               Remove the guard
gitid guard check                   Check identity in current repo
gitid guard fix                     Auto-fix identity mismatch

gitid key generate <profile>        Generate a new SSH key pair
gitid key import <profile> <path>   Import an existing SSH key
gitid key test <profile>            Test SSH connectivity

gitid token set <profile> <host>    Store a token in OS keychain
gitid token test <profile> <host>   Validate a stored token

gitid suggest                       Show rule suggestions from activity
gitid use <profile>                 Set repo-level profile override
gitid clone <url> [dir]             Clone with identity pre-applied
gitid shell-init                    Output shell hook script
gitid prompt                        Print active profile for shell PS1
gitid completions <shell>           Generate shell completions
gitid team check                    Validate against team constraints
```

## Desktop App

The Tauri desktop app provides a visual interface for managing profiles, rules, and monitoring identity status.

**Tabs:**
- **Dashboard** — Identity Guard status, active profile, smart suggestions, repo scanner
- **Profiles** — Create, edit, delete profiles with SSH key testing
- **Rules** — Visual rule editor with priority ordering
- **Doctor** — System health checks with suggested fixes
- **Help** — In-app guides for CLI, IDE integration, and troubleshooting

**Features:** Dark mode, keyboard accessible (WCAG), first-run setup wizard with auto-detection.

## IDE Integration

GitID works with any editor or Git GUI:

- **Identity Guard** fires on every commit regardless of the tool (VS Code, JetBrains, Sublime Merge, etc.)
- **Shell hook** works in any embedded terminal that loads your shell profile
- Run `gitid resolve` once per project to write the identity into the repo's local Git config

See the in-app Help tab for detailed setup instructions per editor.

## Team Constraints

Add a `.gitid.toml` to your repo to enforce identity rules:

```toml
[constraints]
required_domain = "company.com"
require_signing = true

[[profile_hints]]
name = "Company Profile"
email_pattern = "*@company.com"
```

Team members running `gitid team check` will be warned if their identity doesn't meet the constraints.

## Configuration

GitID stores config in `~/.config/gitid/`:

```
~/.config/gitid/
├── profiles.yaml      # Identity profiles
├── rules.yaml         # Matching rules
├── activity.jsonl     # Activity log (for pattern learning)
└── hooks/
    └── pre-commit     # Guard hook script
```

## Architecture

Built with Rust for performance and safety:

- **gitid-core** — Core library: profiles, rules, resolver, SSH, keychain, guard, learning, detection, team constraints
- **gitid-cli** — CLI tool (clap-based, 10+ command families)
- **git-credential-gitid** — Git credential helper binary
- **tauri-app** — Desktop GUI (Tauri 2 + React + TypeScript + Tailwind)

## Contributing

```bash
# Clone and build
git clone https://github.com/vipinkashyap/gitid.git
cd gitid
cargo build

# Run the CLI
cargo run -p gitid-cli -- status

# Run the desktop app
cd tauri-app
npm install
cargo tauri dev

# Run tests
cargo test --workspace
```

## License

MIT — see [LICENSE](LICENSE) for details.
