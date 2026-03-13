import { useState, useEffect } from "react";
import * as api from "@/lib/tauri-api";
import { Card, CardContent, Button, Badge } from "./ui/primitives";
import {
  Terminal,
  Download,
  CheckCircle2,
  AlertTriangle,
  Loader2,
  ExternalLink,
} from "lucide-react";

export default function CliInstallBanner() {
  const [status, setStatus] = useState<api.CliStatusDto | null>(null);
  const [installing, setInstalling] = useState(false);
  const [installResult, setInstallResult] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [dismissed, setDismissed] = useState(false);

  useEffect(() => {
    api.checkCliInstalled().then(setStatus).catch(() => {});
  }, [installResult]);

  // Don't render if CLI is already installed or user dismissed
  if (dismissed) return null;
  if (status === null) return null;
  if (status.installed) {
    // Show a small success indicator briefly, or nothing
    return null;
  }

  const handleInstall = async () => {
    setInstalling(true);
    setError(null);
    try {
      const dir = await api.installCli();
      setInstallResult(dir);
    } catch (e) {
      setError(String(e));
    } finally {
      setInstalling(false);
    }
  };

  return (
    <Card className="border-amber-500/30 bg-amber-500/5">
      <CardContent className="p-4">
        {installResult ? (
          <div className="flex items-center gap-3">
            <CheckCircle2 className="h-5 w-5 text-emerald-400 shrink-0" />
            <div className="flex-1">
              <p className="text-sm font-medium">CLI installed</p>
              <p className="text-xs text-muted-foreground">
                Installed to{" "}
                <code className="font-mono bg-muted px-1 rounded">
                  {installResult}
                </code>
                . Open a new terminal to use{" "}
                <code className="font-mono bg-muted px-1 rounded">gitid</code>{" "}
                commands.
              </p>
            </div>
          </div>
        ) : (
          <div className="flex items-start gap-3">
            <AlertTriangle className="h-5 w-5 text-amber-400 shrink-0 mt-0.5" />
            <div className="flex-1 space-y-2">
              <div>
                <p className="text-sm font-medium">CLI not installed</p>
                <p className="text-xs text-muted-foreground">
                  Install the GitID command-line tool to enable shell
                  integration, Identity Guard hooks, and IDE support.
                </p>
              </div>
              {error && (
                <div className="rounded border border-destructive/30 bg-destructive/5 p-2">
                  <p className="text-xs text-destructive">{error}</p>
                  <p className="text-xs text-muted-foreground mt-1">
                    You can install manually:{" "}
                    <code className="font-mono bg-muted px-1 rounded">
                      cargo install --path crates/gitid-cli
                    </code>
                  </p>
                </div>
              )}
              <div className="flex items-center gap-2">
                <Button size="sm" onClick={handleInstall} disabled={installing}>
                  {installing ? (
                    <Loader2 className="h-3.5 w-3.5 animate-spin mr-1.5" />
                  ) : (
                    <Download className="h-3.5 w-3.5 mr-1.5" />
                  )}
                  Install CLI
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setDismissed(true)}
                >
                  Later
                </Button>
              </div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
