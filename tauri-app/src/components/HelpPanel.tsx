import { useState } from "react";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Badge,
} from "./ui/primitives";
import {
  ChevronRight,
  Lightbulb,
  Users,
  GitFork,
  Shield,
  Terminal,
  Zap,
  BookOpen,
  HelpCircle,
  Keyboard,
  Monitor,
  Download,
} from "lucide-react";

// =============================================================================
// Expandable Help Section
// =============================================================================

interface HelpSectionProps {
  icon: React.ReactNode;
  title: string;
  badge?: string;
  children: React.ReactNode;
  defaultOpen?: boolean;
}

function HelpSection({
  icon,
  title,
  badge,
  children,
  defaultOpen = false,
}: HelpSectionProps) {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <Card>
      <button
        onClick={() => setOpen(!open)}
        className="w-full text-left"
        aria-expanded={open}
      >
        <CardHeader className="pb-3 pt-4 px-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2.5">
              <div className="text-muted-foreground">{icon}</div>
              <CardTitle className="text-sm font-medium">{title}</CardTitle>
              {badge && (
                <Badge variant="outline" className="text-xs">
                  {badge}
                </Badge>
              )}
            </div>
            <ChevronRight
              className={`h-4 w-4 text-muted-foreground transition-transform ${
                open ? "rotate-90" : ""
              }`}
            />
          </div>
        </CardHeader>
      </button>
      {open && (
        <CardContent className="px-4 pb-4 pt-0">
          <div className="text-sm text-muted-foreground space-y-3 leading-relaxed">
            {children}
          </div>
        </CardContent>
      )}
    </Card>
  );
}

// =============================================================================
// Shortcut Key badge
// =============================================================================

function Kbd({ children }: { children: React.ReactNode }) {
  return (
    <kbd className="inline-flex items-center px-1.5 py-0.5 rounded border bg-muted text-xs font-mono text-muted-foreground">
      {children}
    </kbd>
  );
}

// =============================================================================
// HelpPanel (main export)
// =============================================================================

