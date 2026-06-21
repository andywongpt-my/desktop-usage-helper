import { useEffect } from "react";
import { useUsageStore } from "../stores/useUsageStore.js";
import ProviderCard from "./ProviderCard.jsx";
import EmptyState from "./EmptyState.jsx";

/**
 * Dashboard — the main grid of provider cards.
 *
 * Polling is owned by the Rust backend (poll loop emits `usage:statuses`).
 * The renderer only:
 *   - Re-fetches on window focus (cheap, instant feedback after a meal break)
 *   - Receives event updates via the listener in App.jsx
 */
export default function Dashboard({ onRefresh }) {
  const providers = useUsageStore((s) => s.getVisibleProviders());
  const isLoading = useUsageStore((s) => s.isLoading);

  // Re-poll when window regains focus — gives instant freshness on alt-tab.
  useEffect(() => {
    const onFocus = async () => {
      if (typeof onRefresh === "function") {
        try {
          await onRefresh();
        } catch {}
      }
    };
    window.addEventListener("focus", onFocus);
    return () => window.removeEventListener("focus", onFocus);
  }, [onRefresh]);

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
