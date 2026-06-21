import { useEffect } from "react";
import Dashboard from "./components/Dashboard.jsx";
import TopBar from "./components/TopBar.jsx";
import StatusBar from "./components/StatusBar.jsx";
import { useUsageStore } from "./stores/useUsageStore.js";
import { useConfigStore } from "./stores/useConfigStore.js";
import { refreshAll } from "./lib/tauri.js";

export default function App() {
  const setStatuses = useUsageStore((s) => s.setStatuses);
  const setProviders = useUsageStore((s) => s.setProviders);
  const lastRefresh = useUsageStore((s) => s.lastRefresh);
  const isLoading = useUsageStore((s) => s.isLoading);
  const configLoaded = useConfigStore((s) => s.loaded);

  // Boot: load config, then refresh
  useEffect(() => {
    let cancelled = false;
    (async () => {
      await useConfigStore.getState().load();
      if (cancelled) return;
      // First auto-refresh right after config is ready
      try {
        const result = await refreshAll();
        if (cancelled) return;
        setStatuses(result.statuses);
        setProviders(result.providers);
      } catch (err) {
        console.error("[App] initial refresh failed:", err);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [setStatuses, setProviders]);

  if (!configLoaded) {
    return (
      <div className="h-screen flex items-center justify-center text-gray-500">
        Loading configuration…
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col bg-gray-950 text-gray-200 animate-fade-in">
      <TopBar />
      <main className="flex-1 overflow-auto">
        <Dashboard />
      </main>
      <StatusBar lastRefresh={lastRefresh} loading={isLoading} />
    </div>
  );
}
