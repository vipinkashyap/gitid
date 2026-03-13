/**
 * React hooks for GitID data fetching and state management.
 */
import { useState, useEffect, useCallback } from "react";
import * as api from "./tauri-api";

/** Generic async data hook with loading/error states. */
function useAsync<T>(
  fetcher: () => Promise<T>,
  deps: unknown[] = []
): { data: T | null; loading: boolean; error: string | null; refresh: () => void } {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(() => {
    setLoading(true);
    setError(null);
    fetcher()
      .then(setData)
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, deps);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { data, loading, error, refresh };
}

export function useProfiles() {
  return useAsync(() => api.getProfiles(), []);
}

export function useRules() {
  return useAsync(() => api.getRules(), []);
}

export function useStatus(path?: string) {
  return useAsync(() => api.getStatus(path), [path]);
}

export function useDoctor() {
  return useAsync(() => api.runDoctor(), []);
}

export function useRepoScan(directory: string | null) {
  return useAsync(
    () => (directory ? api.scanRepos(directory) : Promise.resolve([])),
    [directory]
  );
}

export function useGuardStatus() {
  return useAsync(() => api.getGuardStatus(), []);
}

export function useSuggestions(minEvidence?: number) {
  return useAsync(() => api.getSuggestions(minEvidence), [minEvidence]);
}

export function useActivityCount() {
  return useAsync(() => api.getActivityCount(), []);
}
