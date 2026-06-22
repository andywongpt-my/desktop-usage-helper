import { useMemo } from "react";

/**
 * Inline SVG sparkline for usage trend.
 * Pure component — props: { points: [{timestamp, used, limit}], width, height }
 */
export default function TrendChart({ points, width = 240, height = 60 }) {
  const { pathD, areaD, lastPct } = useMemo(() => {
    if (!points || points.length < 2) {
      return { pathD: "", areaD: "", lastPct: null };
    }

    const w = width;
    const h = height;
    const pad = 4;

    const xs = points.map((_, i) => {
      if (points.length === 1) return w / 2;
      return pad + (i / (points.length - 1)) * (w - 2 * pad);
    });

    // Y = remaining% (0-100), inverted
    const ys = points.map((p) => {
      if (!p.limit || p.limit <= 0) return h - pad;
      const remaining = ((p.limit - p.used) / p.limit) * 100;
      return h - pad - (remaining / 100) * (h - 2 * pad);
    });

    const pathD = xs
      .map((x, i) => `${i === 0 ? "M" : "L"} ${x.toFixed(1)} ${ys[i].toFixed(1)}`)
      .join(" ");

    const areaD = `${pathD} L ${xs[xs.length - 1].toFixed(1)} ${h - pad} L ${xs[0].toFixed(1)} ${h - pad} Z`;

    const last = points[points.length - 1];
    const lastPct =
      last && last.limit > 0
        ? Math.round(((last.limit - last.used) / last.limit) * 100)
        : null;

    return { pathD, areaD, lastPct };
  }, [points, width, height]);

  if (!points || points.length < 2) {
    return (
      <div className="flex items-center justify-center text-[10px] text-slate-600" style={{ height }}>
        No trend data yet
      </div>
    );
  }

  const color = lastPct == null ? "#60a5fa" :
    lastPct < 30 ? "#f87171" :
    lastPct < 60 ? "#fbbf24" : "#4ade80";

  return (
    <svg width={width} height={height} className="block w-full" viewBox={`0 0 ${width} ${height}`}>
      <defs>
        <linearGradient id={`grad-${color.slice(1)}`} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={color} stopOpacity="0.25" />
          <stop offset="100%" stopColor={color} stopOpacity="0.02" />
        </linearGradient>
      </defs>
      <path d={areaD} fill={`url(#grad-${color.slice(1)})`} />
      <path d={pathD} fill="none" stroke={color} strokeWidth="1.5" strokeLinejoin="round" strokeLinecap="round" />
      {lastPct != null && (
        <text x={width - 4} y={12} textAnchor="end" className="font-mono" fontSize="10" fill={color}>
          {lastPct}%
        </text>
      )}
    </svg>
  );
}