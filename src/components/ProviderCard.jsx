import { useState, useEffect, useCallback } from "react";
import { CheckCircle2, AlertTriangle, XCircle, HelpCircle, Clock, Activity, ChevronDown, TrendingUp, DollarSign, TimerReset, ExternalLink } from "lucide-react";
import TrendChart from "./TrendChart.jsx";
import { useUsageStore } from "../stores/useUsageStore.js";
import { useI18nStore } from "../stores/useI18nStore.js";
import { getHistory, openUrl } from "../lib/tauri.js";

const STATE_STYLES = {
  ok: {
    icon: CheckCircle2,
    iconColor: "text-emerald-300",
    fillClass: "progress-fill-ok",
    badge: "text-emerald-200 bg-emerald-300/10 border-emerald-300/20",
    rail: "from-emerald-300/60 to-emerald-300/0",
    label: "card.healthy",
  },
  warn: {
    icon: AlertTriangle,
    iconColor: "text-amber-300",
    fillClass: "progress-fill-warn",
    badge: "text-amber-200 bg-amber-300/10 border-amber-300/20",
    rail: "from-amber-300/70 to-amber-300/0",
    label: "card.low",
  },
  danger: {
    icon: XCircle,
    iconColor: "text-rose-300",
    fillClass: "progress-fill-danger",
    badge: "text-rose-200 bg-rose-300/10 border-rose-300/20",
    rail: "from-rose-300/70 to-rose-300/0",
    label: "card.critical",
  },
  unknown: {
    icon: HelpCircle,
    iconColor: "text-slate-500",
    fillClass: "progress-fill-info",
    badge: "text-slate-300 bg-white/[0.04] border-white/10",
    rail: "from-slate-400/30 to-slate-400/0",
    label: "card.unknown",
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

/// Format the absolute clock time for a reset timestamp.
/// Returns strings like "Today 3:30 PM", "Tomorrow 9:00 AM", "Jun 25 12:00 AM".
function formatResetClock(resetAt, t) {
  if (!resetAt) return null;
  const date = new Date(resetAt);
  const now = new Date();
  const isToday = date.toDateString() === now.toDateString();
  const tomorrow = new Date(now);
  tomorrow.setDate(tomorrow.getDate() + 1);
  const isTomorrow = date.toDateString() === tomorrow.toDateString();

  const timeStr = date.toLocaleTimeString(undefined, {
    hour: "numeric",
    minute: "2-digit",
  });

  if (isToday) return `${t("card.reset_today")} ${timeStr}`;
  if (isTomorrow) return `${t("card.reset_tomorrow")} ${timeStr}`;
  const monthDay = date.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
  });
  return `${monthDay} ${timeStr}`;
}

/// Get the soonest reset timestamp from primary + secondary metrics.
function getSoonestReset(status) {
  const candidates = [];
  if (status?.primary?.resetAt) candidates.push(status.primary.resetAt);
  if (status?.secondary?.resetAt) candidates.push(status.secondary.resetAt);
  if (candidates.length === 0) return null;
  return Math.min(...candidates);
}

