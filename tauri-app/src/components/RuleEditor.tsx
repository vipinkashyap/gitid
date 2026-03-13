import { useState } from "react";
import { useRules, useProfiles } from "@/lib/hooks";
import * as api from "@/lib/tauri-api";
import type { RuleDto } from "@/lib/tauri-api";
import {
  Card,
  CardContent,
  Button,
  Badge,
  Dialog,
  Input,
  Select,
  HelpTip,
} from "./ui/primitives";
import {
  Folder,
  Globe,
  Link,
  Plus,
  Trash2,
  ArrowUp,
  ArrowDown,
  Loader2,
} from "lucide-react";

// =============================================================================
// Rule type icons and colors
// =============================================================================

const ruleTypeConfig: Record<
  string,
  { icon: React.ReactNode; label: string; color: string }
> = {
  directory: {
    icon: <Folder className="h-4 w-4" />,
    label: "Directory",
    color: "text-blue-400",
  },
  remote: {
    icon: <Link className="h-4 w-4" />,
    label: "Remote URL",
    color: "text-purple-400",
  },
  host: {
    icon: <Globe className="h-4 w-4" />,
    label: "Host",
    color: "text-amber-400",
  },
};

// =============================================================================
// Add Rule Form
// =============================================================================

interface AddRuleFormProps {
  profiles: string[];
  onSave: (ruleType: string, pattern: string, profile: string) => Promise<void>;
  onCancel: () => void;
}

