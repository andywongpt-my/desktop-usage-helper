import { create } from "zustand";

/**
 * Live usage state — driven by Rust background poll loop.
 * The backend emits `usage:statuses` events every poll interval (default 60s)
 * AND on every manual refresh. This store just mirrors the latest snapshot.
 */
export const useUsageStore = create((set, get) => ({
  statuses: {},
  providers: [],
  lastRefresh: null,
  isLoading: false,
  historyCache: {}, // id → { points, fetchedAt }

  setSnapshot: (snapshot) =>
    set({
      statuses: snapshot.statuses ?? {},
      providers: snapshot.providers ?? get().providers,
      lastRefresh: Date.now(),
    }),

  setStatuses: (statuses) =>
    set({ statuses, lastRefresh: Date.now() }),

  setProviders: (providers) => set({ providers }),

  setLoading: (isLoading) => set({ isLoading }),

  setOne: (id, status) =>
    set((s) => ({ statuses: { ...s.statuses, [id]: status } })),

  setHistory: (id, points) =>
    set((s) => ({
      historyCache: { ...s.historyCache, [id]: { points, fetchedAt: Date.now() } },
    })),

  getVisibleProviders: () => {
    const { providers, statuses } = get();
    return providers
      .filter((p) => p.enabled)
      .map((p) => ({ ...p, status: statuses[p.id] }))
      .sort((a, b) => {
        const order = { danger: 0, warn: 1, ok: 2, unknown: 3 };
        const sa = order[a.status?.state ?? "unknown"];
        const sb = order[b.status?.state ?? "unknown"];
        if (sa !== sb) return sa - sb;
        return a.label.localeCompare(b.label);
      });
  },

  /** Get providers visible on the dashboard (enabled + not hidden). */
  getDashboardProviders: (config) => {
    const visible = get().getVisibleProviders();
    if (!config?.providers) return visible;
    return visible.filter((p) => {
      const userCfg = config.providers[p.id] ?? {};
      return !userCfg.hidden;
    });
  },
}));