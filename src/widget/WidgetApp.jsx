import { useEffect, useState } from "react";
import { useUsageStore } from "../stores/useUsageStore.js";

/**
 * Compact widget-mode app for always-on-top mini window.
 * Shows only the key numbers in a small strip.
 */
export default function WidgetApp() {
  const statuses = useUsageStore((s) => s.statuses);
  const providers = useUsageStore((s) => s.providers);
  const [tick, setTick] = useState(0);

  useEffect(() => {
    const t = setInterval(() => setTick((n) => n + 1), 5000);
    return () => clearInterval(t);
  }, []);

  const visible = providers.filter((p) => p.enabled);
  const items = visible.map((p) => {
    const status = statuses[p.id];
    const metric = status?.primary;
    const remaining =
      metric && metric.limit > 0
        ? Math.round(((metric.limit - metric.used) / metric.limit) * 100)
        : null;
    const state = status?.state ?? "unknown";
    return { id: p.id, label: p.label, remaining, state };
  });

  const stateColor = {
    ok: "#4ade80",
    warn: "#fbbf24",
    danger: "#f87171",
    unknown: "#94a3b8",
  };

  return (
    <div className="widget-root">
      <div className="widget-header">
        <span className="widget-dot" />
        <span className="widget-title">Usage Helper</span>
      </div>
      <div className="widget-grid">
        {items.length === 0 ? (
          <div className="widget-empty">No providers</div>
        ) : (
          items.map((item) => (
            <div key={item.id} className="widget-item">
              <div className="widget-item-label">{item.label}</div>
              <div className="widget-item-value" style={{ color: stateColor[item.state] }}>
                {item.remaining != null ? `${item.remaining}%` : "—"}
              </div>
              <div
                className="widget-item-bar"
                style={{ width: `${item.remaining ?? 0}%`, background: stateColor[item.state] }}
              />
            </div>
          ))
        )}
      </div>
    </div>
  );
}