import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { ArrowLeft, Plug } from "lucide-react";
import { UsageBar } from "../components/UsageBar";
import { SessionRow } from "../components/SessionList";
import { api } from "../lib/tauri";
import { useAppStore } from "../stores/appStore";
import type { AISession, ModelUsage, ProviderUsage } from "../types";
import { formatResetTime, providerLabel } from "../lib/utils";

export function ProviderDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { connectProvider, loadAuth } = useAppStore();
  const [provider, setProvider] = useState<ProviderUsage | null>(null);
  const [models, setModels] = useState<ModelUsage[]>([]);
  const [sessions, setSessions] = useState<AISession[]>([]);
  const [connecting, setConnecting] = useState(false);

  useEffect(() => {
    if (!id) return;
    api.getProviderUsage(id).then(setProvider);
    api.getModelUsage(id).then(setModels);
    api.getSessions(id).then(setSessions);
  }, [id]);

  const handleConnect = async () => {
    if (!id) return;
    setConnecting(true);
    try {
      await connectProvider(id);
      const updated = await api.getProviderUsage(id);
      setProvider(updated);
      await loadAuth();
    } finally {
      setConnecting(false);
    }
  };

  if (!provider) {
    return <div className="p-6 text-zinc-500">Loading…</div>;
  }

  return (
    <div className="p-6">
      <Link
        to="/providers"
        className="mb-4 inline-flex items-center gap-1 text-sm text-zinc-500 hover:text-zinc-300"
      >
        <ArrowLeft className="h-4 w-4" /> Providers
      </Link>

      <div className="mb-6 flex items-start justify-between">
        <div>
          <h1 className="text-xl font-semibold">{provider.name}</h1>
          <p className="text-sm capitalize text-zinc-500">
            {provider.auth_status.replace("not", "not ")}
          </p>
        </div>
        {provider.auth_status !== "connected" && provider.installed && (
          <button
            onClick={handleConnect}
            disabled={connecting}
            className="inline-flex items-center gap-2 rounded-lg bg-emerald-600 px-3 py-1.5 text-sm text-white hover:bg-emerald-500 disabled:opacity-50"
          >
            <Plug className="h-4 w-4" />
            Connect {provider.name}
          </button>
        )}
      </div>

      <div className="mb-8 max-w-lg space-y-4 rounded-xl border border-zinc-800 bg-zinc-900/40 p-4">
        <UsageBar
          percentage={provider.five_hour?.percentage}
          label={provider.five_hour?.label ?? "5 hour usage"}
          resetText={formatResetTime(provider.five_hour?.resets_at)}
          unavailable={!provider.usage_available}
        />
        {provider.seven_day?.percentage != null && (
          <UsageBar
            percentage={provider.seven_day.percentage}
            label={provider.seven_day.label ?? "7 day usage"}
            resetText={formatResetTime(provider.seven_day.resets_at)}
          />
        )}
      </div>

      <section className="mb-8">
        <h2 className="mb-3 text-sm font-medium text-zinc-300">Models</h2>
        <div className="grid gap-3 md:grid-cols-2">
          {models.map((m) => (
            <Link
              key={m.model}
              to={`/models/${encodeURIComponent(m.model)}?provider=${id}`}
              className="rounded-lg border border-zinc-800/60 bg-zinc-900/30 px-4 py-3 hover:border-zinc-700"
            >
              <div className="mb-2 text-sm text-zinc-200">{m.model}</div>
              <UsageBar percentage={m.percentage} size="sm" />
            </Link>
          ))}
          {models.length === 0 && (
            <p className="text-sm text-zinc-500">No model data yet</p>
          )}
        </div>
      </section>

      <section>
        <h2 className="mb-3 text-sm font-medium text-zinc-300">Sessions</h2>
        <div className="rounded-xl border border-zinc-800/80 bg-zinc-900/30 px-4">
          {sessions.slice(0, 15).map((s) => (
            <SessionRow key={s.id} session={s} />
          ))}
          {sessions.length === 0 && (
            <p className="py-6 text-center text-sm text-zinc-500">
              No sessions for {providerLabel(id ?? "")}
            </p>
          )}
        </div>
      </section>
    </div>
  );
}
