import { useState, useEffect } from "react";
import * as api from "@/lib/tauri-api";
import type {
  DetectionResult,
  SuggestedProfile,
  DetectedSshKey,
  ProfileDto,
} from "@/lib/tauri-api";
import { Card, CardContent, Button, Badge, Input } from "./ui/primitives";
import {
  Wand2,
  Key,
  User,
  Mail,
  Globe,
  Folder,
  Check,
  ChevronRight,
  Loader2,
  AlertCircle,
  Sparkles,
} from "lucide-react";

// =============================================================================
// Suggestion Card — one per detected identity
// =============================================================================

interface SuggestionCardProps {
  suggestion: SuggestedProfile;
  sshKeys: DetectedSshKey[];
  onImport: (
    name: string,
    profile: ProfileDto,
    dirPattern: string | null
  ) => Promise<void>;
}

function SuggestionCard({ suggestion, sshKeys, onImport }: SuggestionCardProps) {
  const [name, setName] = useState(suggestion.suggested_name);
  const [gitName, setGitName] = useState(suggestion.name);
  const [email, setEmail] = useState(suggestion.email);
  const [sshKey, setSshKey] = useState(suggestion.ssh_key ?? "");
  const [hosts, setHosts] = useState(suggestion.hosts.join(", "));
  const [dirPattern, setDirPattern] = useState(
    suggestion.directory_pattern ?? ""
  );
  const [importing, setImporting] = useState(false);
  const [imported, setImported] = useState(false);
  const [expanded, setExpanded] = useState(false);

  const handleImport = async () => {
    setImporting(true);
    try {
      const profile: ProfileDto = {
        name: gitName,
        email,
        ssh_key: sshKey || null,
        signing_key: null,
        signing_format: null,
        hosts: hosts
          .split(",")
          .map((h) => h.trim())
          .filter(Boolean),
        username: null,
      };
      await onImport(name, profile, dirPattern || null);
      setImported(true);
    } finally {
      setImporting(false);
    }
  };

  if (imported) {
    return (
      <Card className="border-emerald-500/30 bg-emerald-500/5">
        <CardContent className="p-4 flex items-center gap-3">
          <Check className="h-5 w-5 text-emerald-400" />
          <span className="font-medium">
            Profile "{name}" created
          </span>
          {dirPattern && (
            <Badge variant="outline" className="text-xs">
              + directory rule
            </Badge>
          )}
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="border-primary/20">
      <CardContent className="p-4 space-y-3">
        {/* Summary row */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="h-9 w-9 rounded-full bg-primary/10 flex items-center justify-center">
              <User className="h-4 w-4 text-primary" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <span className="font-semibold">{suggestion.suggested_name}</span>
                {suggestion.directory_pattern && (
                  <Badge variant="outline" className="text-xs">
                    <Folder className="h-3 w-3 mr-1" />
                    {suggestion.directory_pattern}
                  </Badge>
                )}
              </div>
              <div className="flex items-center gap-3 text-xs text-muted-foreground">
                {suggestion.name && (
                  <span className="flex items-center gap-1">
                    <User className="h-3 w-3" />
                    {suggestion.name}
                  </span>
                )}
                {suggestion.email && (
                  <span className="flex items-center gap-1">
                    <Mail className="h-3 w-3" />
                    {suggestion.email}
                  </span>
                )}
                {suggestion.ssh_key && (
                  <span className="flex items-center gap-1">
                    <Key className="h-3 w-3" />
                    {suggestion.ssh_key.split("/").pop()}
                  </span>
                )}
              </div>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setExpanded(!expanded)}
            >
              Edit
              <ChevronRight
                className={`h-3 w-3 ml-1 transition-transform ${
                  expanded ? "rotate-90" : ""
                }`}
              />
            </Button>
            <Button size="sm" onClick={handleImport} disabled={importing || !email}>
              {importing ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <>
                  <Check className="h-4 w-4 mr-1" />
                  Import
                </>
              )}
            </Button>
          </div>
        </div>

        {/* Expanded edit form */}
        {expanded && (
          <div className="grid grid-cols-2 gap-3 pt-2 border-t border-border">
            <div>
              <label className="text-xs text-muted-foreground">Profile ID</label>
              <Input
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="h-8 text-sm"
              />
            </div>
            <div>
              <label className="text-xs text-muted-foreground">Git Name</label>
              <Input
                value={gitName}
                onChange={(e) => setGitName(e.target.value)}
                className="h-8 text-sm"
              />
            </div>
            <div>
              <label className="text-xs text-muted-foreground">Email</label>
              <Input
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                className="h-8 text-sm"
              />
            </div>
            <div>
              <label className="text-xs text-muted-foreground">SSH Key</label>
              <select
                value={sshKey}
                onChange={(e) => setSshKey(e.target.value)}
                className="flex h-8 w-full rounded-md border border-input bg-background px-2 text-sm"
              >
                <option value="">None</option>
                {sshKeys.map((k) => (
                  <option key={k.path} value={k.path}>
                    {k.path.split("/").pop()} ({k.key_type})
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label className="text-xs text-muted-foreground">Hosts</label>
              <Input
                value={hosts}
                onChange={(e) => setHosts(e.target.value)}
                placeholder="github.com, gitlab.com"
                className="h-8 text-sm"
              />
            </div>
            <div>
              <label className="text-xs text-muted-foreground">
                Directory Rule
              </label>
              <Input
                value={dirPattern}
                onChange={(e) => setDirPattern(e.target.value)}
                placeholder="~/work/**"
                className="h-8 text-sm"
              />
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

// =============================================================================
// SetupWizard (main export)
// =============================================================================

interface SetupWizardProps {
  onComplete: () => void;
}

export default function SetupWizard({ onComplete }: SetupWizardProps) {
  const [detection, setDetection] = useState<DetectionResult | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api
      .detectSetup()
      .then(setDetection)
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center py-16 space-y-4">
        <Loader2 className="h-8 w-8 animate-spin text-primary" />
        <p className="text-muted-foreground">
          Scanning your Git configuration...
        </p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-center py-12">
        <AlertCircle className="h-8 w-8 text-destructive mx-auto mb-3" />
        <p className="text-destructive">{error}</p>
      </div>
    );
  }

  const hasSuggestions =
    detection && detection.suggested_profiles.length > 0;

  const handleImport = async (
    name: string,
    profile: api.ProfileDto,
    dirPattern: string | null
  ) => {
    await api.importSuggestedProfile(name, profile, dirPattern);
  };

  return (
    <div className="max-w-2xl mx-auto space-y-6 py-4">
      {/* Header */}
      <div className="text-center space-y-2">
        <div className="h-14 w-14 rounded-2xl bg-primary/10 flex items-center justify-center mx-auto">
          <Wand2 className="h-7 w-7 text-primary" />
        </div>
        <h2 className="text-2xl font-bold">Welcome to GitID</h2>
        <p className="text-muted-foreground">
          {hasSuggestions
            ? "We found your existing Git setup. Review and import your identities."
            : "Let's set up your first identity profile."}
        </p>
      </div>

      {/* What we found */}
      {detection && (
        <div className="space-y-3">
          {/* Detected info summary */}
          <div className="flex flex-wrap gap-2 justify-center">
            {detection.global_identity.name && (
              <Badge variant="outline">
                <User className="h-3 w-3 mr-1" />
                {detection.global_identity.name}
              </Badge>
            )}
            {detection.global_identity.email && (
              <Badge variant="outline">
                <Mail className="h-3 w-3 mr-1" />
                {detection.global_identity.email}
              </Badge>
            )}
            {detection.ssh_keys.length > 0 && (
              <Badge variant="outline">
                <Key className="h-3 w-3 mr-1" />
                {detection.ssh_keys.length} SSH key
                {detection.ssh_keys.length !== 1 ? "s" : ""}
              </Badge>
            )}
            {detection.conditional_identities.length > 0 && (
              <Badge variant="outline">
                <Folder className="h-3 w-3 mr-1" />
                {detection.conditional_identities.length} includeIf rule
                {detection.conditional_identities.length !== 1 ? "s" : ""}
              </Badge>
            )}
            {detection.credential_helper && (
              <Badge variant="outline">
                <Globe className="h-3 w-3 mr-1" />
                helper: {detection.credential_helper}
              </Badge>
            )}
          </div>

          {/* SSH keys found */}
          {detection.ssh_keys.length > 0 && (
            <Card>
              <CardContent className="p-4">
                <h3 className="text-sm font-medium mb-2 flex items-center gap-2">
                  <Key className="h-4 w-4" />
                  SSH Keys Found
                </h3>
                <div className="space-y-1.5">
                  {detection.ssh_keys.map((key) => (
                    <div
                      key={key.path}
                      className="flex items-center justify-between text-xs text-muted-foreground font-mono"
                    >
                      <span>{key.path}</span>
                      <div className="flex items-center gap-2">
                        <Badge variant="outline" className="text-xs font-mono">
                          {key.key_type}
                        </Badge>
                        {key.comment && (
                          <span className="text-muted-foreground/60 truncate max-w-48">
                            {key.comment}
                          </span>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      )}

      {/* Suggested profiles */}
      {hasSuggestions && (
        <div className="space-y-3">
          <h3 className="text-sm font-medium flex items-center gap-2">
            <Sparkles className="h-4 w-4 text-amber-400" />
            Suggested Profiles
          </h3>
          {detection!.suggested_profiles.map((suggestion, i) => (
            <SuggestionCard
              key={i}
              suggestion={suggestion}
              sshKeys={detection!.ssh_keys}
              onImport={handleImport}
            />
          ))}
        </div>
      )}

      {/* No suggestions — manual setup prompt */}
      {!hasSuggestions && (
        <Card className="p-6 text-center">
          <p className="text-muted-foreground mb-4">
            No existing Git identities detected. You can create profiles
            manually in the Profiles tab.
          </p>
        </Card>
      )}

      {/* Done button */}
      <div className="flex justify-center pt-4">
        <Button onClick={onComplete} size="lg">
          {hasSuggestions ? "Continue to Dashboard" : "Set Up Manually"}
          <ChevronRight className="h-4 w-4 ml-2" />
        </Button>
      </div>
    </div>
  );
}
