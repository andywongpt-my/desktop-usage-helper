import { Plug } from "lucide-react";

export default function EmptyState() {
  return (
    <div className="h-full flex items-center justify-center">
      <div className="text-center max-w-md p-8">
        <div className="w-16 h-16 rounded-2xl bg-gray-800 mx-auto mb-4 flex items-center justify-center">
          <Plug size={28} className="text-gray-500" />
        </div>
        <h2 className="text-lg font-medium text-gray-200 mb-2">
          No providers enabled
        </h2>
        <p className="text-sm text-gray-500 mb-4">
          Add an API key or enable a provider in Settings to start tracking
          your LLM usage. Keys can come from environment variables
          (<code className="text-gray-400">OLLAMA_API_KEY</code>,
          <code className="text-gray-400"> OPENCODE_ZEN_API_KEY</code>,
          <code className="text-gray-400"> MINIMAX_API_KEY</code>) or be entered manually.
        </p>
        <p className="text-xs text-gray-600">
          The app will detect which env vars are set automatically.
        </p>
      </div>
    </div>
  );
}
