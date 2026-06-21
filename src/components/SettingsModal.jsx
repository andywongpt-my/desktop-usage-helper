import { useEffect, useState } from "react";
import { X, Eye, EyeOff, RotateCcw, CheckCircle2, XCircle } from "lucide-react";
import { useConfigStore } from "../stores/useConfigStore.js";
import { useUsageStore } from "../stores/useUsageStore.js";
import { setApiKey, setProviderEnabled, setAutostart, getAutostartStatus } from "../lib/tauri.js";

export default function SettingsModal({ onClose }) {
  const config = useConfigStore((s) => s.config);
  const envKeys = useConfigStore((s) => s.envKeys);
  const setConfig = useConfigStore((s) => s.setConfig);
  const refreshEnv = useConfigStore((s) => s.refreshEnvKeys);
  const providers = useUsageStore((s) => s.providers);
  const [showKeys, setShowKeys] = useState({});

  const toggleEnabled = async (id, currentEnabled) => {
    try {
      await setProviderEnabled(id, !currentEnabled);
      await setConfig({}); // trigger reload
    } catch (err) {
      console.error("[Settings] toggle enabled failed:", err);
    }
  };

  const setKey = async (id, value) => {
    try {
      await setApiKey(id, value);
      await refreshEnv();
    } catch (err) {
      console.error("[Settings] setApiKey failed:", err);
    }
  };

  // OS-level autostart toggle — calls the autostart plugin then persists.
  // On boot, Settings is mounted before this fires; we surface the OS state
  // (not the cached config flag) so the checkbox matches reality.
  const toggleAutostart = async (currentEnabled) => {
    try {
      const next = await setAutostart(!currentEnabled);
      await setConfig({ autostartEnabled: next.autostartEnabled });
    } catch (err) {
      console.error("[Settings] setAutostart failed:", err);
    }
  };

  // On mount, reconcile the OS-level autostart state with the UI checkbox.
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const enabled = await getAutostartStatus();
        if (cancelled) return;
        if (enabled !== config.autostartEnabled) {
          await setConfig({ autostartEnabled: enabled });
        }
      } catch (err) {
        console.error("[Settings] getAutostartStatus failed:", err);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div
      className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-50 p-4 animate-fade-in"
      onClick={onClose}
    >
      <div
        className="bg-gray-900 border border-gray-800 rounded-2xl w-full max-w-2xl max-h-[85vh] overflow-hidden flex flex-col shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-800">
          <h2 className="text-lg font-semibold text-gray-100">Settings</h2>
          <button
            onClick={onClose}
            className="p-1.5 rounded-md text-gray-400 hover:text-white hover:bg-gray-800"
          >
            <X size={18} />
          </button>
        </div>

        <div className="overflow-auto p-6 space-y-6">
          {/* Poll interval */}
          <section>
            <h3 className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-3">
              Refresh
            </h3>
            <label className="flex items-center gap-3 text-sm">
              <span className="text-gray-300 w-32">Poll interval</span>
              <input
                type="number"
                min={15}
                max={3600}
                value={config.pollIntervalSec}
                onChange={(e) =>
                  setConfig({ pollIntervalSec: Math.max(15, Number(e.target.value)) })
                }
                className="bg-gray-800 border border-gray-700 rounded-md px-3 py-1.5 w-24 text-sm font-mono focus:outline-none focus:border-accent"
              />
              <span className="text-gray-500 text-xs">seconds (min 15)</span>
            </label>
          </section>

          {/* Thresholds */}
          <section>
            <h3 className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-3">
              Alert thresholds
            </h3>
            <p className="text-xs text-gray-500 mb-3">
              <span className="text-gray-400">Warn / Danger</span> drives card
              color.{" "}
              <span className="text-gray-400">Toast</span> fires a Windows
              notification when a provider's remaining % drops below the
              threshold (and recovers above it).
            </p>
            <div className="grid grid-cols-3 gap-3">
              <label className="flex items-center gap-2 text-sm">
                <span className="text-gray-300 w-16">Warn</span>
                <input
                  type="number"
                  min={1}
                  max={99}
                  value={config.warnThresholdPct}
                  onChange={(e) =>
                    setConfig({ warnThresholdPct: Number(e.target.value) })
                  }
                  className="bg-gray-800 border border-gray-700 rounded-md px-2 py-1.5 w-16 text-sm font-mono focus:outline-none focus:border-accent"
                />
                <span className="text-gray-500 text-xs">%</span>
              </label>
              <label className="flex items-center gap-2 text-sm">
                <span className="text-gray-300 w-16">Toast</span>
                <input
                  type="number"
                  min={1}
                  max={99}
                  value={config.toastThresholdPct}
                  onChange={(e) =>
                    setConfig({
                      toastThresholdPct: Math.max(
                        1,
                        Math.min(99, Number(e.target.value))
                      ),
                    })
                  }
                  className="bg-gray-800 border border-gray-700 rounded-md px-2 py-1.5 w-16 text-sm font-mono focus:outline-none focus:border-accent"
                />
                <span className="text-gray-500 text-xs">%</span>
              </label>
              <label className="flex items-center gap-2 text-sm">
                <span className="text-gray-300 w-16">Danger</span>
                <input
                  type="number"
                  min={1}
                  max={99}
                  value={config.dangerThresholdPct}
                  onChange={(e) =>
                    setConfig({ dangerThresholdPct: Number(e.target.value) })
                  }
                  className="bg-gray-800 border border-gray-700 rounded-md px-2 py-1.5 w-16 text-sm font-mono focus:outline-none focus:border-accent"
                />
                <span className="text-gray-500 text-xs">%</span>
              </label>
            </div>
          </section>

          {/* Providers */}
          <section>
            <h3 className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-3">
              Providers
            </h3>
            <div className="space-y-2">
              {providers.map((p) => {
                const userCfg = config.providers?.[p.id] ?? {};
                const hasUserKey = !!userCfg.customApiKey;
                const envPresent = p.envPresent ?? false;
                const hasKey = hasUserKey || envPresent;
                const show = showKeys[p.id];
                return (
                  <div
                    key={p.id}
                    className="bg-gray-850 border border-gray-800 rounded-lg p-4"
                  >
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-medium text-gray-200">
                          {p.label}
                        </span>
                        <span className="text-[10px] text-gray-500 uppercase tracking-wider">
                          {p.kind}
                        </span>
                        {hasKey ? (
                          <span className="flex items-center gap-1 text-[10px] text-emerald-400">
                            <CheckCircle2 size={10} /> key set
                          </span>
                        ) : (
                          <span className="flex items-center gap-1 text-[10px] text-gray-500">
                            <XCircle size={10} /> no key
                          </span>
                        )}
                      </div>
                      <label className="flex items-center gap-2 text-xs cursor-pointer">
                        <input
                          type="checkbox"
                          checked={p.enabled}
                          onChange={() => toggleEnabled(p.id, p.enabled)}
                          className="accent-accent"
                        />
                        <span className="text-gray-400">enabled</span>
                      </label>
                    </div>

                    <div className="flex items-center gap-2">
                      <div className="flex-1 relative">
                        <input
                          type={show ? "text" : "password"}
                          placeholder={
                            envPresent
                              ? `using env ${p.envVar} — override here`
                              : "paste API key…"
                          }
                          value={userCfg.customApiKey ?? ""}
                          onChange={(e) => setKey(p.id, e.target.value)}
                          className="w-full bg-gray-900 border border-gray-700 rounded-md px-3 py-1.5 pr-9 text-xs font-mono focus:outline-none focus:border-accent"
                        />
                        <button
                          type="button"
                          onClick={() =>
                            setShowKeys({ ...showKeys, [p.id]: !show })
                          }
                          className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300"
                        >
                          {show ? <EyeOff size={12} /> : <Eye size={12} />}
                        </button>
                      </div>
                      {hasUserKey && (
                        <button
                          onClick={() => setKey(p.id, "")}
                          className="p-1.5 text-gray-500 hover:text-danger rounded-md"
                          title="Clear custom key"
                        >
                          <RotateCcw size={12} />
                        </button>
                      )}
                    </div>

                    {p.envVar && (
                      <p className="text-[10px] text-gray-600 mt-1.5 font-mono">
                        env: {p.envVar} {envPresent ? "(detected)" : "(not set)"}
                      </p>
                    )}
                  </div>
                );
              })}
            </div>
          </section>

          {/* Notifications + Autostart + Tray */}
          <section>
            <h3 className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-3">
              Behavior
            </h3>
            <label className="flex items-center gap-3 text-sm mb-2">
              <input
                type="checkbox"
                checked={config.notifyEnabled}
                onChange={(e) =>
                  setConfig({ notifyEnabled: e.target.checked })
                }
                className="accent-accent"
              />
              <span className="text-gray-300">
                Show toast notification when usage drops below Toast %
              </span>
            </label>
            <label className="flex items-center gap-3 text-sm mb-2">
              <input
                type="checkbox"
                checked={config.minimizeToTray}
                onChange={(e) =>
                  setConfig({ minimizeToTray: e.target.checked })
                }
                className="accent-accent"
              />
              <span className="text-gray-300">
                Keep app running in tray when window is closed (close-to-tray)
              </span>
            </label>
            <label className="flex items-center gap-3 text-sm">
              <input
                type="checkbox"
                checked={config.autostartEnabled}
                onChange={() => toggleAutostart(config.autostartEnabled)}
                className="accent-accent"
              />
              <span className="text-gray-300">
                Launch on Windows startup
              </span>
            </label>
          </section>
        </div>

        <div className="px-6 py-3 border-t border-gray-800 text-[10px] text-gray-500 font-mono flex justify-between">
          <span>Config stored in tauri-plugin-store (AppData)</span>
          <button onClick={onClose} className="text-accent hover:underline">
            Done
          </button>
        </div>
      </div>
    </div>
  );
}
