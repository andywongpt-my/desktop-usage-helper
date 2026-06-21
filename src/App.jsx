import { useEffect, useState, useCallback, useRef } from "react";
import Dashboard from "./components/Dashboard.jsx";
import TopBar from "./components/TopBar.jsx";
import StatusBar from "./components/StatusBar.jsx";
import SettingsModal from "./components/SettingsModal.jsx";
import { useUsageStore } from "./stores/useUsageStore.js";
import { useConfigStore } from "./stores/useConfigStore.js";
import {
  listProviders,
  refreshAll,
  onUsageStatuses,
  onTrayRefreshRequested,
  onTrayOpenSettings,
} from "./lib/tauri.js";

export default function App() {
  const setStatuses = useUsageStore((s) => s.setStatuses);
  const setProviders = useUsageStore((s) => s.setProviders);
  const lastRefresh = useUsageStore((s) => s.lastRefresh);
  const isLoading = useUsageStore((s) => s.isLoading);
  const configLoaded = useConfigStore((s) => s.loaded);
  const [showSettings, setShowSettings] = useState(false);
  // Coalesce rapid-fire refresh events from the tray + manual clicks.
  const refreshInFlight = useRef(false);

  /** Trigger an immediate refresh — coalesced so concurrent calls don't double-fetch. */
  const triggerRefresh = useCallback(async () => {
    if (refreshInFlight.current) return;
    refreshInFlight.current = true;
    const setLoading = useUsageStore.getState().setLoading;
    setLoading(true);
    try {
      const result = await refreshAll();
      setStatuses(result.statuses);
      setProviders(result.providers);
    } catch (err) {
      console.error("[App] refresh failed:", err);
    } finally {
      setLoading(false);
      refreshInFlight.current = false;
    }
  }, [setStatuses, setProviders]);

  // Boot: load config, then refresh once to populate the UI immediately.
  // The Rust background poll loop will continue to drive updates every poll interval.
  useEffect(() => {
    let cancelled = false;
    (async () => {
      await useConfigStore.getState().load();
      if (cancelled) return;
      try {
        const [result, providers] = await Promise.all([
          refreshAll(),
          listProviders(),
        ]);
        if (cancelled) return;
        setStatuses(result.statuses);
        setProviders(providers);
      } catch (err) {
        console.error("[App] initial refresh failed:", err);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [setStatuses, setProviders]);

  // Subscribe to Rust-driven refresh results (background poll loop + tray menu).
  useEffect(() => {
    let unlisten;
    (async () => {
      unlisten = await onUsageStatuses((statuses) => {
        // Update statuses map; do not overwrite the providers list (it changes
        // only when the user toggles enabled, which has its own refresh path).
        setStatuses(statuses);
      });
    })();
    return () => {
      if (typeof unlisten === "function") unlisten();
    };
  }, [setStatuses]);

  // Tray menu: "Refresh now" — fire an immediate refresh.
  useEffect(() => {
    let unlisten;
    (async () => {
      unlisten = await onTrayRefreshRequested(() => {
        triggerRefresh();
      });
    })();
    return () => {
      if (typeof unlisten === "function") unlisten();
    };
  }, [triggerRefresh]);

  // Tray menu: "Open settings" — surface the SettingsModal.
  useEffect(() => {
    let unlisten;
    (async () => {
      unlisten = await onTrayOpenSettings(() => {
        setShowSettings(true);
      });
    })();
    return () => {
      if (typeof unlisten === "function") unlisten();
    };
  }, []);

  if (!configLoaded) {
    return (
      <div className="h-screen flex items-center justify-center text-gray-500">
        Loading configuration…
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col bg-gray-950 text-gray-200 animate-fade-in">
      <TopBar onRefresh={triggerRefresh} onOpenSettings={() => setShowSettings(true)} />
      <main className="flex-1 overflow-auto">
        <Dashboard onRefresh={triggerRefresh} />
      </main>
      <StatusBar lastRefresh={lastRefresh} loading={isLoading} />
      {showSettings && <SettingsModal onClose={() => setShowSettings(false)} />}
    </div>
  );
}
