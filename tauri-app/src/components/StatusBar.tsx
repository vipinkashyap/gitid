import { useState } from "react";
import { useStatus, useRepoScan, useProfiles } from "@/lib/hooks";
import type { DetectedRepo } from "@/lib/tauri-api";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Button,
  Badge,
  Input,
} from "./ui/primitives";
import {
  FolderSearch,
  GitBranch,
  User,
  Mail,
  Key,
  MapPin,
  Search,
  Loader2,
} from "lucide-react";
import GuardPanel from "./GuardPanel";
import SuggestionsPanel from "./SuggestionsPanel";

// =============================================================================
// Status Check Panel
// =============================================================================

function StatusCheck() {
  const [checkPath, setCheckPath] = useState("");
  const { data: status, loading, refresh } = useStatus(checkPath || undefined);

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="text-base flex items-center gap-2">
          <MapPin className="h-4 w-4" />
          Profile Resolution Check
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="flex gap-2">
          <Input
            value={checkPath}
            onChange={(e) => setCheckPath(e.target.value)}
            placeholder="Enter a directory path to check..."
            className="flex-1"
          />
          <Button onClick={refresh} disabled={loading}>
            {loading ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <Search className="h-4 w-4" />
            )}
          </Button>
        </div>

        {status && (
          <div className="rounded-lg border p-4 space-y-2">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <FolderSearch className="h-4 w-4" />
              <span className="font-mono text-xs truncate">
                {status.directory}
              </span>
            </div>

            {status.profile_name ? (
              <>
                <div className="flex items-center gap-2">
                  <Badge variant="success" className="text-sm">
                    {status.profile_name}
                  </Badge>
                  {status.reason && (
                    <span className="text-xs text-muted-foreground">
                      via {status.reason}
                    </span>
                  )}
                </div>
                {status.profile && (
                  <div className="space-y-1 text-sm text-muted-foreground pl-2 border-l-2 border-border">
                    <div className="flex items-center gap-2">
                      <User className="h-3 w-3" />
                      {status.profile.name}
                    </div>
                    <div className="flex items-center gap-2">
                      <Mail className="h-3 w-3" />
                      {status.profile.email}
                    </div>
                    {status.profile.ssh_key && (
                      <div className="flex items-center gap-2">
                        <Key className="h-3 w-3" />
                        <span className="font-mono text-xs">
                          {status.profile.ssh_key}
                        </span>
                      </div>
                    )}
                  </div>
                )}
              </>
            ) : (
              <p className="text-sm text-amber-400">
                No profile resolved for this directory.
              </p>
            )}

            {status.remote_url && (
              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                <GitBranch className="h-3 w-3" />
                <span className="font-mono truncate">{status.remote_url}</span>
              </div>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

// =============================================================================
// Repo Scanner Panel
// =============================================================================

function RepoScanner() {
  const [scanDir, setScanDir] = useState("");
  const [scanning, setScanning] = useState(false);
  const { data: repos, refresh } = useRepoScan(scanning ? scanDir : null);
  const { data: profiles } = useProfiles();

  const handleScan = () => {
    if (scanDir) {
      setScanning(true);
      refresh();
    }
  };

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="text-base flex items-center gap-2">
          <FolderSearch className="h-4 w-4" />
          Detect Repositories
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="flex gap-2">
          <Input
            value={scanDir}
            onChange={(e) => setScanDir(e.target.value)}
            placeholder="~/projects"
            className="flex-1"
          />
          <Button onClick={handleScan} disabled={!scanDir}>
            Scan
          </Button>
        </div>

        {repos && repos.length > 0 && (
          <div className="space-y-2 max-h-80 overflow-y-auto">
            {repos.map((repo) => (
              <RepoRow
                key={repo.path}
                repo={repo}
                profileNames={profiles ? Object.keys(profiles) : []}
              />
            ))}
            <p className="text-xs text-muted-foreground text-center pt-2">
              Found {repos.length} repo(s)
            </p>
          </div>
        )}

        {repos && repos.length === 0 && scanning && (
          <p className="text-sm text-muted-foreground text-center py-4">
            No git repos found in this directory.
          </p>
        )}
      </CardContent>
    </Card>
  );
}

// =============================================================================
// Repo Row
// =============================================================================

function RepoRow({
  repo,
  profileNames: _profileNames,
}: {
  repo: DetectedRepo;
  profileNames: string[];
}) {
  return (
    <div className="flex items-center gap-3 p-2.5 rounded-lg border text-sm">
      <GitBranch className="h-4 w-4 text-muted-foreground shrink-0" />
      <div className="flex-1 min-w-0">
        <div className="font-medium truncate">{repo.name}</div>
        {repo.remote_url && (
          <div className="text-xs text-muted-foreground font-mono truncate">
            {repo.remote_url}
          </div>
        )}
      </div>
      <div className="flex items-center gap-2 shrink-0">
        {repo.current_profile ? (
          <Badge variant="success">{repo.current_profile}</Badge>
        ) : repo.current_email ? (
          <Badge variant="outline" className="text-xs">
            {repo.current_email}
          </Badge>
        ) : (
          <Badge variant="warning">no identity</Badge>
        )}
      </div>
    </div>
  );
}

// =============================================================================
// Overview Stats
// =============================================================================

function OverviewStats() {
  const { data: profiles } = useProfiles();
  const { data: status } = useStatus();

  const profileCount = profiles ? Object.keys(profiles).length : 0;

  return (
    <Card>
      <CardContent className="p-3">
        <div className="flex items-center justify-between gap-4 text-sm">
          <div className="flex items-center gap-2">
            <User className="h-3.5 w-3.5 text-muted-foreground" />
            <span className="font-medium">{profileCount}</span>
            <span className="text-muted-foreground">profile{profileCount !== 1 ? "s" : ""}</span>
          </div>
          <div className="h-4 border-l" />
          <div className="flex items-center gap-2 flex-1 min-w-0">
            {status?.profile_name ? (
              <>
                <Badge variant="success">{status.profile_name}</Badge>
                <span className="text-xs text-muted-foreground truncate">
                  via {status.reason ?? "default"}
                </span>
              </>
            ) : (
              <span className="text-muted-foreground">No active profile</span>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

// =============================================================================
// StatusBar (main export) — Dashboard/Overview tab
// =============================================================================

export default function StatusBar() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-xl font-semibold">Dashboard</h2>
        <p className="text-sm text-muted-foreground">
          Overview of your GitID configuration
        </p>
      </div>

      <GuardPanel />
      <OverviewStats />
      <SuggestionsPanel />
      <StatusCheck />
      <RepoScanner />
    </div>
  );
}