export default function HelpPanel() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-xl font-semibold">Help</h2>
        <p className="text-sm text-muted-foreground">
          Learn how to use GitID to manage your Git identities
        </p>
      </div>

      {/* Quick Start */}
      <div className="space-y-2">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground px-1">
          Getting Started
        </h3>

        <HelpSection
          icon={<Download className="h-4 w-4" />}
          title="Installing the CLI"
          defaultOpen={true}
        >
          <p>
            This desktop app manages your profiles and rules, but the CLI is
            needed for shell integration, Identity Guard hooks, and IDE
            commands like{" "}
            <code className="text-xs font-mono bg-muted px-1 rounded">gitid resolve</code>.
          </p>
          <div className="space-y-2">
            <p className="font-medium text-foreground">Option 1: Install script</p>
            <code className="block text-xs font-mono bg-muted p-2 rounded">
              curl -fsSL https://gitid.dev/install | sh
            </code>
          </div>
          <div className="space-y-2">
            <p className="font-medium text-foreground">Option 2: Build from source</p>
            <code className="block text-xs font-mono bg-muted p-2 rounded whitespace-pre">
              {"cargo install --path crates/gitid-cli\ncargo install --path crates/git-credential-gitid"}
            </code>
          </div>
          <div className="rounded-lg border border-amber-500/20 bg-amber-500/5 p-3">
            <p className="text-xs">
              <span className="font-medium text-amber-400">Check:</span> After
              installing, run{" "}
              <code className="font-mono bg-muted px-1 rounded">gitid --version</code>{" "}
              in a new terminal. If you see "command not found", make sure{" "}
              <code className="font-mono bg-muted px-1 rounded">~/.local/bin</code>{" "}
              (or{" "}
              <code className="font-mono bg-muted px-1 rounded">~/.cargo/bin</code>)
              is in your PATH.
            </p>
          </div>
        </HelpSection>

        <HelpSection
          icon={<Lightbulb className="h-4 w-4" />}
          title="What is GitID?"
        >
          <p>
            GitID manages multiple Git identities so you never commit with the
            wrong name, email, or SSH key. If you use separate accounts for
            work, personal projects, and open-source, GitID automatically
            switches between them based on rules you define.
          </p>
          <div className="rounded-lg border p-3 space-y-2">
            <p className="font-medium text-foreground text-xs uppercase tracking-wide">
              Quick setup in 3 steps
            </p>
            <ol className="space-y-1.5 text-sm pl-4">
              <li>
                <span className="font-medium text-foreground">1.</span>{" "}
                Create a profile for each identity in the{" "}
                <span className="font-medium text-foreground">Profiles</span>{" "}
                tab
              </li>
              <li>
                <span className="font-medium text-foreground">2.</span>{" "}
                Add rules in the{" "}
                <span className="font-medium text-foreground">Rules</span> tab
                to map directories or remotes to profiles
              </li>
              <li>
                <span className="font-medium text-foreground">3.</span>{" "}
                Enable Identity Guard on the{" "}
                <span className="font-medium text-foreground">Dashboard</span>{" "}
                to prevent accidental mismatches
              </li>
            </ol>
          </div>
        </HelpSection>
      </div>

      {/* Feature Guides */}
      <div className="space-y-2">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground px-1">
          Features
        </h3>

        <HelpSection
          icon={<Users className="h-4 w-4" />}
          title="Profiles"
        >
          <p>
            A profile is a named Git identity containing a name, email, and
            optionally an SSH key, signing key, and associated hostnames.
          </p>
          <div className="space-y-2">
            <p className="font-medium text-foreground">Common setups:</p>
            <ul className="space-y-1 pl-3">
              <li>
                <span className="text-foreground font-medium">Work</span> —
                your corporate email + work SSH key
              </li>
              <li>
                <span className="text-foreground font-medium">Personal</span> —
                personal email + personal SSH key
              </li>
              <li>
                <span className="text-foreground font-medium">OSS</span> —
                public-facing email for open-source contributions
              </li>
            </ul>
          </div>
          <p>
            You can test SSH connectivity for any profile to verify your keys
            are properly configured with each Git host.
          </p>
        </HelpSection>

        <HelpSection
          icon={<GitFork className="h-4 w-4" />}
          title="Rules & Profile Resolution"
        >
          <p>
            Rules determine which profile is used for each repository. GitID
            checks rules in priority order and uses the first match.
          </p>
          <div className="space-y-2">
            <p className="font-medium text-foreground">Rule types:</p>
            <ul className="space-y-1.5 pl-3">
              <li>
                <Badge variant="outline" className="text-xs mr-1.5 bg-blue-500/10 text-blue-400 border-blue-500/20">
                  Directory
                </Badge>
                Match repos by file path (e.g.{" "}
                <code className="text-xs font-mono bg-muted px-1 rounded">~/work/**</code>)
              </li>
              <li>
                <Badge variant="outline" className="text-xs mr-1.5 bg-purple-500/10 text-purple-400 border-purple-500/20">
                  Remote
                </Badge>
                Match by remote URL pattern (e.g.{" "}
                <code className="text-xs font-mono bg-muted px-1 rounded">github.com/mycompany/*</code>)
              </li>
              <li>
                <Badge variant="outline" className="text-xs mr-1.5 bg-amber-500/10 text-amber-400 border-amber-500/20">
                  Hostname
                </Badge>
                Match by Git host (e.g.{" "}
                <code className="text-xs font-mono bg-muted px-1 rounded">gitlab.internal.com</code>)
              </li>
            </ul>
          </div>
          <p>
            Rules are evaluated top-to-bottom within each type. Use the arrow
            buttons to change priority. You can also set a default profile as
            a fallback when no rule matches.
          </p>
        </HelpSection>

        <HelpSection
          icon={<Shield className="h-4 w-4" />}
          title="Identity Guard"
          badge="Pre-commit"
        >
          <p>
            Identity Guard is a Git pre-commit hook that checks your configured
            identity against the expected profile before every commit. If
            there's a mismatch, the commit is blocked until you fix it.
          </p>
          <div className="space-y-2">
            <p className="font-medium text-foreground">States:</p>
            <ul className="space-y-1 pl-3">
              <li>
                <span className="text-emerald-400 font-medium">OK</span> —
                identity matches the expected profile
              </li>
              <li>
                <span className="text-amber-400 font-medium">Mismatch</span> —
                wrong identity detected; click "Fix Now" to correct it
              </li>
              <li>
                <span className="text-muted-foreground font-medium">No profile</span> —
                no rule matched this repo
              </li>
            </ul>
          </div>
          <p>
            Enable Guard from the Dashboard. It uses Git's{" "}
            <code className="text-xs font-mono bg-muted px-1 rounded">core.hooksPath</code>{" "}
            to install a lightweight hook script.
          </p>
        </HelpSection>

        <HelpSection
          icon={<Zap className="h-4 w-4" />}
          title="Smart Suggestions"
          badge="Beta"
        >
          <p>
            GitID learns from your activity over time. When it notices
            patterns — like you always use a certain profile in repos under a
            specific directory — it suggests new rules to save you time.
          </p>
          <p>
            Suggestions appear on the Dashboard once enough evidence has been
            gathered (at least 3 matching observations). You can accept to
            create the rule automatically, or dismiss if it's not useful.
          </p>
        </HelpSection>
      </div>

      {/* CLI & Shell */}
      <div className="space-y-2">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground px-1">
          CLI & Terminal
        </h3>

        <HelpSection
          icon={<Terminal className="h-4 w-4" />}
          title="Shell Integration"
          badge="Requires CLI"
        >
          <p>
            The shell hook automatically switches identities when you{" "}
            <code className="text-xs font-mono bg-muted px-1 rounded">cd</code>{" "}
            into a repository. Make sure you've installed the CLI first (see
            "Installing the CLI" above).
          </p>
          <div className="rounded-lg border p-3 space-y-2">
            <p className="font-medium text-foreground text-xs">
              Add to your shell profile:
            </p>
            <code className="block text-xs font-mono bg-muted p-2 rounded">
              eval "$(gitid shell-hook)"
            </code>
          </div>
          <div className="space-y-2">
            <p className="font-medium text-foreground">Useful commands:</p>
            <ul className="space-y-1 pl-3 font-mono text-xs">
              <li>
                <span className="text-foreground">gitid status</span>{" "}
                <span className="font-sans">— show active profile</span>
              </li>
              <li>
                <span className="text-foreground">gitid doctor</span>{" "}
                <span className="font-sans">— check configuration health</span>
              </li>
              <li>
                <span className="text-foreground">gitid list</span>{" "}
                <span className="font-sans">— list all profiles</span>
              </li>
              <li>
                <span className="text-foreground">gitid scan ~/projects</span>{" "}
                <span className="font-sans">— find repos and their identities</span>
              </li>
            </ul>
          </div>
        </HelpSection>
      </div>

      {/* IDE & Editors */}
      <div className="space-y-2">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground px-1">
          IDE & Editors
        </h3>

        <HelpSection
          icon={<Monitor className="h-4 w-4" />}
          title="VS Code"
          badge="Requires CLI"
        >
          <p>
            GitID works with VS Code in two ways depending on how you use Git.
          </p>
          <div className="space-y-2">
            <p className="font-medium text-foreground">Integrated terminal</p>
            <p>
              If you have the shell hook installed, VS Code's terminal picks it
              up automatically. Identities switch when you{" "}
              <code className="text-xs font-mono bg-muted px-1 rounded">cd</code>{" "}
              between repos, just like a standalone terminal.
            </p>
          </div>
          <div className="space-y-2">
            <p className="font-medium text-foreground">Source Control panel</p>
            <p>
              When you commit from the VS Code sidebar, the shell hook doesn't
              run — but Identity Guard still fires. If the identity is wrong,
              the commit will be blocked and VS Code will show the error in
              its Git output.
            </p>
          </div>
          <div className="rounded-lg border p-3 space-y-2">
            <p className="font-medium text-foreground text-xs">
              Recommended: run this once per project
            </p>
            <code className="block text-xs font-mono bg-muted p-2 rounded">
              gitid resolve
            </code>
            <p className="text-xs">
              This writes the correct identity to the repo's local Git config.
              After that, VS Code's Source Control panel will use it for all
              commits — no shell hook needed.
            </p>
          </div>
        </HelpSection>

        <HelpSection
          icon={<Monitor className="h-4 w-4" />}
          title="JetBrains (IntelliJ, WebStorm, etc.)"
          badge="Requires CLI"
        >
          <p>
            Same story as VS Code: the built-in terminal works with the shell
            hook, and JetBrains' own Git integration respects whatever is in
            the repo's local config.
          </p>
          <div className="rounded-lg border p-3 space-y-2">
            <p className="font-medium text-foreground text-xs">
              Set up once per project
            </p>
            <p className="text-xs">
              Open the JetBrains terminal and run{" "}
              <code className="font-mono bg-muted px-1 rounded">gitid resolve</code>.
              This ensures the correct name, email, and SSH key are in place
              before your first commit.
            </p>
          </div>
          <p>
            Identity Guard works with JetBrains commit dialogs too — a
            mismatch will show as a pre-commit hook failure in the commit
            results panel.
          </p>
        </HelpSection>

        <HelpSection
          icon={<Monitor className="h-4 w-4" />}
          title="Other editors & GUI clients"
          badge="Requires CLI"
        >
          <p>
            GitID is compatible with any tool that runs Git commands under the
            hood — Sublime Merge, GitKraken, Tower, Fork, and others.
          </p>
          <div className="space-y-2">
            <p className="font-medium text-foreground">What works everywhere:</p>
            <ul className="space-y-1 pl-3">
              <li>
                <span className="font-medium text-foreground">Identity Guard</span> —
                the pre-commit hook fires on every commit, regardless of the tool
              </li>
              <li>
                <span className="font-medium text-foreground">gitid resolve</span> —
                run once per project to write the identity into the repo config
              </li>
              <li>
                <span className="font-medium text-foreground">Shell hook</span> —
                works in any embedded terminal that loads your shell profile
              </li>
            </ul>
          </div>
          <div className="rounded-lg border border-amber-500/20 bg-amber-500/5 p-3">
            <p className="text-xs">
              <span className="font-medium text-amber-400">Tip:</span> If you
              open projects from the IDE launcher (File → Open Recent) without
              using a terminal, always run{" "}
              <code className="font-mono bg-muted px-1 rounded">gitid resolve</code>{" "}
              first, or rely on Identity Guard to catch mismatches at commit
              time.
            </p>
          </div>
        </HelpSection>
      </div>

      {/* Keyboard Shortcuts & Tips */}
      <div className="space-y-2">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground px-1">
          Tips
        </h3>

        <HelpSection
          icon={<Keyboard className="h-4 w-4" />}
          title="Keyboard Navigation"
        >
          <p>
            GitID is fully keyboard-accessible. All interactive elements can
            be reached with <Kbd>Tab</Kbd> and activated with <Kbd>Enter</Kbd>{" "}
            or <Kbd>Space</Kbd>.
          </p>
          <ul className="space-y-1 pl-3">
            <li>
              <Kbd>←</Kbd> <Kbd>→</Kbd> to switch between tabs
            </li>
            <li>
              <Kbd>Tab</Kbd> to move through interactive elements
            </li>
            <li>
              <Kbd>Esc</Kbd> to close any dialog
            </li>
          </ul>
        </HelpSection>

        <HelpSection
          icon={<BookOpen className="h-4 w-4" />}
          title="Doctor Tab"
        >
          <p>
            The Doctor tab runs a health check on your entire GitID setup. It
            verifies that your profiles are valid, SSH keys exist, rules
            reference real profiles, and your shell integration is properly
            configured.
          </p>
          <p>
            Each check shows a status (OK, Warning, or Error) along with a
            suggested fix if something needs attention.
          </p>
        </HelpSection>

        <HelpSection
          icon={<HelpCircle className="h-4 w-4" />}
          title="Troubleshooting"
        >
          <div className="space-y-3">
            <div>
              <p className="font-medium text-foreground">
                "gitid: command not found"
              </p>
              <p>
                The CLI isn't installed or isn't on your PATH. See "Installing
                the CLI" at the top of this page. After installing, open a new
                terminal window — existing terminals won't pick up the change.
                Check with{" "}
                <code className="text-xs font-mono bg-muted px-1 rounded">
                  which gitid
                </code>{" "}
                to confirm it's accessible.
              </p>
            </div>
            <div>
              <p className="font-medium text-foreground">
                "No profile resolved" for a repo
              </p>
              <p>
                Add a directory or remote rule that matches the repo's path or
                remote URL. Use the Profile Resolution Check on the Dashboard
                to test which profile resolves for any path.
              </p>
            </div>
            <div>
              <p className="font-medium text-foreground">
                SSH connection test fails
              </p>
              <p>
                Ensure the SSH key path in your profile points to a valid
                private key, and that the key is added to your Git host
                (GitHub, GitLab, etc). Run{" "}
                <code className="text-xs font-mono bg-muted px-1 rounded">
                  ssh-add -l
                </code>{" "}
                to check loaded keys.
              </p>
            </div>
            <div>
              <p className="font-medium text-foreground">
                Guard blocks every commit
              </p>
              <p>
                The guard checks that the repo's configured email matches the
                expected profile. Click "Fix Now" on the Dashboard, or make
                sure your rules correctly map the repo to the right profile.
              </p>
            </div>
            <div>
              <p className="font-medium text-foreground">
                Shell hook doesn't activate
              </p>
              <p>
                Verify that{" "}
                <code className="text-xs font-mono bg-muted px-1 rounded">
                  eval "$(gitid shell-hook)"
                </code>{" "}
                is in your shell profile (~/.bashrc, ~/.zshrc, or config.fish).
                Open a new terminal window after adding it.
              </p>
            </div>
          </div>
        </HelpSection>
      </div>
    </div>
  );
}
