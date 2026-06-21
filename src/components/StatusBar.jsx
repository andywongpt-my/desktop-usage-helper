import { useEffect, useState } from "react";

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
  useEffect(() => {
    const t = setInterval(() => setTick((n) => n + 1), 1000);
    return () => clearInterval(t);
  }, []);

  return (
    <footer className="flex items-center justify-between px-5 py-1.5 border-t border-gray-800 bg-gray-900/40 text-[11px] text-gray-500 font-mono">
      <span>
        {loading ? "refreshing…" : `last refresh: ${timeAgo(lastRefresh)}`}
      </span>
      <span>desktop-usage-helper v0.1.0</span>
    </footer>
  );
}
