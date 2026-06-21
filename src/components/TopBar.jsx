import { RefreshCw, Settings, Github } from "lucide-react";
import { useState } from "react";
import { useUsageStore } from "../stores/useUsageStore.js";
import { refreshAll } from "../lib/tauri.js";
import SettingsModal from "./SettingsModal.jsx";

export default function TopBar() {
  const [showSettings, setShowSettings] = useState(false);
  const isLoading = useUsageStore((s) => s.isLoading);
  const setStatuses = useUsageStore((s) => s.setStatuses);
  const setProviders = useUsageStore((s) => s.setProviders);
  const setLoading = useUsageStore((s) => s.setLoading);
  const providers = useUsageStore((s) => s.providers);

  const enabledCount = providers.filter((p) => p.enabled).length;

  const onRefresh = async () => {
    setLoading(true);
    try {
      const result = await refreshAll();
      setStatuses(result.statuses);
      setProviders(result.providers);
    } catch (err) {
      console.error("[TopBar] refresh failed:", err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <>
      <header className="flex items-center justify-between px-5 py-3 border-b border-gray-800 bg-gray-900/60 backdrop-blur-sm">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-accent to-blue-600 flex items-center justify-center text-white font-bold text-sm">
            U
          </div>
          <div>
            <h1 className="text-sm font-semibold text-gray-100">
              Desktop Usage Helper
            </h1>
            <p className="text-xs text-gray-500">
              {enabledCount} of {providers.length} providers active
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={onRefresh}
            disabled={isLoading}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs text-gray-300 hover:text-white hover:bg-gray-800 disabled:opacity-50 transition-colors"
            title="Refresh all providers now"
          >
            <RefreshCw
              size={14}
              className={isLoading ? "animate-spin" : ""}
            />
            <span>Refresh</span>
          </button>
          <button
            onClick={() => setShowSettings(true)}
            className="p-1.5 rounded-md text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
            title="Settings"
          >
            <Settings size={16} />
          </button>
          <a
            href="https://github.com/andywongpt-my/desktop-usage-helper"
            target="_blank"
            rel="noreferrer"
            className="p-1.5 rounded-md text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
            title="GitHub"
          >
            <Github size={16} />
          </a>
        </div>
      </header>
      {showSettings && <SettingsModal onClose={() => setShowSettings(false)} />}
    </>
  );
}
