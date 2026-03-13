import { useState } from "react";
import { useSuggestions, useActivityCount } from "@/lib/hooks";
import * as api from "@/lib/tauri-api";
import type { SuggestionDto } from "@/lib/tauri-api";
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
  Lightbulb,
  Folder,
  Globe,
  Link,
  Check,
  X,
  Loader2,
  Activity,
} from "lucide-react";

// =============================================================================
// Rule type visual config (matches RuleEditor)
// =============================================================================

const ruleTypeConfig: Record<
  string,
  { icon: React.ReactNode; label: string; color: string }
> = {
  directory: {
    icon: <Folder className="h-3.5 w-3.5" />,
    label: "Directory",
    color: "text-blue-400",
  },
  remote: {
    icon: <Link className="h-3.5 w-3.5" />,
    label: "Remote URL",
    color: "text-purple-400",
  },
  host: {
    icon: <Globe className="h-3.5 w-3.5" />,
    label: "Host",
    color: "text-amber-400",
  },
  default: {
    icon: <Globe className="h-3.5 w-3.5" />,
    label: "Default",
    color: "text-muted-foreground",
  },
};

// =============================================================================
// Suggestion Row
// =============================================================================

function SuggestionRow({
  suggestion,
  onApply,
  onDismiss,
}: {
  suggestion: SuggestionDto;
  onApply: () => Promise<void>;
  onDismiss: () => void;
}) {
  const [applying, setApplying] = useState(false);
  const [applied, setApplied] = useState(false);
  const [dismissed, setDismissed] = useState(false);

  const config = ruleTypeConfig[suggestion.rule_type] ?? ruleTypeConfig.host;

  const handleApply = async () => {
    setApplying(true);
    try {
      await onApply();
      setApplied(true);
    } finally {
      setApplying(false);
    }
  };

  if (dismissed) return null;

  if (applied) {
    return (
      <div className="flex items-center gap-3 p-3 rounded-lg border border-emerald-500/20 bg-emerald-500/5">
        <Check className="h-4 w-4 text-emerald-400" />
        <span className="text-sm text-emerald-400">
          Rule applied: <span className="font-mono">{suggestion.pattern}</span>{" "}
          → {suggestion.profile}
        </span>
      </div>
    );
  }

  return (
    <div className="flex items-start gap-3 p-3 rounded-lg border group hover:bg-accent/50 transition-colors">
      <div className={`shrink-0 mt-0.5 ${config.color}`}>{config.icon}</div>
      <div className="flex-1 min-w-0 space-y-1">
        <div className="flex items-center gap-2">
          <span className="font-mono text-sm truncate">
            {suggestion.pattern}
          </span>
          <span className="text-muted-foreground text-xs">→</span>
          <Badge variant="secondary">{suggestion.profile}</Badge>
        </div>
        <p className="text-xs text-muted-foreground">{suggestion.reason}</p>
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <Badge variant="outline" className="text-xs">
            {config.label}
          </Badge>
          <span>
            {suggestion.evidence_count} event
            {suggestion.evidence_count !== 1 ? "s" : ""}
          </span>
        </div>
      </div>
      <div className="flex gap-1 opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 transition-opacity shrink-0">
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8"
          onClick={handleApply}
          disabled={applying}
          aria-label={`Apply suggestion: ${suggestion.pattern} to ${suggestion.profile}`}
        >
          {applying ? (
            <Loader2 className="h-3.5 w-3.5 animate-spin" />
          ) : (
            <Check className="h-3.5 w-3.5 text-emerald-400" />
          )}
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8"
          onClick={() => {
            setDismissed(true);
            onDismiss();
          }}
          aria-label={`Dismiss suggestion: ${suggestion.pattern}`}
        >
          <X className="h-3.5 w-3.5" />
        </Button>
      </div>
    </div>
  );
}

// =============================================================================
// SuggestionsPanel (main export)
// =============================================================================

export default function SuggestionsPanel() {
  const { data: suggestions, loading, refresh } = useSuggestions(2);
  const { data: activityCount } = useActivityCount();
  const [dismissed, setDismissed] = useState<Set<number>>(new Set());

  const handleApply = async (s: SuggestionDto) => {
    await api.applySuggestion(s.rule_type, s.pattern, s.profile);
    refresh();
  };

  const handleDismiss = (index: number) => {
    setDismissed((prev) => new Set(prev).add(index));
  };

  const visible =
    suggestions?.filter((s) => !dismissed.has(s.index)) ?? [];

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="text-base flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Lightbulb className="h-4 w-4 text-amber-400" />
            Smart Suggestions
            <HelpTip text="GitID learns from your activity and suggests new rules when it detects patterns." />
          </div>
          <div className="flex items-center gap-2">
            {activityCount !== null && activityCount > 0 && (
              <Badge variant="outline" className="text-xs">
                <Activity className="h-3 w-3 mr-1" />
                {activityCount} event{activityCount !== 1 ? "s" : ""} logged
              </Badge>
            )}
          </div>
        </CardTitle>
        <p className="text-xs text-muted-foreground">
          Rule suggestions based on your Git usage patterns. Accept to create
          automatic rules.
        </p>
      </CardHeader>
      <CardContent className="space-y-2">
        {loading && (
          <div className="flex items-center justify-center py-4">
            <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
          </div>
        )}

        {!loading && visible.length === 0 && (
          <div className="text-center py-4">
            <p className="text-sm text-muted-foreground">
              {activityCount === 0 || activityCount === null
                ? "No activity recorded yet. Use GitID for a while and suggestions will appear here."
                : "No new suggestions. Keep using GitID and check back later."}
            </p>
          </div>
        )}

        {!loading &&
          visible.map((s) => (
            <SuggestionRow
              key={`${s.rule_type}-${s.pattern}-${s.profile}`}
              suggestion={s}
              onApply={() => handleApply(s)}
              onDismiss={() => handleDismiss(s.index)}
            />
          ))}
      </CardContent>
    </Card>
  );
}
