import { useEffect, useState } from "react";
import { X, Eye, EyeOff, RotateCcw, CheckCircle2, XCircle, Bell, Power, TimerReset } from "lucide-react";
import { useConfigStore } from "../stores/useConfigStore.js";
import { useUsageStore } from "../stores/useUsageStore.js";
import { setApiKey, setProviderEnabled, setAutostart, getAutostartStatus } from "../lib/tauri.js";

function Section({ icon: Icon, title, children }) {
  return (
    <section className="section-card">
      <div className="mb-4 flex items-center gap-2 text-sm font-semibold text-white">
        <span className="grid h-8 w-8 place-items-center rounded-xl border border-white/10 bg-white/[0.04] text-slate-300">
          <Icon size={15} />
        </span>
        {title}
      </div>
      {children}
    </section>
  );
}

export default function SettingsModal({ onClose }) {
  const config = useConfigStore((s) => s.config);
  const setConfig = useConfigStore((s) => s.setConfig);
  const refreshEnv = useConfigStore((s) => s.refreshEnvKeys);
  const providers = useUsageStore((s) => s.providers);
  const [showKeys, setShowKeys] = useState({});

  const toggleEnabled = async (id, currentEnabled) => {
    try {
      await setProviderEnabled(id, !currentEnabled);
      await setConfig({});
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

  const toggleAutostart = async (currentEnabled) => {
    try {
      const next = await setAutostart(!currentEnabled);
      await setConfig({ autostartEnabled: next.autostartEnabled });
    } catch (err) {
      console.error("[Settings] setAutostart failed:", err);
    }
  };

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
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal-panel" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-start justify-between gap-4 border-b border-white/10 px-5 py-4 sm:px-6">
          <div>
            <h2 className="text-xl font-semibold tracking-tight text-white">Settings</h2>
            <p className="mt-1 text-sm text-slate-500">Tune refresh cadence, alerts, keys and startup behavior.</p>
          </div>
          <button
            onClick={onClose}
            className="chrome-button h-9 w-9 px-0"
            aria-label="Close settings"
          >
            <X size={17} />
          </button>
        </div>

        <div className="overflow-auto p-4 sm:p-6">
          <div className="grid gap-4 lg:grid-cols-[0.85fr_1.15fr]">
            <div className="space-y-4">
              <Section icon={TimerReset} title="Refresh">
                <label className="block text-sm">
                  <span className="mb-2 block text-slate-400">Poll interval</span>
                  <div className="flex items-center gap-2">
                    <input
                      type="number"
                      min={15}
                      max={3600}
                      value={config.pollIntervalSec}
                      onChange={(e) =>
                        setConfig({ pollIntervalSec: Math.max(15, Number(e.target.value)) })
                      }
                      className="small-input"
                    />
                    <span className="text-xs text-slate-500">seconds, minimum 15</span>
                  </div>
                </label>
              </Section>

              <Section icon={Bell} title="Alert thresholds">
                <p className="mb-4 text-xs leading-5 text-slate-500">
                  Warn and danger color cards. Toast sends a Windows notification when remaining usage drops below the threshold.
                </p>
                <div className="grid grid-cols-3 gap-2">
                  <label className="text-xs text-slate-400">
                    Warn
                    <input
                      type="number"
                      min={1}
                      max={99}
                      value={config.warnThresholdPct}
                      onChange={(e) => setConfig({ warnThresholdPct: Number(e.target.value) })}
                      className="small-input mt-2 w-full"
                    />
                  </label>
                  <label className="text-xs text-slate-400">
                    Toast
                    <input
                      type="number"
                      min={1}
                      max={99}
                      value={config.toastThresholdPct}
                      onChange={(e) =>
                        setConfig({ toastThresholdPct: Math.max(1, Math.min(99, Number(e.target.value))) })
                      }
                      className="small-input mt-2 w-full"
                    />
                  </label>
                  <label className="text-xs text-slate-400">
                    Danger
                    <input
                      type="number"
                      min={1}
                      max={99}
                      value={config.dangerThresholdPct}
                      onChange={(e) => setConfig({ dangerThresholdPct: Number(e.target.value) })}
                      className="small-input mt-2 w-full"
                    />
                  </label>
                </div>
              </Section>

              <Section icon={Power} title="Behavior">
                <div className="space-y-3">
                  <label className="flex items-start gap-3 text-sm">
                    <input
                      type="checkbox"
                      checked={config.notifyEnabled}
                      onChange={(e) => setConfig({ notifyEnabled: e.target.checked })}
                      className="mt-1 accent-accent"
                    />
                    <span className="text-slate-300">Show toast notification when usage drops below Toast %</span>
                  </label>
                  <label className="flex items-start gap-3 text-sm">
                    <input
                      type="checkbox"
                      checked={config.minimizeToTray}
                      onChange={(e) => setConfig({ minimizeToTray: e.target.checked })}
                      className="mt-1 accent-accent"
                    />
                    <span className="text-slate-300">Keep app running in tray when the window closes</span>
                  </label>
                  <label className="flex items-start gap-3 text-sm">
                    <input
                      type="checkbox"
                      checked={config.autostartEnabled}
                      onChange={() => toggleAutostart(config.autostartEnabled)}
                      className="mt-1 accent-accent"
                    />
                    <span className="text-slate-300">Launch on Windows startup</span>
                  </label>
                </div>
              </Section>
            </div>

            <Section icon={CheckCircle2} title="Providers">
              <div className="space-y-3">
                {providers.map((p) => {
                  const userCfg = config.providers?.[p.id] ?? {};
                  const hasUserKey = !!userCfg.customApiKey;
                  const envPresent = p.envPresent ?? false;
                  const hasKey = hasUserKey || envPresent;
                  const show = showKeys[p.id];
                  return (
                    <div key={p.id} className="rounded-2xl border border-white/10 bg-slate-950/35 p-4">
                      <div className="mb-3 flex items-start justify-between gap-3">
                        <div className="min-w-0">
                          <div className="flex flex-wrap items-center gap-2">
                            <span className="text-sm font-semibold text-white">{p.label}</span>
                            <span className="rounded-full border border-white/10 bg-white/[0.04] px-2 py-0.5 text-[10px] text-slate-500">
                              {p.kind}
                            </span>
                            {hasKey ? (
                              <span className="inline-flex items-center gap-1 text-[10px] text-emerald-300">
                                <CheckCircle2 size={11} /> key set
                              </span>
                            ) : (
                              <span className="inline-flex items-center gap-1 text-[10px] text-slate-500">
                                <XCircle size={11} /> no key
                              </span>
                            )}
                          </div>
                          {p.envVar && (
                            <p className="mt-1.5 font-mono text-[10px] text-slate-600">
                              env: {p.envVar} {envPresent ? "(detected)" : "(not set)"}
                            </p>
                          )}
                        </div>
                        <label className="flex shrink-0 items-center gap-2 text-xs text-slate-400">
                          <input
                            type="checkbox"
                            checked={p.enabled}
                            onChange={() => toggleEnabled(p.id, p.enabled)}
                            className="accent-accent"
                          />
                          enabled
                        </label>
                      </div>

                      <div className="flex items-center gap-2">
                        <div className="relative flex-1">
                          <input
                            type={show ? "text" : "password"}
                            aria-label={`${p.label} API key`}
                            placeholder={envPresent ? `using env ${p.envVar}, override here` : "paste API key"}
                            value={userCfg.customApiKey ?? ""}
                            onChange={(e) => setKey(p.id, e.target.value)}
                            className="field-input w-full pr-10 font-mono text-xs"
                          />
                          <button
                            type="button"
                            onClick={() => setShowKeys({ ...showKeys, [p.id]: !show })}
                            className="absolute right-2 top-1/2 -translate-y-1/2 rounded-lg p-1 text-slate-500 transition-colors hover:bg-white/[0.05] hover:text-slate-200"
                            aria-label={show ? "Hide API key" : "Show API key"}
                          >
                            {show ? <EyeOff size={13} /> : <Eye size={13} />}
                          </button>
                        </div>
                        {hasUserKey && (
                          <button
                            onClick={() => setKey(p.id, "")}
                            className="chrome-button h-9 w-9 px-0 text-slate-500 hover:text-rose-200"
                            title="Clear custom key"
                          >
                            <RotateCcw size={13} />
                          </button>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            </Section>
          </div>
        </div>

        <div className="flex items-center justify-between border-t border-white/10 px-5 py-3 text-[11px] text-slate-500 sm:px-6">
          <span>Config is stored locally with tauri-plugin-store.</span>
          <button onClick={onClose} className="chrome-button primary-action h-8 px-4">
            Done
          </button>
        </div>
      </div>
    </div>
  );
}
