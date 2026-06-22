import { useEffect, useState } from "react";
import { X, Eye, EyeOff, RotateCcw, CheckCircle2, XCircle, Bell, Power, TimerReset, Moon, MoonStar, Globe, Plus, Trash2, Cloud, Server, Tag, EyeOff as HideIcon, Download } from "lucide-react";
import { useConfigStore } from "../stores/useConfigStore.js";
import { useUsageStore } from "../stores/useUsageStore.js";
import { useI18nStore } from "../stores/useI18nStore.js";
import { setApiKey, setProviderEnabled, setAutostart, getAutostartStatus, syncExport, syncImport, checkForUpdates, downloadAndInstallUpdate } from "../lib/tauri.js";

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
  const t = useI18nStore((s) => s.t);
  const setLanguage = useI18nStore((s) => s.setLanguage);
  const [showKeys, setShowKeys] = useState({});
  const [syncStatus, setSyncStatus] = useState(null);
  const [updateStatus, setUpdateStatus] = useState(null); // null | "checking" | "available" | "downloading" | "done" | "error"
  const [updateInfo, setUpdateInfo] = useState(null);
  const [updateProgress, setUpdateProgress] = useState(0);

  const toggleEnabled = async (id, currentEnabled) => {
    try {
      const nextEnabled = !currentEnabled;
      const updated = await setProviderEnabled(id, nextEnabled);
      useConfigStore.setState({ config: updated });
      // Keep provider metadata in sync immediately. The checkbox and dashboard
      // visibility are driven by useUsageStore.providers, while the persisted
      // truth comes back through AppConfig. Without this, Enable appears to do
      // nothing until the next list/refresh cycle or app restart.
      useUsageStore.setState((state) => ({
        providers: state.providers.map((p) =>
          p.id === id ? { ...p, enabled: nextEnabled } : p
        ),
      }));
    } catch (err) {
      console.error("[Settings] toggle enabled failed:", err);
    }
  };

  const setKey = async (id, value) => {
    try {
      const updated = await setApiKey(id, value);
      // CRITICAL: update the config store with the returned config,
      // otherwise the input loses its value on the next render.
      useConfigStore.setState({ config: updated });
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

  const handleLanguageChange = (lang) => {
    setLanguage(lang);
    setConfig({ language: lang });
  };

  const handleSyncPush = async () => {
    setSyncStatus("pushing");
    try {
      const gistId = await syncExport();
      await setConfig({ syncGistId: gistId });
      setSyncStatus("pushed");
    } catch (err) {
      console.error("[Settings] sync push failed:", err);
      setSyncStatus("error");
    }
  };

  const handleSyncPull = async () => {
    setSyncStatus("pulling");
    try {
      await syncImport();
      await useConfigStore.getState().load();
      setSyncStatus("pulled");
    } catch (err) {
      console.error("[Settings] sync pull failed:", err);
      setSyncStatus("error");
    }
  };

  const handleCheckUpdate = async () => {
    setUpdateStatus("checking");
    setUpdateInfo(null);
    try {
      const update = await checkForUpdates();
      if (!update) {
        setUpdateStatus("none");
      } else {
        setUpdateInfo({ version: update.version, date: update.date, body: update.body });
        setUpdateStatus("available");
      }
    } catch (err) {
      console.error("[Settings] update check failed:", err);
      setUpdateStatus("error");
    }
  };

  const handleInstallUpdate = async () => {
    setUpdateStatus("downloading");
    setUpdateProgress(0);
    try {
      await downloadAndInstallUpdate((frac) => setUpdateProgress(frac));
      setUpdateStatus("done");
    } catch (err) {
      console.error("[Settings] update install failed:", err);
      setUpdateStatus("error");
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
            <h2 className="text-xl font-semibold tracking-tight text-white">{t("settings.title")}</h2>
            <p className="mt-1 text-sm text-slate-500">{t("settings.desc")}</p>
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
              {/* Refresh */}
              <Section icon={TimerReset} title={t("settings.refresh")}>
                <label className="block text-sm">
                  <span className="mb-2 block text-slate-400">{t("settings.poll_interval")}</span>
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
                    <span className="text-xs text-slate-500">{t("settings.seconds")}</span>
                  </div>
                </label>
                <label className="mt-3 block text-sm">
                  <span className="mb-2 block text-slate-400">{t("settings.startup_delay")}</span>
                  <div className="flex items-center gap-2">
                    <input
                      type="number"
                      min={0}
                      max={300}
                      value={config.startupDelaySec}
                      onChange={(e) =>
                        setConfig({ startupDelaySec: Math.max(0, Number(e.target.value)) })
                      }
                      className="small-input"
                    />
                    <span className="text-xs text-slate-500">{t("settings.delay_desc")}</span>
                  </div>
                </label>
              </Section>

              {/* Alert thresholds */}
              <Section icon={Bell} title={t("settings.alerts")}>
                <p className="mb-4 text-xs leading-5 text-slate-500">
                  {t("settings.alerts_desc")}
                </p>
                <div className="grid grid-cols-3 gap-2">
                  <label className="text-xs text-slate-400">
                    {t("settings.warn")}
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
                    {t("settings.toast")}
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
                    {t("settings.danger")}
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

                {/* DND */}
                <div className="mt-4 border-t border-white/10 pt-3">
                  <p className="mb-2 text-xs text-slate-400">{t("settings.dnd")}</p>
                  <p className="mb-3 text-[11px] text-slate-600">{t("settings.dnd_desc")}</p>
                  <div className="flex items-center gap-3">
                    <label className="text-xs text-slate-400">
                      {t("settings.dnd_start")}
                      <input
                        type="time"
                        value={config.dndStart ?? ""}
                        onChange={(e) => setConfig({ dndStart: e.target.value || null })}
                        className="small-input mt-1 w-24"
                      />
                    </label>
                    <label className="text-xs text-slate-400">
                      {t("settings.dnd_end")}
                      <input
                        type="time"
                        value={config.dndEnd ?? ""}
                        onChange={(e) => setConfig({ dndEnd: e.target.value || null })}
                        className="small-input mt-1 w-24"
                      />
                    </label>
                  </div>
                </div>
              </Section>

              {/* Behavior */}
              <Section icon={Power} title={t("settings.behavior")}>
                <div className="space-y-3">
                  <label className="flex items-start gap-3 text-sm">
                    <input
                      type="checkbox"
                      checked={config.notifyEnabled}
                      onChange={(e) => setConfig({ notifyEnabled: e.target.checked })}
                      className="mt-1 accent-accent"
                    />
                    <span className="text-slate-300">{t("settings.toast_notif")}</span>
                  </label>
                  <label className="flex items-start gap-3 text-sm">
                    <input
                      type="checkbox"
                      checked={config.minimizeToTray}
                      onChange={(e) => setConfig({ minimizeToTray: e.target.checked })}
                      className="mt-1 accent-accent"
                    />
                    <span className="text-slate-300">{t("settings.close_to_tray")}</span>
                  </label>
                  <label className="flex items-start gap-3 text-sm">
                    <input
                      type="checkbox"
                      checked={config.autostartEnabled}
                      onChange={() => toggleAutostart(config.autostartEnabled)}
                      className="mt-1 accent-accent"
                    />
                    <span className="text-slate-300">{t("settings.autostart")}</span>
                  </label>
                </div>
              </Section>

              {/* Language */}
              <Section icon={Globe} title={t("settings.language")}>
                <div className="flex gap-2">
                  <button
                    onClick={() => handleLanguageChange("en-US")}
                    className={`chrome-button ${config.language === "en-US" ? "primary-action" : ""}`}
                  >
                    English
                  </button>
                  <button
                    onClick={() => handleLanguageChange("zh-CN")}
                    className={`chrome-button ${config.language === "zh-CN" ? "primary-action" : ""}`}
                  >
                    中文
                  </button>
                </div>
              </Section>

              {/* Cross-device sync */}
              <Section icon={Cloud} title={t("settings.sync")}>
                <p className="mb-3 text-[11px] text-slate-600">{t("settings.sync_desc")}</p>
                <div className="space-y-2">
                  <input
                    type="password"
                    placeholder={t("settings.gist_token")}
                    value={config.syncGistToken ?? ""}
                    onChange={(e) => setConfig({ syncGistToken: e.target.value || null })}
                    className="field-input w-full font-mono text-xs"
                  />
                  <input
                    type="text"
                    placeholder={t("settings.gist_id")}
                    value={config.syncGistId ?? ""}
                    onChange={(e) => setConfig({ syncGistId: e.target.value || null })}
                    className="field-input w-full font-mono text-xs"
                  />
                  <div className="flex gap-2 pt-1">
                    <button onClick={handleSyncPush} className="chrome-button primary-action" disabled={syncStatus === "pushing"}>
                      <Cloud size={13} />
                      {syncStatus === "pushing" ? "..." : t("settings.sync_push")}
                    </button>
                    <button onClick={handleSyncPull} className="chrome-button" disabled={syncStatus === "pulling"}>
                      <RotateCcw size={13} />
                      {syncStatus === "pulling" ? "..." : t("settings.sync_pull")}
                    </button>
                  </div>
                  {syncStatus === "pushed" && <p className="text-[10px] text-emerald-400">Pushed ✓</p>}
                  {syncStatus === "pulled" && <p className="text-[10px] text-emerald-400">Pulled ✓</p>}
                  {syncStatus === "error" && <p className="text-[10px] text-rose-400">Failed — check console</p>}
                </div>
              </Section>

              {/* Auto-update */}
              <Section icon={Download} title="Updates">
                <p className="mb-3 text-[11px] text-slate-600">
                  Check for and install the latest version automatically.
                </p>
                {updateStatus === "available" && updateInfo && (
                  <div className="mb-3 rounded-lg border border-white/10 bg-slate-950/50 p-3">
                    <p className="text-xs text-slate-300">
                      <span className="font-semibold text-white">v{updateInfo.version}</span>
                      {updateInfo.date && <span className="text-slate-500"> · {updateInfo.date}</span>}
                    </p>
                    {updateInfo.body && (
                      <p className="mt-1.5 whitespace-pre-wrap text-[11px] leading-4 text-slate-500 line-clamp-6">
                        {updateInfo.body}
                      </p>
                    )}
                  </div>
                )}
                {updateStatus === "downloading" && (
                  <div className="mb-3">
                    <div className="h-1.5 overflow-hidden rounded-full bg-white/10">
                      <div
                        className="h-full rounded-full bg-accent transition-all"
                        style={{ width: `${Math.round(updateProgress * 100)}%` }}
                      />
                    </div>
                    <p className="mt-1 text-[10px] text-slate-500">{Math.round(updateProgress * 100)}%</p>
                  </div>
                )}
                <div className="flex gap-2">
                  {updateStatus === "available" ? (
                    <button onClick={handleInstallUpdate} className="chrome-button primary-action" disabled={updateStatus === "downloading"}>
                      <Download size={13} />
                      {updateStatus === "downloading" ? "Installing..." : "Download & Install"}
                    </button>
                  ) : (
                    <button onClick={handleCheckUpdate} className="chrome-button primary-action" disabled={updateStatus === "checking"}>
                      {updateStatus === "checking" ? "Checking..." : "Check for Updates"}
                    </button>
                  )}
                </div>
                {updateStatus === "none" && <p className="mt-2 text-[10px] text-emerald-400">You're on the latest version ✓</p>}
                {updateStatus === "done" && <p className="mt-2 text-[10px] text-emerald-400">Update installed — restarting... ✓</p>}
                {updateStatus === "error" && <p className="mt-2 text-[10px] text-rose-400">Update failed — check console</p>}
              </Section>

              {/* Service mode info */}
              <Section icon={Server} title={t("settings.service_mode")}>
                <p className="text-[11px] leading-5 text-slate-500">{t("settings.service_desc")}</p>
                <code className="mt-2 block rounded-lg border border-white/10 bg-slate-950/50 px-3 py-2 font-mono text-[10px] text-slate-400">
                  desktop-usage-helper.exe --service
                </code>
              </Section>
            </div>

            {/* Providers */}
            <Section icon={CheckCircle2} title={t("settings.providers")}>
              <div className="space-y-3">
                {providers.map((p) => {
                  const userCfg = config.providers?.[p.id] ?? {};
                  const enabled = userCfg.enabled ?? p.enabled;
                  const hasUserKey = !!userCfg.customApiKey;
                  const envPresent = p.envPresent ?? false;
                  const hasKey = hasUserKey || envPresent;
                  const show = showKeys[p.id];
                  const accounts = userCfg.accounts ?? [];
                  const tags = userCfg.tags ?? [];
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
                                <CheckCircle2 size={11} /> {t("settings.key_set")}
                              </span>
                            ) : (
                              <span className="inline-flex items-center gap-1 text-[10px] text-slate-500">
                                <XCircle size={11} /> {t("settings.no_key")}
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
                            checked={!!enabled}
                            onChange={() => toggleEnabled(p.id, !!enabled)}
                            className="accent-accent"
                          />
                          {t("settings.enabled")}
                        </label>
                      </div>

                      {/* Hide toggle */}
                      <div className="mb-3 flex items-center justify-end">
                        <label className="flex items-center gap-1.5 text-[10px] text-slate-500">
                          <input
                            type="checkbox"
                            checked={!!userCfg.hidden}
                            onChange={(e) => setConfig({ providers: { [p.id]: { ...userCfg, hidden: e.target.checked } } })}
                            className="accent-accent"
                          />
                          <HideIcon size={11} />
                          {t("settings.hide_from_dashboard")}
                        </label>
                      </div>

                      {/* API key */}
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

                      {/* Multi-account */}
                      {accounts.length > 0 && (
                        <div className="mt-2 space-y-1.5">
                          {accounts.map((acc, i) => (
                            <div key={i} className="flex items-center gap-2">
                              <input
                                type="text"
                                placeholder={`Account ${i + 1} label`}
                                value={acc.label ?? ""}
                                onChange={(e) => {
                                  const next = [...accounts];
                                  next[i] = { ...acc, label: e.target.value };
                                  setConfig({ providers: { [p.id]: { ...userCfg, accounts: next } } });
                                }}
                                className="field-input flex-1 text-xs"
                              />
                              <input
                                type="password"
                                placeholder="API key"
                                value={acc.apiKey ?? ""}
                                onChange={(e) => {
                                  const next = [...accounts];
                                  next[i] = { ...acc, apiKey: e.target.value };
                                  setConfig({ providers: { [p.id]: { ...userCfg, accounts: next } } });
                                }}
                                className="field-input flex-1 font-mono text-xs"
                              />
                              <button
                                onClick={() => {
                                  const next = accounts.filter((_, j) => j !== i);
                                  setConfig({ providers: { [p.id]: { ...userCfg, accounts: next } } });
                                }}
                                className="chrome-button h-8 w-8 px-0 text-slate-500 hover:text-rose-200"
                              >
                                <Trash2 size={12} />
                              </button>
                            </div>
                          ))}
                        </div>
                      )}
                      <button
                        onClick={() => {
                          const next = [...accounts, { label: null, apiKey: "", enabled: true }];
                          setConfig({ providers: { [p.id]: { ...userCfg, accounts: next } } });
                        }}
                        className="mt-2 inline-flex items-center gap-1 text-[11px] text-accent hover:text-accent/80"
                      >
                        <Plus size={12} />
                        {t("settings.add_account")}
                      </button>

                      {/* Cost per unit */}
                      <div className="mt-3 flex items-center gap-2">
                        <label className="text-[11px] text-slate-500">{t("settings.cost_per_unit")}</label>
                        <input
                          type="number"
                          step="0.01"
                          min="0"
                          placeholder="0.00"
                          value={userCfg.costPerUnit ?? ""}
                          onChange={(e) => {
                            const v = e.target.value ? Number(e.target.value) : null;
                            setConfig({ providers: { [p.id]: { ...userCfg, costPerUnit: v } } });
                          }}
                          className="small-input w-20"
                        />
                      </div>

                      {/* Tags */}
                      <div className="mt-2 flex items-center gap-2">
                        <Tag size={12} className="text-slate-600" />
                        <input
                          type="text"
                          placeholder={t("settings.tags_desc")}
                          value={tags.join(", ")}
                          onChange={(e) => {
                            const next = e.target.value.split(",").map((s) => s.trim()).filter(Boolean);
                            setConfig({ providers: { [p.id]: { ...userCfg, tags: next } } });
                          }}
                          className="field-input flex-1 text-xs"
                        />
                      </div>
                    </div>
                  );
                })}
              </div>
            </Section>
          </div>
        </div>

        <div className="flex items-center justify-between border-t border-white/10 px-5 py-3 text-[11px] text-slate-500 sm:px-6">
          <span>{t("settings.config_stored")}</span>
          <button onClick={onClose} className="chrome-button primary-action h-8 px-4">
            {t("settings.done")}
          </button>
        </div>
      </div>
    </div>
  );
}