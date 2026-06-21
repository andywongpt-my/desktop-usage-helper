import { create } from "zustand";

/**
 * Live usage state — driven by Rust background poll loop.
 * The backend emits `usage:statuses` events every poll interval (default 60s)
 * AND on every manual refresh. This store just mirrors the latest snapshot.
 *
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

  /** Replace the entire snapshot (statuses + providers). Called from App.jsx
   * boot and from the Dashboard refresh button. */
  setSnapshot: (snapshot) =>
    set({
      statuses: snapshot.statuses ?? {},
      providers: snapshot.providers ?? get().providers,
      lastRefresh: Date.now(),
    }),

  /** Update only the statuses map — used by the Rust event subscription. */
  setStatuses: (statuses) =>
    set({ statuses, lastRefresh: Date.now() }),

  /** Update only the providers metadata (e.g. after toggling enabled). */
  setProviders: (providers) => set({ providers }),

  setLoading: (isLoading) => set({ isLoading }),

  /** Patch a single provider's status — used by `set_one` future IPC. */
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
