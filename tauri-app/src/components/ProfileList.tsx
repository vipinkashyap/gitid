import { useState } from "react";
import { useProfiles } from "@/lib/hooks";
import * as api from "@/lib/tauri-api";
import type { ProfileDto } from "@/lib/tauri-api";
import { Card, CardContent, Button, Badge, Dialog, Input } from "./ui/primitives";
import {
  User,
  Mail,
  Key,
  Globe,
  Plus,
  Pencil,
  Trash2,
  Shield,
  Wifi,
  Loader2,
} from "lucide-react";

// =============================================================================
// Profile Form (shared between create and edit)
// =============================================================================

interface ProfileFormProps {
  initial?: ProfileDto;
  onSave: (name: string, profile: ProfileDto) => Promise<void>;
  onCancel: () => void;
  isNew: boolean;
}

function ProfileForm({ initial, onSave, onCancel, isNew }: ProfileFormProps) {
  const [profileName, setProfileName] = useState(isNew ? "" : "");
  const [name, setName] = useState(initial?.name ?? "");
  const [email, setEmail] = useState(initial?.email ?? "");
  const [sshKey, setSshKey] = useState(initial?.ssh_key ?? "");
  const [username, setUsername] = useState(initial?.username ?? "");
  const [hosts, setHosts] = useState(initial?.hosts.join(", ") ?? "");
  const [signingKey, setSigningKey] = useState(initial?.signing_key ?? "");
  const [signingFormat, setSigningFormat] = useState(initial?.signing_format ?? "");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    setError(null);
    try {
      const profile: ProfileDto = {
        name,
        email,
        ssh_key: sshKey || null,
        signing_key: signingKey || null,
        signing_format: signingFormat || null,
        hosts: hosts
          .split(",")
          .map((h) => h.trim())
          .filter(Boolean),
        username: username || null,
      };
      await onSave(profileName, profile);
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <h2 className="text-lg font-semibold">
        {isNew ? "New Profile" : "Edit Profile"}
      </h2>

      {isNew && (
        <div>
          <label className="text-sm font-medium text-muted-foreground">
            Profile ID
          </label>
          <Input
            value={profileName}
            onChange={(e) => setProfileName(e.target.value)}
            placeholder="e.g., personal, work, oss"
            required
          />
        </div>
      )}

      <div className="grid grid-cols-2 gap-4">
        <div>
          <label className="text-sm font-medium text-muted-foreground">
            Git Name
          </label>
          <Input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Vipin Sharma"
            required
          />
        </div>
        <div>
          <label className="text-sm font-medium text-muted-foreground">
            Git Email
          </label>
          <Input
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder="vipin@example.com"
            type="email"
            required
          />
        </div>
      </div>

      <div>
        <label className="text-sm font-medium text-muted-foreground">
          SSH Private Key Path
        </label>
        <Input
          value={sshKey}
          onChange={(e) => setSshKey(e.target.value)}
          placeholder="~/.ssh/id_ed25519_personal"
        />
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div>
          <label className="text-sm font-medium text-muted-foreground">
            HTTPS Username
          </label>
          <Input
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            placeholder="GitHub username"
          />
        </div>
        <div>
          <label className="text-sm font-medium text-muted-foreground">
            Associated Hosts
          </label>
          <Input
            value={hosts}
            onChange={(e) => setHosts(e.target.value)}
            placeholder="github.com, gitlab.com"
          />
        </div>
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div>
          <label className="text-sm font-medium text-muted-foreground">
            Signing Key
          </label>
          <Input
            value={signingKey}
            onChange={(e) => setSigningKey(e.target.value)}
            placeholder="~/.ssh/id_ed25519 or GPG key ID"
          />
        </div>
        <div>
          <label className="text-sm font-medium text-muted-foreground">
            Signing Format
          </label>
          <Input
            value={signingFormat}
            onChange={(e) => setSigningFormat(e.target.value)}
            placeholder="ssh, gpg, or x509"
          />
        </div>
      </div>

      {error && (
        <p className="text-sm text-destructive">{error}</p>
      )}

      <div className="flex justify-end gap-2 pt-2">
        <Button type="button" variant="ghost" onClick={onCancel}>
          Cancel
        </Button>
        <Button type="submit" disabled={saving}>
          {saving && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
          {isNew ? "Create" : "Save"}
        </Button>
      </div>
    </form>
  );
}

// =============================================================================
// Profile Card
// =============================================================================

interface ProfileCardProps {
  id: string;
  profile: ProfileDto;
  onEdit: () => void;
  onDelete: () => void;
}

function ProfileCard({ id, profile, onEdit, onDelete }: ProfileCardProps) {
  const [testing, setTesting] = useState(false);
  const [testResults, setTestResults] = useState<[string, boolean][] | null>(null);

  const handleTestSsh = async () => {
    setTesting(true);
    try {
      const results = await api.testSshConnection(id);
      setTestResults(results);
    } catch {
      setTestResults(null);
    } finally {
      setTesting(false);
    }
  };

  return (
    <Card className="group">
      <CardContent className="p-5">
        <div className="flex items-start justify-between">
          <div className="space-y-3 flex-1">
            <div className="flex items-center gap-2">
              <h3 className="text-lg font-semibold">{id}</h3>
              {profile.signing_key && (
                <Badge variant="success">
                  <Shield className="h-3 w-3 mr-1" />
                  signed
                </Badge>
              )}
            </div>

            <div className="space-y-1.5 text-sm">
              <div className="flex items-center gap-2 text-muted-foreground">
                <User className="h-3.5 w-3.5" />
                <span>{profile.name}</span>
              </div>
              <div className="flex items-center gap-2 text-muted-foreground">
                <Mail className="h-3.5 w-3.5" />
                <span>{profile.email}</span>
              </div>
              {profile.ssh_key && (
                <div className="flex items-center gap-2 text-muted-foreground">
                  <Key className="h-3.5 w-3.5" />
                  <span className="font-mono text-xs">{profile.ssh_key}</span>
                </div>
              )}
              {profile.hosts.length > 0 && (
                <div className="flex items-center gap-2 text-muted-foreground">
                  <Globe className="h-3.5 w-3.5" />
                  <div className="flex gap-1.5">
                    {profile.hosts.map((host) => (
                      <Badge key={host} variant="outline" className="text-xs">
                        {host}
                      </Badge>
                    ))}
                  </div>
                </div>
              )}
            </div>

            {testResults && (
              <div className="flex gap-2 pt-1">
                {testResults.map(([host, ok]) => (
                  <Badge key={host} variant={ok ? "success" : "destructive"}>
                    {host}: {ok ? "connected" : "failed"}
                  </Badge>
                ))}
              </div>
            )}
          </div>

          <div className="flex gap-1 opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 transition-opacity">
            {profile.ssh_key && profile.hosts.length > 0 && (
              <Button
                variant="ghost"
                size="icon"
                onClick={handleTestSsh}
                aria-label={`Test SSH connections for ${id}`}
                disabled={testing}
              >
                {testing ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <Wifi className="h-4 w-4" />
                )}
              </Button>
            )}
            <Button variant="ghost" size="icon" onClick={onEdit} aria-label={`Edit profile ${id}`}>
              <Pencil className="h-4 w-4" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              onClick={onDelete}
              aria-label={`Delete profile ${id}`}
              className="text-destructive hover:text-destructive"
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

// =============================================================================
// ProfileList (main export)
// =============================================================================

export default function ProfileList() {
  const { data: profiles, loading, error, refresh } = useProfiles();
  const [showForm, setShowForm] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [deleteConfirm, setDeleteConfirm] = useState<string | null>(null);

  const handleCreate = async (name: string, profile: ProfileDto) => {
    await api.createProfile(name, profile);
    setShowForm(false);
    refresh();
  };

  const handleUpdate = async (_name: string, profile: ProfileDto) => {
    if (!editingId) return;
    await api.updateProfile(editingId, profile);
    setEditingId(null);
    refresh();
  };

  const handleDelete = async (name: string) => {
    await api.deleteProfile(name);
    setDeleteConfirm(null);
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
        <p>Failed to load profiles: {error}</p>
        <Button variant="outline" className="mt-4" onClick={refresh}>
          Retry
        </Button>
      </div>
    );
  }

  const entries = profiles ? Object.entries(profiles) : [];

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold">Profiles</h2>
          <p className="text-sm text-muted-foreground">
            Manage your Git identity profiles
          </p>
        </div>
        <Button onClick={() => setShowForm(true)}>
          <Plus className="h-4 w-4 mr-2" />
          New Profile
        </Button>
      </div>

      {entries.length === 0 && !showForm && (
        <Card className="p-8 text-center">
          <p className="text-muted-foreground mb-4">
            No profiles yet. Create your first identity profile.
          </p>
          <Button onClick={() => setShowForm(true)}>
            <Plus className="h-4 w-4 mr-2" />
            Create Profile
          </Button>
        </Card>
      )}

      <div className="grid gap-3">
        {entries.map(([id, profile]) => (
          <ProfileCard
            key={id}
            id={id}
            profile={profile}
            onEdit={() => setEditingId(id)}
            onDelete={() => setDeleteConfirm(id)}
          />
        ))}
      </div>

      {/* New Profile Dialog */}
      <Dialog open={showForm} onClose={() => setShowForm(false)}>
        <ProfileForm
          isNew
          onSave={handleCreate}
          onCancel={() => setShowForm(false)}
        />
      </Dialog>

      {/* Edit Profile Dialog */}
      <Dialog open={!!editingId} onClose={() => setEditingId(null)}>
        {editingId && profiles?.[editingId] && (
          <ProfileForm
            initial={profiles[editingId]}
            isNew={false}
            onSave={handleUpdate}
            onCancel={() => setEditingId(null)}
          />
        )}
      </Dialog>

      {/* Delete Confirmation */}
      <Dialog open={!!deleteConfirm} onClose={() => setDeleteConfirm(null)}>
        <div className="space-y-4">
          <h2 className="text-lg font-semibold">Delete Profile</h2>
          <p className="text-muted-foreground">
            Are you sure you want to delete{" "}
            <span className="font-semibold text-foreground">{deleteConfirm}</span>?
            This cannot be undone.
          </p>
          <div className="flex justify-end gap-2">
            <Button variant="ghost" onClick={() => setDeleteConfirm(null)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={() => deleteConfirm && handleDelete(deleteConfirm)}
            >
              Delete
            </Button>
          </div>
        </div>
      </Dialog>
    </div>
  );
}
