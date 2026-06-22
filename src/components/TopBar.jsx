import { RefreshCw, Settings, Github, Bell, PanelTopClose, Sun, Moon, LayoutGrid } from "lucide-react";
import { useUsageStore } from "../stores/useUsageStore.js";
import { useI18nStore } from "../stores/useI18nStore.js";
import { useThemeStore } from "../stores/useThemeStore.js";
import { useConfigStore } from "../stores/useConfigStore.js";
import { toggleWidget } from "../lib/tauri.js";

function StatePill({ count, tone, label }) {
  const toneClass = {
    ok: "text-emerald-200 border-emerald-300/20 bg-emerald-300/10",
    warn: "text-amber-200 border-amber-300/20 bg-amber-300/10",
    danger: "text-rose-200 border-rose-300/20 bg-rose-300/10",
    neutral: "text-slate-300 border-white/10 bg-white/[0.04]",
  }[tone];
  return (
    <span className={`inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-[11px] ${toneClass}`}>
      <span className="font-mono text-[12px] text-white">{count}</span>
      <span>{label}</span>
    </span>
  );
}

export default function TopBar({ onRefresh, onOpenSettings }) {
  const isLoading = useUsageStore((s) => s.isLoading);
  const providers = useUsageStore((s) => s.providers);
  const statuses = useUsageStore((s) => s.statuses);
  const t = useI18nStore((s) => s.t);
  const toggleTheme = useThemeStore((s) => s.toggle);
  const theme = useThemeStore((s) => s.theme);
  const setConfig = useConfigStore((s) => s.setConfig);

  const enabled = providers.filter((p) => p.enabled);
  const counts = enabled.reduce(
    (acc, p) => {
      const state = statuses[p.id]?.state ?? "unknown";
      if (state === "danger") acc.danger += 1;
      else if (state === "warn") acc.warn += 1;
      else if (state === "ok") acc.ok += 1;
      else acc.unknown += 1;
      return acc;
    },
    { ok: 0, warn: 0, danger: 0, unknown: 0 }
  );

  const handleToggleTheme = () => {
    const next = toggleTheme();
    setConfig({ theme: next });
  };

  const handleToggleWidget = () => {
    toggleWidget().catch((e) => console.error("[TopBar] toggleWidget failed:", e));
  };

  return (
    <header className="topbar">
      <div className="flex min-w-0 items-center gap-3">
        <div className="brand-mark">U</div>
        <div className="min-w-0">
          <h1 className="truncate text-sm font-semibold tracking-tight text-white">
            {t("app.title")}
          </h1>
          <p className="truncate text-[11px] text-slate-500">
            {t("app.subtitle")}
          </p>
        </div>
      </div>

      <div className="hidden min-w-0 flex-1 items-center justify-center gap-2 lg:flex">
        <StatePill count={counts.danger} tone="danger" label={t("topbar.critical")} />
        <StatePill count={counts.warn} tone="warn" label={t("topbar.low")} />
        <StatePill count={counts.ok} tone="ok" label={t("topbar.healthy")} />
        <StatePill count={counts.unknown} tone="neutral" label={t("topbar.unknown")} />
      </div>

      <div className="flex shrink-0 items-center gap-2">
        <button
          onClick={handleToggleWidget}
          className="chrome-button"
          title={t("topbar.widget")}
        >
          <LayoutGrid size={15} />
        </button>
        <button
          onClick={handleToggleTheme}
          className="chrome-button"
          title={t("topbar.theme")}
        >
          {theme === "dark" ? <Sun size={15} /> : <Moon size={15} />}
        </button>
        <button
          onClick={onRefresh}
          disabled={isLoading}
          className="chrome-button primary-action"
          title={t("topbar.refresh")}
        >
          <RefreshCw size={14} className={isLoading ? "animate-spin" : ""} />
          <span className="hidden sm:inline">{t("topbar.refresh")}</span>
        </button>
        <button
          onClick={onOpenSettings}
          className="chrome-button"
          title={t("topbar.settings")}
        >
          <Settings size={15} />
        </button>
        <a
          href="https://github.com/andywongpt-my/desktop-usage-helper"
          target="_blank"
          rel="noreferrer"
          className="chrome-button"
          title="GitHub"
        >
          <Github size={15} />
        </a>
        <div className="hidden h-6 w-px bg-white/10 sm:block" />
        <div className="hidden items-center gap-1.5 rounded-full border border-white/10 bg-white/[0.03] px-2.5 py-1 text-[11px] text-slate-400 xl:flex">
          <Bell size={12} className="text-slate-500" />
          <span>{t("topbar.tray_alerts")}</span>
          <PanelTopClose size={12} className="text-slate-500" />
        </div>
      </div>
    </header>
  );
}