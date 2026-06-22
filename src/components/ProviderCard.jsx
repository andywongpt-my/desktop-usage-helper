import { useState } from "react";
import { CheckCircle2, AlertTriangle, XCircle, HelpCircle, Clock, Activity, ChevronDown } from "lucide-react";

const STATE_STYLES = {
  ok: {
    icon: CheckCircle2,
    iconColor: "text-emerald-300",
    fillClass: "progress-fill-ok",
    badge: "text-emerald-200 bg-emerald-300/10 border-emerald-300/20",
    rail: "from-emerald-300/60 to-emerald-300/0",
    label: "Healthy",
  },
  warn: {
    icon: AlertTriangle,
    iconColor: "text-amber-300",
    fillClass: "progress-fill-warn",
    badge: "text-amber-200 bg-amber-300/10 border-amber-300/20",
    rail: "from-amber-300/70 to-amber-300/0",
    label: "Low",
  },
  danger: {
    icon: XCircle,
    iconColor: "text-rose-300",
    fillClass: "progress-fill-danger",
    badge: "text-rose-200 bg-rose-300/10 border-rose-300/20",
    rail: "from-rose-300/70 to-rose-300/0",
    label: "Critical",
  },
  unknown: {
    icon: HelpCircle,
    iconColor: "text-slate-500",
    fillClass: "progress-fill-info",
    badge: "text-slate-300 bg-white/[0.04] border-white/10",
    rail: "from-slate-400/30 to-slate-400/0",
    label: "Unknown",
  },
};

function formatNumber(n) {
  if (n == null) return "-";
  if (Math.abs(n) >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (Math.abs(n) >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
  return String(Math.round(n * 100) / 100);
}

function pct(used, limit) {
  if (!limit || limit <= 0) return null;
  return Math.min(100, Math.max(0, (used / limit) * 100));
}

function timeUntilReset(resetAt) {
  if (!resetAt) return null;
  const ms = resetAt - Date.now();
  if (ms <= 0) return "resetting";
  const min = Math.floor(ms / 60_000);
  if (min < 60) return `${min}m`;
  const h = Math.floor(min / 60);
  if (h < 48) return `${h}h ${min % 60}m`;
  const d = Math.floor(h / 24);
  return `${d}d ${h % 24}h`;
}

function MetricBar({ metric }) {
  if (!metric) return null;
  const p = pct(metric.used, metric.limit);
  const fillClass = p == null ? "progress-fill-info" :
    p < 30 ? "progress-fill-danger" :
    p < 60 ? "progress-fill-warn" : "progress-fill-ok";
  return (
    <div className="metric-block">
      <div className="flex items-start justify-between gap-3 text-xs">
        <div>
          <p className="text-slate-300">{metric.label}</p>
          {metric.resetAt && (
            <p className="mt-1 flex items-center gap-1 text-[10px] text-slate-500">
              <Clock size={10} />
              <span>resets in {timeUntilReset(metric.resetAt)}</span>
            </p>
          )}
        </div>
        <span className="shrink-0 font-mono text-slate-100">
          {formatNumber(metric.used)} / {formatNumber(metric.limit)}
          {metric.unit ? <span className="ml-1 text-slate-500">{metric.unit}</span> : null}
        </span>
      </div>
      <div className="progress-track mt-2">
        <div
          className={`progress-fill ${fillClass}`}
          style={{ width: `${p ?? 100}%` }}
        />
      </div>
    </div>
  );
}

export default function ProviderCard({ provider }) {
  const [expanded, setExpanded] = useState(false);
  const status = provider.status;
  const state = status?.state ?? "unknown";
  const style = STATE_STYLES[state];
  const Icon = style.icon;

  const hasError = !!status?.error;
  const primaryMetric = status?.primary;
  const secondaryMetric = status?.secondary;

  return (
    <article className="provider-card">
      <div className={`absolute inset-x-0 top-0 h-px bg-gradient-to-r ${style.rail}`} />
      <div className="flex items-start justify-between gap-3">
        <div className="flex min-w-0 items-center gap-3">
          <div className="provider-icon">
            <Icon size={17} className={style.iconColor} />
          </div>
          <div className="min-w-0">
            <h3 className="truncate text-sm font-semibold text-white">
              {provider.label}
            </h3>
            <p className="mt-0.5 truncate text-[11px] text-slate-500">
              {provider.kind}
            </p>
          </div>
        </div>
        <span className={`rounded-full border px-2.5 py-1 text-[10px] font-medium ${style.badge}`}>
          {style.label}
        </span>
      </div>

      {hasError ? (
        <div className="mt-4 rounded-xl border border-rose-300/15 bg-rose-300/10 p-3 text-xs leading-5 text-rose-100 break-words">
          {status.error}
        </div>
      ) : (
        <div className="mt-4 space-y-3">
          <MetricBar metric={primaryMetric} />
          {secondaryMetric && <MetricBar metric={secondaryMetric} />}
        </div>
      )}

      <div className="mt-4 flex items-center justify-between border-t border-white/10 pt-3">
        <div className="flex items-center gap-1.5 text-[10px] text-slate-500">
          <Activity size={11} />
          <span className="font-mono">{status?.latencyMs != null ? `${status.latencyMs}ms` : "-"}</span>
        </div>
        <button
          onClick={() => setExpanded(!expanded)}
          className="inline-flex items-center gap-1 rounded-md px-1.5 py-1 text-[10px] text-slate-500 transition-colors hover:bg-white/[0.04] hover:text-slate-200 active:scale-[0.98]"
        >
          <ChevronDown size={12} className={expanded ? "rotate-180 transition-transform" : "transition-transform"} />
          details
        </button>
      </div>

      {expanded && status && (
        <pre className="mt-3 max-h-40 overflow-auto rounded-xl border border-white/10 bg-slate-950/70 p-3 font-mono text-[10px] leading-4 text-slate-400">
          {JSON.stringify(status, null, 2)}
        </pre>
      )}
    </article>
  );
}
