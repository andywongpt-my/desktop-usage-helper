import { create } from "zustand";

/**
 * Live usage state — refreshed by Rust every poll interval.
 * Status shape (from Rust):
 *   { id, label, kind, state: 'ok'|'warn'|'danger'|'unknown',
 *     primary: { label, used, limit, unit, resetAt? },
 *     secondary?: { label, used, limit, unit, resetAt? },
 *     error?: string, fetchedAt, latencyMs }
 */
export const useUsageStore = create((set, get) => ({
  statuses: {},
  providers: [], // provider metadata list (id, label, kind, enabled, hasKey)
  lastRefresh: null,
  isLoading: false,

  setStatuses: (statuses) => set({ statuses, lastRefresh: Date.now() }),
  setProviders: (providers) => set({ providers }),

  setLoading: (isLoading) => set({ isLoading }),

  setOne: (id, status) =>
    set((s) => ({ statuses: { ...s.statuses, [id]: status } })),

  getVisibleProviders: () => {
    const { providers, statuses } = get();
    return providers
      .filter((p) => p.enabled)
      .map((p) => ({ ...p, status: statuses[p.id] }))
      .sort((a, b) => {
        // Danger first, warn next, then ok, unknown last
        const order = { danger: 0, warn: 1, ok: 2, unknown: 3 };
        const sa = order[a.status?.state ?? "unknown"];
        const sb = order[b.status?.state ?? "unknown"];
        if (sa !== sb) return sa - sb;
        return a.label.localeCompare(b.label);
      });
  },
}));
