/**
 * Typed TypeScript bindings for GitID Tauri IPC commands.
 * Each function maps 1:1 to a #[tauri::command] in commands.rs.
 */
import { invoke } from "@tauri-apps/api/core";

// =============================================================================
// Types (mirrors Rust DTOs)
// =============================================================================

export interface ProfileDto {
  name: string;
  email: string;
  ssh_key: string | null;
  signing_key: string | null;
  signing_format: string | null;
  hosts: string[];
  username: string | null;
}

export interface RuleDto {
  id: number;
  rule_type: "directory" | "remote" | "host";
  pattern: string;
  profile: string;
}

export interface RulesDto {
  rules: RuleDto[];
  default: string | null;
}

export interface StatusDto {
  directory: string;
  profile_name: string | null;
  reason: string | null;
  profile: ProfileDto | null;
  remote_url: string | null;
}

export interface DoctorCheck {
  name: string;
  status: "ok" | "warning" | "error";
  message: string;
  fix: string | null;
}

export interface DetectedRepo {
  path: string;
  name: string;
  remote_url: string | null;
  current_profile: string | null;
  current_email: string | null;
}

// =============================================================================
// Profile API
// =============================================================================

export async function getProfiles(): Promise<Record<string, ProfileDto>> {
  return invoke("get_profiles");
}

export async function getProfile(name: string): Promise<ProfileDto> {
  return invoke("get_profile", { name });
}

export async function createProfile(
  name: string,
  profile: ProfileDto
): Promise<void> {
  return invoke("create_profile", { name, profile });
}

export async function updateProfile(
  name: string,
  profile: ProfileDto
): Promise<void> {
  return invoke("update_profile", { name, profile });
}

export async function deleteProfile(name: string): Promise<void> {
  return invoke("delete_profile", { name });
}

// =============================================================================
// Rules API
// =============================================================================

export async function getRules(): Promise<RulesDto> {
  return invoke("get_rules");
}

export async function addRule(
  ruleType: string,
  pattern: string,
  profile: string
): Promise<void> {
  return invoke("add_rule", {
    ruleType,
    pattern,
    profile,
  });
}

export async function removeRule(
  ruleType: string,
  index: number
): Promise<void> {
  return invoke("remove_rule", { ruleType, index });
}

export async function setDefaultProfile(profile: string): Promise<void> {
  return invoke("set_default_profile", { profile });
}

export async function reorderRules(
  ruleType: string,
  newOrder: number[]
): Promise<void> {
  return invoke("reorder_rules", { ruleType, newOrder });
}

// =============================================================================
// Status API
// =============================================================================

export async function getStatus(path?: string): Promise<StatusDto> {
  return invoke("get_status", { path: path ?? null });
}

// =============================================================================
// Doctor API
// =============================================================================

export async function runDoctor(): Promise<DoctorCheck[]> {
  return invoke("run_doctor");
}

export async function installCredentialHelper(): Promise<void> {
  return invoke("install_credential_helper");
}

// =============================================================================
// Repo scanning API
// =============================================================================

export async function scanRepos(directory: string): Promise<DetectedRepo[]> {
  return invoke("scan_repos", { directory });
}

// =============================================================================
// SSH API
// =============================================================================

export async function testSshConnection(
  profileName: string
): Promise<[string, boolean][]> {
  return invoke("test_ssh_connection", { profileName });
}

// =============================================================================
// Detection / Import API
// =============================================================================

export interface DetectedSshKey {
  path: string;
  pub_path: string;
  key_type: string;
  comment: string;
  fingerprint: string | null;
}

export interface DetectedIdentity {
  source: string;
  name: string | null;
  email: string | null;
  signing_key: string | null;
}

export interface SuggestedProfile {
  suggested_name: string;
  name: string;
  email: string;
  ssh_key: string | null;
  hosts: string[];
  directory_pattern: string | null;
}

export interface DetectionResult {
  global_identity: DetectedIdentity;
  conditional_identities: DetectedIdentity[];
  ssh_keys: DetectedSshKey[];
  credential_helper: string | null;
  suggested_profiles: SuggestedProfile[];
}

export async function detectSetup(): Promise<DetectionResult> {
  return invoke("detect_setup");
}

export async function importSuggestedProfile(
  name: string,
  profile: ProfileDto,
  directoryPattern: string | null
): Promise<void> {
  return invoke("import_suggested_profile", {
    name,
    profile,
    directoryPattern,
  });
}

// =============================================================================
// Guard API
// =============================================================================

export interface GuardStatusDto {
  installed: boolean;
  verdict: "ok" | "mismatch" | "no_profile" | "not_a_repo";
  profile: string | null;
  expected_email: string | null;
  actual_email: string | null;
}

export async function getGuardStatus(): Promise<GuardStatusDto> {
  return invoke("guard_status");
}

export async function guardInstall(): Promise<void> {
  return invoke("guard_install");
}

export async function guardUninstall(): Promise<void> {
  return invoke("guard_uninstall");
}

export async function guardFix(): Promise<void> {
  return invoke("guard_fix");
}

// =============================================================================
// Suggestions / Learning API
// =============================================================================

export interface SuggestionDto {
  index: number;
  rule_type: string;
  profile: string;
  pattern: string;
  evidence_count: number;
  reason: string;
}

export async function getSuggestions(
  minEvidence?: number
): Promise<SuggestionDto[]> {
  return invoke("get_suggestions", { minEvidence: minEvidence ?? null });
}

export async function getActivityCount(): Promise<number> {
  return invoke("get_activity_count");
}

export async function applySuggestion(
  ruleType: string,
  pattern: string,
  profile: string
): Promise<void> {
  return invoke("apply_suggestion", { ruleType, pattern, profile });
}

// =============================================================================
// CLI Installation API
// =============================================================================

export interface CliStatusDto {
  installed: boolean;
  path: string | null;
  version: string | null;
}

export async function checkCliInstalled(): Promise<CliStatusDto> {
  return invoke("check_cli_installed");
}

export async function installCli(): Promise<string> {
  return invoke("install_cli");
}
