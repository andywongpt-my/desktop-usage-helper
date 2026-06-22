import { useEffect, useMemo, useState } from "react";
import { Activity, AlertTriangle, CheckCircle2, HelpCircle, ChevronDown } from "lucide-react";
import { useUsageStore } from "../stores/useUsageStore.js";
import { useI18nStore } from "../stores/useI18nStore.js";
import { useConfigStore } from "../stores/useConfigStore.js";
import ProviderCard from "./ProviderCard.jsx";
import EmptyState from "./EmptyState.jsx";

function SummaryTile({ icon: Icon, label, value, tone }) {
  const toneClass = {
    ok: "text-emerald-200 bg-emerald-300/10 border-emerald-300/15",
    warn: "text-amber-200 bg-amber-300/10 border-amber-300/15",
    danger: "text-rose-200 bg-rose-300/10 border-rose-300/15",
    neutral: "text-slate-300 bg-white/[0.04] border-white/10",
  }[tone];

  return (
    <div className={`summary-tile ${toneClass}`}>
      <Icon size={16} />
      <div>
        <p className="font-mono text-lg leading-none text-white">{value}</p>
        <p className="mt-1 text-[11px] text-slate-500">{label}</p>
      </div>
    </div>
  );
}

function ProviderGroup({ tag, providers, onRefresh }) {
  const [collapsed, setCollapsed] = useState(false);
  const t = useI18nStore((s) => s.t);

  const worstState = providers.reduce((worst, p) => {
    const order = { danger: 0, warn: 1, ok: 2, unknown: 3 };
    const s = p.status?.state ?? "unknown";
    return order[s] < order[worst] ? s : worst;
  }, "ok");

  const stateColor = {
    ok: "text-emerald-300",
    warn: "text-amber-300",
    danger: "text-rose-300",
    unknown: "text-slate-500",
  };

  return (
    <div className="space-y-4">
      {tag !== "__ungrouped" && (
        <button
          onClick={() => setCollapsed(!collapsed)}
          className="group flex w-full items-center gap-2 text-left"
        >
          <ChevronDown
            size={16}
            className={`text-slate-500 transition-transform ${collapsed ? "-rotate-90" : ""}`}
          />
          <span className="text-sm font-semibold text-white">{tag}</span>
          <span className="text-xs text-slate-500">({providers.length})</span>
          <span className={`ml-1 h-2 w-2 rounded-full ${stateColor[worstState]?.replace("text-", "bg-")}`} />
          <div className="flex-1 border-t border-white/10" />
        </button>
      )}
      {!collapsed && (
        <section className="provider-grid">
          {providers.map((p) => (
            <ProviderCard key={p.id} provider={p} />
          ))}
        </section>
      )}
    </div>
  );
}

export default function Dashboard({ onRefresh }) {
  const providers = useUsageStore((s) => s.getVisibleProviders());
  const isLoading = useUsageStore((s) => s.isLoading);
  const config = useConfigStore((s) => s.config);
  const t = useI18nStore((s) => s.t);

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

  const counts = useMemo(() => {
    return providers.reduce(
      (acc, p) => {
        const state = p.status?.state ?? "unknown";
        if (state === "danger") acc.danger += 1;
        else if (state === "warn") acc.warn += 1;
        else if (state === "ok") acc.ok += 1;
        else acc.unknown += 1;
        return acc;
      },
      { ok: 0, warn: 0, danger: 0, unknown: 0 }
    );
  }, [providers]);

  // Group providers by first tag (ungrouped → "__ungrouped")
  const grouped = useMemo(() => {
    const groups = {};
    for (const p of providers) {
      const userCfg = config.providers?.[p.id] ?? {};
      const tags = userCfg.tags ?? p.status?.tags ?? [];
      const groupKey = tags.length > 0 ? tags[0] : "__ungrouped";
      if (!groups[groupKey]) groups[groupKey] = [];
      groups[groupKey].push(p);
    }
    return groups;
  }, [providers, config.providers]);

  if (providers.length === 0) {
    return <EmptyState />;
  }

  const groupKeys = Object.keys(grouped).sort((a, b) => {
    if (a === "__ungrouped") return 1;
    if (b === "__ungrouped") return -1;
    return a.localeCompare(b);
  });

  return (
    <div className="mx-auto flex w-full max-w-[1440px] flex-col gap-4">
      <section className="hero-panel">
        <div className="min-w-0">
          <p className="text-xs text-slate-500">{t("dashboard.subtitle")}</p>
          <div className="mt-2 flex flex-col gap-2 lg:flex-row lg:items-end lg:justify-between">
            <div>
              <h2 className="text-2xl font-semibold tracking-tight text-white sm:text-3xl">
                {t("dashboard.title")}
              </h2>
              <p className="mt-2 max-w-2xl text-sm leading-6 text-slate-400">
                {t("dashboard.desc")}
              </p>
            </div>
            {isLoading && (
              <div className="sync-chip">
                <Activity size={14} className="animate-pulse" />
                <span>{t("dashboard.syncing")}</span>
              </div>
            )}
          </div>
        </div>
        <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
          <SummaryTile icon={AlertTriangle} label={t("dashboard.critical")} value={counts.danger} tone="danger" />
          <SummaryTile icon={AlertTriangle} label={t("dashboard.low")} value={counts.warn} tone="warn" />
          <SummaryTile icon={CheckCircle2} label={t("dashboard.healthy")} value={counts.ok} tone="ok" />
          <SummaryTile icon={HelpCircle} label={t("dashboard.unknown")} value={counts.unknown} tone="neutral" />
        </div>
      </section>

      {groupKeys.length <= 1 ? (
        <section className="provider-grid">
          {providers.map((p) => (
            <ProviderCard key={p.id} provider={p} />
          ))}
        </section>
      ) : (
        <div className="space-y-6">
          {groupKeys.map((key) => (
            <ProviderGroup key={key} tag={key} providers={grouped[key]} onRefresh={onRefresh} />
          ))}
        </div>
      )}
    </div>
  );
}