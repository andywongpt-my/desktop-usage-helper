import { useEffect, useMemo } from "react";
import { Activity, AlertTriangle, CheckCircle2, HelpCircle } from "lucide-react";
import { useUsageStore } from "../stores/useUsageStore.js";
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

export default function Dashboard({ onRefresh }) {
  const providers = useUsageStore((s) => s.getVisibleProviders());
  const isLoading = useUsageStore((s) => s.isLoading);

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

  if (providers.length === 0) {
    return <EmptyState />;
  }

  return (
    <div className="mx-auto flex w-full max-w-[1440px] flex-col gap-4">
      <section className="hero-panel">
        <div className="min-w-0">
          <p className="text-xs text-slate-500">Provider command center</p>
          <div className="mt-2 flex flex-col gap-2 lg:flex-row lg:items-end lg:justify-between">
            <div>
              <h2 className="text-2xl font-semibold tracking-tight text-white sm:text-3xl">
                Know what is safe to use next.
              </h2>
              <p className="mt-2 max-w-2xl text-sm leading-6 text-slate-400">
                A compact desktop view for balances, reset windows, latency and threshold alerts.
              </p>
            </div>
            {isLoading && (
              <div className="sync-chip">
                <Activity size={14} className="animate-pulse" />
                <span>syncing providers</span>
              </div>
            )}
          </div>
        </div>
        <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
          <SummaryTile icon={AlertTriangle} label="critical" value={counts.danger} tone="danger" />
          <SummaryTile icon={AlertTriangle} label="low" value={counts.warn} tone="warn" />
          <SummaryTile icon={CheckCircle2} label="healthy" value={counts.ok} tone="ok" />
          <SummaryTile icon={HelpCircle} label="unknown" value={counts.unknown} tone="neutral" />
        </div>
      </section>

      <section className="provider-grid">
        {providers.map((p) => (
          <ProviderCard key={p.id} provider={p} />
        ))}
      </section>
    </div>
  );
}
