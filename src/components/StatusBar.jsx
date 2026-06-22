import { useEffect, useState } from "react";
import { Activity, Clock3 } from "lucide-react";
import { useI18nStore } from "../stores/useI18nStore.js";
import pkg from "../../package.json";

function timeAgo(ts) {
  if (!ts) return "never";
  const s = Math.floor((Date.now() - ts) / 1000);
  if (s < 5) return "just now";
  if (s < 60) return `${s}s ago`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m}m ago`;
  const h = Math.floor(m / 60);
  return `${h}h ago`;
}

export default function StatusBar({ lastRefresh, loading }) {
  const [, setTick] = useState(0);
  const t = useI18nStore((s) => s.t);
  useEffect(() => {
    const timer = setInterval(() => setTick((n) => n + 1), 1000);
    return () => clearInterval(timer);
  }, []);

  return (
    <footer className="statusbar">
      <span className="inline-flex items-center gap-1.5">
        {loading ? <Activity size={12} className="animate-pulse text-slate-300" /> : <Clock3 size={12} />}
        {loading ? t("status.refreshing") : t("status.last_refresh", timeAgo(lastRefresh))}
      </span>
      <span className="hidden sm:inline">desktop-usage-helper v{pkg.version}</span>
    </footer>
  );
}