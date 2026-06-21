import { useState } from "react";
import { CheckCircle2, AlertTriangle, XCircle, HelpCircle, Clock, Activity } from "lucide-react";

const STATE_STYLES = {
  ok: {
    icon: CheckCircle2,
    iconColor: "text-ok",
    fillClass: "progress-fill-ok",
    badge: "text-emerald-400 bg-emerald-400/10 border-emerald-400/20",
    label: "Healthy",
  },
  warn: {
    icon: AlertTriangle,
    iconColor: "text-warn",
    fillClass: "progress-fill-warn",
    badge: "text-amber-400 bg-amber-400/10 border-amber-400/20",
    label: "Low",
  },
  danger: {
    icon: XCircle,
    iconColor: "text-danger",
    fillClass: "progress-fill-danger",
    badge: "text-rose-400 bg-rose-400/10 border-rose-400/20",
    label: "Critical",
  },
  unknown: {
    icon: HelpCircle,
    iconColor: "text-gray-500",
    fillClass: "progress-fill-info",
    badge: "text-gray-400 bg-gray-400/10 border-gray-400/20",
    label: "Unknown",
  },
};

function formatNumber(n) {
  if (n == null) return "—";
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
  if (ms <= 0) return "resetting…";
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
    <div className="space-y-1.5">
      <div className="flex items-baseline justify-between text-xs">
        <span className="text-gray-400">{metric.label}</span>
        <span className="font-mono text-gray-200">
          {formatNumber(metric.used)} / {formatNumber(metric.limit)}
          {metric.unit ? <span className="text-gray-500 ml-1">{metric.unit}</span> : null}
        </span>
      </div>
      <div className="progress-track">
        <div
          className={`progress-fill ${fillClass}`}
          style={{ width: `${p ?? 100}%` }}
        />
      </div>
      {metric.resetAt && (
        <div className="flex items-center gap-1 text-[10px] text-gray-500 font-mono">
          <Clock size={10} />
          <span>resets in {timeUntilReset(metric.resetAt)}</span>
        </div>
      )}
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
    <div className="card card-hover">
      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center gap-2 min-w-0">
          <Icon size={16} className={style.iconColor} />
          <div className="min-w-0">
            <h3 className="text-sm font-medium text-gray-100 truncate">
              {provider.label}
            </h3>
            <p className="text-[10px] text-gray-500 uppercase tracking-wider">
              {provider.kind}
            </p>
          </div>
        </div>
        <span className={`px-2 py-0.5 rounded-full border text-[10px] font-medium ${style.badge}`}>
          {style.label}
        </span>
      </div>

      {hasError ? (
        <div className="text-xs text-danger bg-rose-500/10 border border-rose-500/20 rounded-md p-2 mb-2 break-words">
          {status.error}
        </div>
      ) : (
        <div className="space-y-3">
          <MetricBar metric={primaryMetric} />
          {secondaryMetric && <MetricBar metric={secondaryMetric} />}
        </div>
      )}

      <div className="flex items-center justify-between mt-3 pt-3 border-t border-gray-800">
        <div className="flex items-center gap-1 text-[10px] text-gray-500 font-mono">
          <Activity size={10} />
          <span>{status?.latencyMs != null ? `${status.latencyMs}ms` : "—"}</span>
        </div>
        <button
          onClick={() => setExpanded(!expanded)}
          className="text-[10px] text-gray-500 hover:text-gray-300 font-mono"
        >
          {expanded ? "−" : "+"} details
        </button>
      </div>

      {expanded && status && (
        <pre className="mt-3 text-[10px] text-gray-500 bg-gray-950 rounded p-2 overflow-auto max-h-40 font-mono">
          {JSON.stringify(status, null, 2)}
        </pre>
      )}
    </div>
  );
}