function MetricBar({ metric, t }) {
  if (!metric) return null;
  const p = pct(metric.used, metric.limit);
  // p = used% (0-100). High used% = danger (red), low used% = ok (green).
  const fillClass = p == null ? "progress-fill-info" :
    p >= 90 ? "progress-fill-danger" :
    p >= 60 ? "progress-fill-warn" : "progress-fill-ok";
  const resetClock = formatResetClock(metric.resetAt, t);
  return (
    <div className="metric-block">
      <div className="flex items-start justify-between gap-3 text-xs">
        <div>
          <p className="text-slate-300">{metric.label}</p>
          {metric.resetAt && (
            <div className="mt-1 flex flex-col gap-0.5 text-[10px] text-slate-500">
              <span className="flex items-center gap-1">
                <Clock size={10} />
                <span>{t("card.resets_in", timeUntilReset(metric.resetAt))}</span>
              </span>
              {resetClock && (
                <span className="flex items-center gap-1 pl-[14px] text-slate-600">
                  <TimerReset size={10} />
                  <span>{resetClock}</span>
                </span>
              )}
            </div>
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
  const [showTrend, setShowTrend] = useState(false);
  const [trendData, setTrendData] = useState(null);
  const [trendRange, setTrendRange] = useState(24); // hours
  const status = provider.status;
  const state = status?.state ?? "unknown";
  const style = STATE_STYLES[state];
  const Icon = style.icon;
  const t = useI18nStore((s) => s.t);
  const setHistory = useUsageStore((s) => s.setHistory);
  const historyCache = useUsageStore((s) => s.historyCache);

  const hasError = !!status?.error;
  const primaryMetric = status?.primary;
  const secondaryMetric = status?.secondary;
  const costEstimate = status?.costEstimate;

  // Soonest reset across all metrics
  const soonestReset = getSoonestReset(status);
  const resetMs = soonestReset ? soonestReset - Date.now() : null;
  const isResetNear = resetMs != null && resetMs > 0 && resetMs < 2 * 60 * 60 * 1000; // < 2h
  const isResetToday = soonestReset && new Date(soonestReset).toDateString() === new Date().toDateString();

  const loadTrend = useCallback(async (hours) => {
    const range = hours ?? trendRange;
    const cached = historyCache[provider.id];
    if (cached && Date.now() - cached.fetchedAt < 60_000) {
      setTrendData(cached.points);
      return;
    }
    try {
      const points = await getHistory(provider.id, range);
      setTrendData(points);
      setHistory(provider.id, points);
    } catch (e) {
      console.error("[ProviderCard] getHistory failed:", e);
    }
  }, [provider.id, trendRange, historyCache, setHistory]);

  useEffect(() => {
    if (showTrend && !trendData) {
      loadTrend();
    }
  }, [showTrend, trendData, loadTrend]);

  const handleTrendToggle = () => {
    if (!showTrend && !trendData) {
      loadTrend();
    }
    setShowTrend(!showTrend);
  };

  const handleRangeChange = (hours) => {
    setTrendRange(hours);
    setTrendData(null);
    if (showTrend) loadTrend(hours);
  };

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
              {status?.accountLabel && (
                <span className="ml-1 text-slate-600">· {status.accountLabel}</span>
              )}
            </p>
          </div>
        </div>
        <span className={`shrink-0 rounded-full border px-2.5 py-1 text-[10px] font-medium ${style.badge}`}>
          {t(style.label)}
        </span>
      </div>

      {hasError ? (
        <div className="mt-4 rounded-xl border border-rose-300/15 bg-rose-300/10 p-3 text-xs leading-5 text-rose-100 break-words">
          {status.error}
        </div>
      ) : (
        <div className="mt-4 space-y-3">
          <MetricBar metric={primaryMetric} t={t} />
          {secondaryMetric && <MetricBar metric={secondaryMetric} t={t} />}
          {costEstimate != null && (
            <div className="flex items-center gap-2 text-[11px] text-slate-400">
              <DollarSign size={12} className="text-emerald-400" />
              <span>{t("card.estimate", costEstimate.toFixed(2))}</span>
            </div>
          )}
        </div>
      )}

      <div className="mt-4 flex items-center justify-between border-t border-white/10 pt-3">
        <div className="flex items-center gap-1.5 text-[10px] text-slate-500">
          <Activity size={11} />
          <span className="font-mono">{status?.latencyMs != null ? `${status.latencyMs}ms` : "-"}</span>
          {soonestReset && (
            <span className={`ml-2 inline-flex items-center gap-1 rounded px-1.5 py-0.5 ${
              isResetNear
                ? "bg-amber-300/15 text-amber-200"
                : isResetToday
                  ? "bg-sky-300/10 text-sky-200"
                  : "text-slate-600"
            }`}>
              <TimerReset size={10} />
              <span>{t("card.resets_in", timeUntilReset(soonestReset))}</span>
            </span>
          )}
        </div>
        <div className="flex items-center gap-2">
          {provider.docs_url && (
            <button
              onClick={() => openUrl(provider.docs_url)}
              className="inline-flex items-center gap-1 rounded-md px-1.5 py-1 text-[10px] text-slate-500 transition-colors hover:bg-white/[0.04] hover:text-slate-200 active:scale-[0.98]"
              title={provider.docs_url}
            >
              <ExternalLink size={12} />
              {t("card.dashboard")}
            </button>
          )}
          <button
            onClick={handleTrendToggle}
            className={`inline-flex items-center gap-1 rounded-md px-1.5 py-1 text-[10px] transition-colors hover:bg-white/[0.04] hover:text-slate-200 active:scale-[0.98] ${showTrend ? "text-accent" : "text-slate-500"}`}
          >
            <TrendingUp size={12} />
            {t("card.trend")}
          </button>
          <button
            onClick={() => setExpanded(!expanded)}
            className="inline-flex items-center gap-1 rounded-md px-1.5 py-1 text-[10px] text-slate-500 transition-colors hover:bg-white/[0.04] hover:text-slate-200 active:scale-[0.98]"
          >
            <ChevronDown size={12} className={expanded ? "rotate-180 transition-transform" : "transition-transform"} />
            {t("card.details")}
          </button>
        </div>
      </div>

      {showTrend && (
        <div className="mt-3 rounded-xl border border-white/10 bg-slate-950/35 p-3">
          <div className="mb-2 flex items-center gap-2 text-[10px] text-slate-500">
            {[
              { h: 1, label: "1h" },
              { h: 6, label: "6h" },
              { h: 24, label: "24h" },
              { h: 168, label: "7d" },
            ].map(({ h, label }) => (
              <button
                key={h}
                onClick={() => handleRangeChange(h)}
                className={`rounded px-1.5 py-0.5 ${trendRange === h ? "bg-accent/20 text-accent" : "hover:bg-white/[0.04]"}`}
              >
                {label}
              </button>
            ))}
          </div>
          {trendData ? (
            <TrendChart points={trendData} />
          ) : (
            <div className="flex h-[60px] items-center justify-center text-[10px] text-slate-600">
              Loading...
            </div>
          )}
        </div>
      )}

      {expanded && status && (
        <pre className="mt-3 max-h-40 overflow-auto rounded-xl border border-white/10 bg-slate-950/70 p-3 font-mono text-[10px] leading-4 text-slate-400">
          {JSON.stringify(status, null, 2)}
        </pre>
      )}
    </article>
  );
}