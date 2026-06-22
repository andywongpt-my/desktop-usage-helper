import { Plug, Settings } from "lucide-react";
import { useI18nStore } from "../stores/useI18nStore.js";

export default function EmptyState() {
  const t = useI18nStore((s) => s.t);
  return (
    <div className="mx-auto flex min-h-[calc(100dvh-140px)] w-full max-w-[900px] items-center justify-center">
      <div className="empty-panel">
        <div className="empty-icon">
          <Plug size={30} />
        </div>
        <h2 className="mt-5 text-xl font-semibold tracking-tight text-white">
          {t("empty.title")}
        </h2>
        <p className="mx-auto mt-3 max-w-xl text-sm leading-6 text-slate-400">
          {t("empty.desc")}
        </p>
        <div className="mt-5 flex flex-wrap justify-center gap-2 text-xs text-slate-500">
          <code className="env-pill">OLLAMA_API_KEY</code>
          <code className="env-pill">OPENCODE_ZEN_API_KEY</code>
          <code className="env-pill">MINIMAX_API_KEY</code>
        </div>
        <div className="mt-6 inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/[0.04] px-4 py-2 text-xs text-slate-300">
          <Settings size={14} />
          {t("empty.open_settings")}
        </div>
      </div>
    </div>
  );
}
