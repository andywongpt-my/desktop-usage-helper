import { useEffect } from "react";
import { useUsageStore } from "../stores/useUsageStore.js";
import { useConfigStore } from "../stores/useConfigStore.js";
import { refreshAll } from "../lib/tauri.js";
import ProviderCard from "./ProviderCard.jsx";
import EmptyState from "./EmptyState.jsx";

export default function Dashboard() {
  const providers = useUsageStore((s) => s.getVisibleProviders());
  const setStatuses = useUsageStore((s) => s.setStatuses);
  const setProviders = useUsageStore((s) => s.setProviders);
  const setLoading = useUsageStore((s) => s.setLoading);
  const isLoading = useUsageStore((s) => s.isLoading);
  const pollIntervalSec = useConfigStore((s) => s.config.pollIntervalSec);

  // Auto-refresh loop
  useEffect(() => {
    let cancelled = false;
    const tick = async () => {
      if (cancelled || document.hidden) return;
      setLoading(true);
      try {
        const result = await refreshAll();
        if (cancelled) return;
        setStatuses(result.statuses);
        setProviders(result.providers);
      } catch (err) {
        console.error("[Dashboard] refresh failed:", err);
      } finally {
        if (!cancelled) setLoading(false);
      }
    };
    const interval = setInterval(tick, Math.max(15, pollIntervalSec) * 1000);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [pollIntervalSec, setStatuses, setProviders, setLoading]);

  // Re-poll when window regains focus
  useEffect(() => {
    const onFocus = async () => {
      try {
        const result = await refreshAll();
        setStatuses(result.statuses);
        setProviders(result.providers);
      } catch {}
    };
    window.addEventListener("focus", onFocus);
    return () => window.removeEventListener("focus", onFocus);
  }, [setStatuses, setProviders]);

  if (providers.length === 0) {
    return <EmptyState />;
  }

  return (
    <div className="p-5">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-sm font-medium text-gray-400 uppercase tracking-wider">
          Active providers
        </h2>
        {isLoading && (
          <span className="text-xs text-gray-500 animate-pulse-slow">
            syncing…
          </span>
        )}
      </div>
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
        {providers.map((p) => (
          <ProviderCard key={p.id} provider={p} />
        ))}
      </div>
    </div>
  );
}