function AddRuleForm({ profiles, onSave, onCancel }: AddRuleFormProps) {
  const [ruleType, setRuleType] = useState("directory");
  const [pattern, setPattern] = useState("");
  const [profile, setProfile] = useState(profiles[0] ?? "");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const placeholders: Record<string, string> = {
    directory: "~/work/**",
    remote: "*github.com/my-company/*",
    host: "github.com",
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    setError(null);
    try {
      await onSave(ruleType, pattern, profile);
    } catch (e) {
      setError(String(e));
      setSaving(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <h2 className="text-lg font-semibold">Add Rule</h2>

      <div>
        <label className="text-sm font-medium text-muted-foreground">
          Rule Type
        </label>
        <Select value={ruleType} onChange={(e) => setRuleType(e.target.value)}>
          <option value="directory">Directory (highest priority)</option>
          <option value="remote">Remote URL Pattern</option>
          <option value="host">Host Default (lowest priority)</option>
        </Select>
      </div>

      <div>
        <label className="text-sm font-medium text-muted-foreground">
          Pattern
        </label>
        <Input
          value={pattern}
          onChange={(e) => setPattern(e.target.value)}
          placeholder={placeholders[ruleType]}
          required
        />
        <p className="text-xs text-muted-foreground mt-1">
          {ruleType === "directory" &&
            "Glob pattern matching against the repo's directory path"}
          {ruleType === "remote" &&
            "Glob pattern matching against the remote URL"}
          {ruleType === "host" && "Exact hostname match"}
        </p>
      </div>

      <div>
        <label className="text-sm font-medium text-muted-foreground">
          Profile
        </label>
        <Select value={profile} onChange={(e) => setProfile(e.target.value)}>
          {profiles.map((p) => (
            <option key={p} value={p}>
              {p}
            </option>
          ))}
        </Select>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      <div className="flex justify-end gap-2 pt-2">
        <Button type="button" variant="ghost" onClick={onCancel}>
          Cancel
        </Button>
        <Button type="submit" disabled={saving}>
          {saving && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
          Add Rule
        </Button>
      </div>
    </form>
  );
}

// =============================================================================
// Rule Row
// =============================================================================

interface RuleRowProps {
  rule: RuleDto;
  index: number;
  total: number;
  onMoveUp: () => void;
  onMoveDown: () => void;
  onDelete: () => void;
}

function RuleRow({ rule, index, total, onMoveUp, onMoveDown, onDelete }: RuleRowProps) {
  const config = ruleTypeConfig[rule.rule_type] ?? ruleTypeConfig.host;

  return (
    <div className="flex items-center gap-3 p-3 rounded-lg border bg-card group hover:bg-accent/50 transition-colors">
      <Badge variant="outline" className="text-xs shrink-0">
        {index + 1}
      </Badge>

      <div className={`shrink-0 ${config.color}`}>{config.icon}</div>

      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="font-mono text-sm truncate">{rule.pattern}</span>
          <span className="text-muted-foreground text-xs">→</span>
          <Badge variant="secondary">{rule.profile}</Badge>
        </div>
        <p className="text-xs text-muted-foreground">{config.label} rule</p>
      </div>

      <div className="flex gap-1 opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 transition-opacity">
        <Button
          variant="ghost"
          size="icon"
          onClick={onMoveUp}
          disabled={index === 0}
          aria-label="Increase priority"
          className="h-8 w-8"
        >
          <ArrowUp className="h-3.5 w-3.5" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          onClick={onMoveDown}
          disabled={index === total - 1}
          aria-label="Decrease priority"
          className="h-8 w-8"
        >
          <ArrowDown className="h-3.5 w-3.5" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          onClick={onDelete}
          aria-label={`Delete ${config.label.toLowerCase()} rule ${rule.pattern}`}
          className="h-8 w-8 text-destructive hover:text-destructive"
        >
          <Trash2 className="h-3.5 w-3.5" />
        </Button>
      </div>
    </div>
  );
}

// =============================================================================
// RuleEditor (main export)
// =============================================================================

export default function RuleEditor() {
  const { data: rulesData, loading, error, refresh } = useRules();
  const { data: profiles } = useProfiles();
  const [showForm, setShowForm] = useState(false);
  const [defaultProfile, setDefaultProfile] = useState<string | null>(null);
  const [deleteConfirm, setDeleteConfirm] = useState<RuleDto | null>(null);

  const profileNames = profiles ? Object.keys(profiles) : [];

  const handleAddRule = async (
    ruleType: string,
    pattern: string,
    profile: string
  ) => {
    await api.addRule(ruleType, pattern, profile);
    setShowForm(false);
    refresh();
  };

  const handleRemoveRule = async (rule: RuleDto) => {
    // Calculate the index within its type
    const sameTypeRules =
      rulesData?.rules.filter((r) => r.rule_type === rule.rule_type) ?? [];
    const typeIndex = sameTypeRules.findIndex((r) => r.id === rule.id);
    if (typeIndex >= 0) {
      await api.removeRule(rule.rule_type, typeIndex);
      setDeleteConfirm(null);
      refresh();
    }
  };

  const handleMoveRule = async (
    rule: RuleDto,
    direction: "up" | "down"
  ) => {
    const sameTypeRules =
      rulesData?.rules.filter((r) => r.rule_type === rule.rule_type) ?? [];
    const typeIndex = sameTypeRules.findIndex((r) => r.id === rule.id);
    if (typeIndex < 0) return;

    const newIndex =
      direction === "up" ? typeIndex - 1 : typeIndex + 1;
    if (newIndex < 0 || newIndex >= sameTypeRules.length) return;

    const order = sameTypeRules.map((_, i) => i);
    [order[typeIndex], order[newIndex]] = [order[newIndex], order[typeIndex]];
    await api.reorderRules(rule.rule_type, order);
    refresh();
  };

  const handleSetDefault = async (profile: string) => {
    await api.setDefaultProfile(profile);
    setDefaultProfile(null);
    refresh();
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-center py-12 text-destructive">
        <p>Failed to load rules: {error}</p>
        <Button variant="outline" className="mt-4" onClick={refresh}>
          Retry
        </Button>
      </div>
    );
  }

  const rules = rulesData?.rules ?? [];

  // Group by type for display
  const dirRules = rules.filter((r) => r.rule_type === "directory");
  const remoteRules = rules.filter((r) => r.rule_type === "remote");
  const hostRules = rules.filter((r) => r.rule_type === "host");

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold flex items-center gap-2">
            Rules
            <HelpTip text="Rules map repositories to profiles. Directory rules match file paths, remote rules match Git URLs, and hostname rules match Git hosts." />
          </h2>
          <p className="text-sm text-muted-foreground">
            Configure how GitID selects profiles. Higher rules take priority.
          </p>
        </div>
        <Button onClick={() => setShowForm(true)}>
          <Plus className="h-4 w-4 mr-2" />
          Add Rule
        </Button>
      </div>

      {/* Priority explanation */}
      <div className="flex items-center gap-4 text-xs text-muted-foreground">
        <span>Resolution priority:</span>
        <div className="flex items-center gap-1">
          <Badge variant="outline" className="text-xs">
            1
          </Badge>
          <span>Repo override</span>
        </div>
        <span>→</span>
        <div className="flex items-center gap-1 text-blue-400">
          <Badge variant="outline" className="text-xs">
            2
          </Badge>
          <span>Directory</span>
        </div>
        <span>→</span>
        <div className="flex items-center gap-1 text-purple-400">
          <Badge variant="outline" className="text-xs">
            3
          </Badge>
          <span>Remote URL</span>
        </div>
        <span>→</span>
        <div className="flex items-center gap-1 text-amber-400">
          <Badge variant="outline" className="text-xs">
            4
          </Badge>
          <span>Host</span>
        </div>
        <span>→</span>
        <div className="flex items-center gap-1">
          <Badge variant="outline" className="text-xs">
            5
          </Badge>
          <span>Default</span>
        </div>
      </div>

      {rules.length === 0 && !showForm && (
        <Card className="p-8 text-center">
          <p className="text-muted-foreground mb-4">
            No rules configured. Add rules to automatically select profiles.
          </p>
          <Button onClick={() => setShowForm(true)}>
            <Plus className="h-4 w-4 mr-2" />
            Add First Rule
          </Button>
        </Card>
      )}

      {/* Directory Rules */}
      {dirRules.length > 0 && (
        <div className="space-y-2">
          <h3 className="text-sm font-medium text-blue-400 flex items-center gap-2">
            <Folder className="h-4 w-4" />
            Directory Rules
          </h3>
          {dirRules.map((rule, i) => (
            <RuleRow
              key={rule.id}
              rule={rule}
              index={i}
              total={dirRules.length}
              onMoveUp={() => handleMoveRule(rule, "up")}
              onMoveDown={() => handleMoveRule(rule, "down")}
              onDelete={() => setDeleteConfirm(rule)}
            />
          ))}
        </div>
      )}

      {/* Remote Rules */}
      {remoteRules.length > 0 && (
        <div className="space-y-2">
          <h3 className="text-sm font-medium text-purple-400 flex items-center gap-2">
            <Link className="h-4 w-4" />
            Remote URL Rules
          </h3>
          {remoteRules.map((rule, i) => (
            <RuleRow
              key={rule.id}
              rule={rule}
              index={i}
              total={remoteRules.length}
              onMoveUp={() => handleMoveRule(rule, "up")}
              onMoveDown={() => handleMoveRule(rule, "down")}
              onDelete={() => setDeleteConfirm(rule)}
            />
          ))}
        </div>
      )}

      {/* Host Rules */}
      {hostRules.length > 0 && (
        <div className="space-y-2">
          <h3 className="text-sm font-medium text-amber-400 flex items-center gap-2">
            <Globe className="h-4 w-4" />
            Host Rules
          </h3>
          {hostRules.map((rule, i) => (
            <RuleRow
              key={rule.id}
              rule={rule}
              index={i}
              total={hostRules.length}
              onMoveUp={() => handleMoveRule(rule, "up")}
              onMoveDown={() => handleMoveRule(rule, "down")}
              onDelete={() => setDeleteConfirm(rule)}
            />
          ))}
        </div>
      )}

      {/* Global Default */}
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-sm font-medium">Global Default</h3>
              <p className="text-xs text-muted-foreground">
                Used when no other rule matches
              </p>
            </div>
            <div className="flex items-center gap-2">
              {rulesData?.default ? (
                <Badge variant="secondary">{rulesData.default}</Badge>
              ) : (
                <span className="text-sm text-muted-foreground">None set</span>
              )}
              <Button
                variant="outline"
                size="sm"
                onClick={() => setDefaultProfile(rulesData?.default ?? "")}
              >
                Change
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Add Rule Dialog */}
      <Dialog open={showForm} onClose={() => setShowForm(false)}>
        <AddRuleForm
          profiles={profileNames}
          onSave={handleAddRule}
          onCancel={() => setShowForm(false)}
        />
      </Dialog>

      {/* Delete Rule Confirmation */}
      <Dialog
        open={!!deleteConfirm}
        onClose={() => setDeleteConfirm(null)}
      >
        {deleteConfirm && (
          <div className="space-y-4">
            <h2 className="text-lg font-semibold">Delete Rule</h2>
            <p className="text-muted-foreground">
              Are you sure you want to delete the{" "}
              <span className="font-semibold text-foreground">
                {ruleTypeConfig[deleteConfirm.rule_type]?.label ?? ""}
              </span>{" "}
              rule{" "}
              <span className="font-mono text-foreground">
                {deleteConfirm.pattern}
              </span>
              ? This cannot be undone.
            </p>
            <div className="flex justify-end gap-2">
              <Button variant="ghost" onClick={() => setDeleteConfirm(null)}>
                Cancel
              </Button>
              <Button
                variant="destructive"
                onClick={() => handleRemoveRule(deleteConfirm)}
              >
                Delete
              </Button>
            </div>
          </div>
        )}
      </Dialog>

      {/* Set Default Dialog */}
      <Dialog
        open={defaultProfile !== null}
        onClose={() => setDefaultProfile(null)}
      >
        <div className="space-y-4">
          <h2 className="text-lg font-semibold">Set Default Profile</h2>
          <Select
            value={defaultProfile ?? ""}
            onChange={(e) => setDefaultProfile(e.target.value)}
          >
            <option value="" disabled>
              Select a profile...
            </option>
            {profileNames.map((p) => (
              <option key={p} value={p}>
                {p}
              </option>
            ))}
          </Select>
          <div className="flex justify-end gap-2">
            <Button variant="ghost" onClick={() => setDefaultProfile(null)}>
              Cancel
            </Button>
            <Button
              onClick={() => defaultProfile && handleSetDefault(defaultProfile)}
              disabled={!defaultProfile}
            >
              Set Default
            </Button>
          </div>
        </div>
      </Dialog>
    </div>
  );
}
