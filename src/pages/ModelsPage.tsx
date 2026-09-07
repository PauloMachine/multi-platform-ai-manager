import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { UsageBar } from "../components/UsageBar";
import { api } from "../lib/tauri";
import type { ModelUsage } from "../types";

export function ModelsPage() {
  const [models, setModels] = useState<ModelUsage[]>([]);

  useEffect(() => {
    api.getModelUsage().then(setModels);
  }, []);

  return (
    <div className="p-6">
      <h1 className="mb-1 text-xl font-semibold">Models</h1>
      <p className="mb-6 text-sm text-zinc-500">
        Usage breakdown by model across all providers
      </p>
      <div className="grid gap-3 md:grid-cols-2 lg:grid-cols-3">
        {models.map((m) => (
          <Link
            key={m.model}
            to={`/models/${encodeURIComponent(m.model)}`}
            className="rounded-xl border border-zinc-800/60 bg-zinc-900/30 p-4 hover:border-zinc-700"
          >
            <h3 className="mb-2 text-sm font-medium text-zinc-200">{m.model}</h3>
            <UsageBar percentage={m.percentage} size="sm" />
            <p className="mt-2 text-xs text-zinc-500">
              {m.session_count} sessions
              {m.tokens != null && ` · ${m.tokens.toLocaleString()} tokens`}
            </p>
          </Link>
        ))}
        {models.length === 0 && (
          <p className="text-sm text-zinc-500">No model usage recorded yet</p>
        )}
      </div>
    </div>
  );
}
