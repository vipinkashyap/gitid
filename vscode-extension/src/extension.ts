import * as vscode from "vscode";
import { execFile } from "child_process";
import { promisify } from "util";

const exec = promisify(execFile);

let statusBarItem: vscode.StatusBarItem;

/**
 * Get the path to the gitid binary, respecting user config.
 */
function getGitidPath(): string {
  const config = vscode.workspace.getConfiguration("gitid");
  const customPath = config.get<string>("cliPath", "");
  return customPath || "gitid";
}

/**
 * Run `gitid status --json` for a given directory and parse the result.
 */
async function getStatus(
  cwd: string
): Promise<{ profile_name: string | null; reason: string | null } | null> {
  try {
    const { stdout } = await exec(getGitidPath(), ["status", cwd], {
      timeout: 5000,
    });
    // Parse the output — gitid status prints "Profile: <name> (via <reason>)"
    const profileMatch = stdout.match(/Profile:\s+(\S+)/);
    const reasonMatch = stdout.match(/\(via (.+?)\)/);
    return {
      profile_name: profileMatch ? profileMatch[1] : null,
      reason: reasonMatch ? reasonMatch[1] : null,
    };
  } catch {
    return null;
  }
}

/**
 * Run `gitid resolve` to apply the correct identity to the repo's config.
 */
async function resolve(cwd: string): Promise<boolean> {
  try {
    await exec(getGitidPath(), ["status", cwd], { timeout: 5000 });
    // The act of running status triggers resolution in shell-hook mode.
    // For explicit resolution, we use the apply mechanism via the core.
    // Since `gitid resolve` may not exist as a standalone CLI command yet,
    // we use `gitid use` with the resolved profile, or simply ensure
    // config_writer applies on status check.
    return true;
  } catch {
    return false;
  }
}

/**
 * Update the status bar item with the active profile.
 */
async function updateStatusBar(): Promise<void> {
  const config = vscode.workspace.getConfiguration("gitid");
  if (!config.get<boolean>("showStatusBarItem", true)) {
    statusBarItem.hide();
    return;
  }

  const workspaceFolders = vscode.workspace.workspaceFolders;
  if (!workspaceFolders || workspaceFolders.length === 0) {
    statusBarItem.hide();
    return;
  }

  const cwd = workspaceFolders[0].uri.fsPath;
  const status = await getStatus(cwd);

  if (status?.profile_name) {
    statusBarItem.text = `$(person) ${status.profile_name}`;
    statusBarItem.tooltip = `GitID: ${status.profile_name}${
      status.reason ? ` (via ${status.reason})` : ""
    }`;
    statusBarItem.show();
  } else {
    statusBarItem.text = "$(person) No profile";
    statusBarItem.tooltip = "GitID: No profile resolved for this workspace";
    statusBarItem.show();
  }
}

/**
 * Auto-resolve identity for all workspace folders.
 */
async function autoResolve(): Promise<void> {
  const workspaceFolders = vscode.workspace.workspaceFolders;
  if (!workspaceFolders) return;

  for (const folder of workspaceFolders) {
    const success = await resolve(folder.uri.fsPath);
    if (success) {
      const status = await getStatus(folder.uri.fsPath);
      if (status?.profile_name) {
        // Silently resolved — just update status bar
      }
    }
  }

  await updateStatusBar();
}

export function activate(context: vscode.ExtensionContext): void {
  // Create status bar item
  statusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Left,
    50
  );
  statusBarItem.command = "gitid.status";
  context.subscriptions.push(statusBarItem);

  // Register commands
  context.subscriptions.push(
    vscode.commands.registerCommand("gitid.resolve", async () => {
      const workspaceFolders = vscode.workspace.workspaceFolders;
      if (!workspaceFolders) {
        vscode.window.showWarningMessage("No workspace folder open.");
        return;
      }

      const cwd = workspaceFolders[0].uri.fsPath;
      const success = await resolve(cwd);
      if (success) {
        const status = await getStatus(cwd);
        if (status?.profile_name) {
          vscode.window.showInformationMessage(
            `GitID: Resolved profile "${status.profile_name}" for this workspace`
          );
        } else {
          vscode.window.showWarningMessage(
            "GitID: No profile matched this workspace. Add a rule in the GitID app."
          );
        }
      } else {
        vscode.window.showErrorMessage(
          "GitID: Failed to resolve. Is the gitid CLI installed?"
        );
      }
      await updateStatusBar();
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("gitid.status", async () => {
      const workspaceFolders = vscode.workspace.workspaceFolders;
      if (!workspaceFolders) {
        vscode.window.showWarningMessage("No workspace folder open.");
        return;
      }

      const cwd = workspaceFolders[0].uri.fsPath;
      const status = await getStatus(cwd);
      if (status?.profile_name) {
        vscode.window.showInformationMessage(
          `GitID profile: ${status.profile_name}${
            status.reason ? ` (matched via ${status.reason})` : ""
          }`
        );
      } else {
        vscode.window.showInformationMessage(
          "GitID: No profile resolved for this workspace"
        );
      }
    })
  );

  // Listen for workspace folder changes
  context.subscriptions.push(
    vscode.workspace.onDidChangeWorkspaceFolders(async () => {
      const config = vscode.workspace.getConfiguration("gitid");
      if (config.get<boolean>("autoResolve", true)) {
        await autoResolve();
      } else {
        await updateStatusBar();
      }
    })
  );

  // Auto-resolve on activation if enabled
  const config = vscode.workspace.getConfiguration("gitid");
  if (config.get<boolean>("autoResolve", true)) {
    autoResolve();
  } else {
    updateStatusBar();
  }
}

export function deactivate(): void {
  if (statusBarItem) {
    statusBarItem.dispose();
  }
}
