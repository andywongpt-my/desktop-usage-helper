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
  const refreshInFlight = useRef(false);

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

  useEffect(() => {
    let unlisten;
    (async () => {
      unlisten = await onUsageStatuses((statuses) => {
        setStatuses(statuses);
      });
    })();
    return () => {
      if (typeof unlisten === "function") unlisten();
    };
  }, [setStatuses]);

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
      <div className="min-h-[100dvh] app-shell flex items-center justify-center text-slate-400">
        <div className="loading-card">
          <div className="loading-mark">U</div>
          <div>
            <p className="text-sm font-semibold text-slate-100">Loading configuration</p>
            <p className="mt-1 text-xs text-slate-500">Reading local store and provider metadata.</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-[100dvh] app-shell flex flex-col text-slate-200 animate-fade-in">
      <TopBar onRefresh={triggerRefresh} onOpenSettings={() => setShowSettings(true)} />
      <main className="flex-1 overflow-auto px-4 pb-4 pt-3 sm:px-5">
        <Dashboard onRefresh={triggerRefresh} />
      </main>
      <StatusBar lastRefresh={lastRefresh} loading={isLoading} />
      {showSettings && <SettingsModal onClose={() => setShowSettings(false)} />}
    </div>
  );
}
