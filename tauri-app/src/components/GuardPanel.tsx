import { useState } from "react";
import { useGuardStatus } from "@/lib/hooks";
import * as api from "@/lib/tauri-api";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Button,
  Badge,
  HelpTip,
} from "./ui/primitives";
import {
  ShieldCheck,
  ShieldAlert,
  ShieldOff,
  Loader2,
  Power,
  PowerOff,
  AlertTriangle,
  CheckCircle2,
  XCircle,
  Mail,
  Wrench,
} from "lucide-react";

// =============================================================================
// Guard Verdict Display
// =============================================================================

function VerdictDisplay({
  verdict,
  profile,
  expectedEmail,
  actualEmail,
  onFix,
}: {
  verdict: string;
  profile: string | null;
  expectedEmail: string | null;
  actualEmail: string | null;
  onFix: () => Promise<void>;
}) {
  const [fixing, setFixing] = useState(false);

  const handleFix = async () => {
    setFixing(true);
    try {
      await onFix();
    } finally {
      setFixing(false);
    }
  };

  switch (verdict) {
    case "ok":
      return (
        <div className="flex items-start gap-3 p-3 rounded-lg border border-emerald-500/20 bg-emerald-500/5">
          <CheckCircle2 className="h-4 w-4 text-emerald-400 mt-0.5 shrink-0" />
          <div className="space-y-1">
            <p className="text-sm font-medium text-emerald-400">
              Identity matches
            </p>
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <Badge variant="success">{profile}</Badge>
              <span className="flex items-center gap-1">
                <Mail className="h-3 w-3" />
                {expectedEmail}
              </span>
            </div>
          </div>
        </div>
      );

    case "mismatch":
      return (
        <div className="flex items-start gap-3 p-3 rounded-lg border border-amber-500/20 bg-amber-500/5">
          <AlertTriangle className="h-4 w-4 text-amber-400 mt-0.5 shrink-0" />
          <div className="flex-1 space-y-1">
            <div className="flex items-center justify-between">
              <p className="text-sm font-medium text-amber-400">
                Identity mismatch
              </p>
              <Button
                variant="outline"
                size="sm"
                onClick={handleFix}
                disabled={fixing}
                aria-label="Fix identity mismatch"
              >
                {fixing ? (
                  <Loader2 className="h-3.5 w-3.5 animate-spin mr-1.5" />
                ) : (
                  <Wrench className="h-3.5 w-3.5 mr-1.5" />
                )}
                Fix Now
              </Button>
            </div>
            <div className="space-y-1 text-xs text-muted-foreground">
              <div className="flex items-center gap-2">
                <span className="w-16 text-right">Expected:</span>
                <Badge variant="success">{profile}</Badge>
                <span>{expectedEmail}</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="w-16 text-right">Actual:</span>
                <Badge variant="warning">{actualEmail ?? "unknown"}</Badge>
              </div>
            </div>
          </div>
        </div>
      );

    case "no_profile":
      return (
        <div className="flex items-center gap-3 p-3 rounded-lg border text-sm text-muted-foreground">
          <ShieldOff className="h-4 w-4 shrink-0" />
          <span>No profile resolved for the current directory.</span>
        </div>
      );

    case "not_a_repo":
      return (
        <div className="flex items-center gap-3 p-3 rounded-lg border text-sm text-muted-foreground">
          <XCircle className="h-4 w-4 shrink-0" />
          <span>Not inside a Git repository.</span>
        </div>
      );

    default:
      return null;
  }
}

// =============================================================================
// GuardPanel (main export)
// =============================================================================

export default function GuardPanel() {
  const { data: status, loading, refresh } = useGuardStatus();
  const [toggling, setToggling] = useState(false);

  const handleToggle = async () => {
    setToggling(true);
    try {
      if (status?.installed) {
        await api.guardUninstall();
      } else {
        await api.guardInstall();
      }
      refresh();
    } catch {
      // Error will surface on refresh
    } finally {
      setToggling(false);
    }
  };

  const handleFix = async () => {
    await api.guardFix();
    refresh();
  };

  const installed = status?.installed ?? false;

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="text-base flex items-center justify-between">
          <div className="flex items-center gap-2">
            {installed ? (
              <ShieldCheck className="h-4 w-4 text-emerald-400" />
            ) : (
              <ShieldAlert className="h-4 w-4 text-muted-foreground" />
            )}
            Identity Guard
            <HelpTip text="Pre-commit hook that blocks commits when your Git identity doesn't match the expected profile." />
          </div>
          <div className="flex items-center gap-2">
            <Badge variant={installed ? "success" : "outline"}>
              {installed ? "Active" : "Inactive"}
            </Badge>
            <Button
              variant="outline"
              size="sm"
              onClick={handleToggle}
              disabled={toggling || loading}
              aria-label={installed ? "Disable identity guard" : "Enable identity guard"}
            >
              {toggling ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : installed ? (
                <>
                  <PowerOff className="h-3.5 w-3.5 mr-1.5" />
                  Disable
                </>
              ) : (
                <>
                  <Power className="h-3.5 w-3.5 mr-1.5" />
                  Enable
                </>
              )}
            </Button>
          </div>
        </CardTitle>
        <p className="text-xs text-muted-foreground">
          Pre-commit hook that catches wrong-identity commits before they happen.
        </p>
      </CardHeader>
      <CardContent className="space-y-3">
        {loading && (
          <div className="flex items-center justify-center py-4">
            <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
          </div>
        )}

        {!loading && status && (
          <VerdictDisplay
            verdict={status.verdict}
            profile={status.profile}
            expectedEmail={status.expected_email}
            actualEmail={status.actual_email}
            onFix={handleFix}
          />
        )}

        {!loading && !installed && (
          <p className="text-xs text-muted-foreground">
            When enabled, GitID installs a global{" "}
            <code className="font-mono">pre-commit</code> hook via{" "}
            <code className="font-mono">core.hooksPath</code>. It checks your
            identity before each commit and warns if there's a mismatch.
          </p>
        )}
      </CardContent>
    </Card>
  );
}
