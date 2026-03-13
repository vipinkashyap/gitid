import { useState } from "react";
import { useDoctor } from "@/lib/hooks";
import * as api from "@/lib/tauri-api";
import type { DoctorCheck } from "@/lib/tauri-api";
import { Card, CardContent, Button, Badge } from "./ui/primitives";
import {
  CheckCircle2,
  AlertTriangle,
  XCircle,
  RefreshCw,
  Download,
  Loader2,
  Stethoscope,
} from "lucide-react";

// =============================================================================
// Status icons
// =============================================================================

function StatusIcon({ status }: { status: string }) {
  switch (status) {
    case "ok":
      return <CheckCircle2 className="h-4 w-4 text-emerald-400" />;
    case "warning":
      return <AlertTriangle className="h-4 w-4 text-amber-400" />;
    case "error":
      return <XCircle className="h-4 w-4 text-red-400" />;
    default:
      return null;
  }
}

// =============================================================================
// Check Row
// =============================================================================

function CheckRow({ check }: { check: DoctorCheck }) {
  return (
    <div className="flex items-start gap-3 p-3 rounded-lg border">
      <StatusIcon status={check.status} />
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium">{check.name}</span>
          <Badge
            variant={
              check.status === "ok"
                ? "success"
                : check.status === "warning"
                ? "warning"
                : "destructive"
            }
          >
            {check.status}
          </Badge>
        </div>
        <p className="text-xs text-muted-foreground mt-0.5">{check.message}</p>
        {check.fix && (
          <p className="text-xs text-blue-400 mt-1 font-mono">{check.fix}</p>
        )}
      </div>
    </div>
  );
}

// =============================================================================
// DoctorView (main export)
// =============================================================================

export default function DoctorView() {
  const { data: checks, loading, error, refresh } = useDoctor();
  const [installing, setInstalling] = useState(false);

  const handleInstall = async () => {
    setInstalling(true);
    try {
      await api.installCredentialHelper();
      refresh();
    } catch {
      // Error will show in doctor checks
    } finally {
      setInstalling(false);
    }
  };

  const okCount = checks?.filter((c) => c.status === "ok").length ?? 0;
  const warnCount = checks?.filter((c) => c.status === "warning").length ?? 0;
  const errCount = checks?.filter((c) => c.status === "error").length ?? 0;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold flex items-center gap-2">
            <Stethoscope className="h-5 w-5" />
            Doctor
          </h2>
          <p className="text-sm text-muted-foreground">
            Verify your GitID configuration health
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={handleInstall} disabled={installing}>
            {installing ? (
              <Loader2 className="h-4 w-4 mr-2 animate-spin" />
            ) : (
              <Download className="h-4 w-4 mr-2" />
            )}
            Install Helper
          </Button>
          <Button variant="outline" onClick={refresh} disabled={loading}>
            {loading ? (
              <Loader2 className="h-4 w-4 mr-2 animate-spin" />
            ) : (
              <RefreshCw className="h-4 w-4 mr-2" />
            )}
            Re-check
          </Button>
        </div>
      </div>

      {/* Summary */}
      {checks && (
        <div className="grid grid-cols-3 gap-4">
          <Card>
            <CardContent className="p-4 flex items-center gap-3">
              <CheckCircle2 className="h-8 w-8 text-emerald-400" />
              <div>
                <p className="text-2xl font-bold">{okCount}</p>
                <p className="text-xs text-muted-foreground">Passed</p>
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="p-4 flex items-center gap-3">
              <AlertTriangle className="h-8 w-8 text-amber-400" />
              <div>
                <p className="text-2xl font-bold">{warnCount}</p>
                <p className="text-xs text-muted-foreground">Warnings</p>
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="p-4 flex items-center gap-3">
              <XCircle className="h-8 w-8 text-red-400" />
              <div>
                <p className="text-2xl font-bold">{errCount}</p>
                <p className="text-xs text-muted-foreground">Errors</p>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Loading */}
      {loading && (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
        </div>
      )}

      {/* Error */}
      {error && (
        <div className="text-center py-12 text-destructive">
          <p>Failed to run checks: {error}</p>
        </div>
      )}

      {/* Check Results */}
      {checks && (
        <div className="space-y-2">
          {checks.map((check, i) => (
            <CheckRow key={i} check={check} />
          ))}
        </div>
      )}

      {/* All clear */}
      {checks && errCount === 0 && warnCount === 0 && (
        <Card className="border-emerald-500/20 bg-emerald-500/5">
          <CardContent className="p-4 text-center">
            <p className="text-emerald-400 font-medium">
              All checks passed! GitID is healthy.
            </p>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
