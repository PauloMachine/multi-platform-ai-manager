import { RefreshCw } from "lucide-react";
import { ProviderCard } from "../components/ProviderCard";
import { ActiveSessionCard, SessionRow } from "../components/SessionList";
import { UsageBar } from "../components/UsageBar";
import { useAppStore } from "../stores/appStore";
import { providerLabel, formatDateTime } from "../lib/utils";

export function DashboardPage() {
  const { dashboard, refresh, refreshing } = useAppStore();

  if (!dashboard) {
    return (
      <div className="flex h-full items-center justify-center text-zinc-500">
        Loading dashboard…
      </div>
    );
  }

  return (
    <div className="p-6">
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold text-zinc-100">Dashboard</h1>
          <p className="text-sm text-zinc-500">
            Monitor your AI coding tools locally
          </p>
        </div>
        <button
          onClick={() => refresh()}
          disabled={refreshing}
          className="inline-flex items-center gap-2 rounded-lg border border-zinc-700 bg-zinc-900 px-3 py-1.5 text-sm text-zinc-300 transition-colors hover:border-zinc-600 hover:bg-zinc-800 disabled:opacity-50"
        >
          <RefreshCw className={`h-4 w-4 ${refreshing ? "animate-spin" : ""}`} />
          Refresh
        </button>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        {dashboard.providers.map((p) => (
          <ProviderCard key={p.provider} provider={p} />
        ))}
      </div>

      <section className="mt-8">
        <h2 className="mb-3 text-sm font-medium text-zinc-300">
          Currently Running
        </h2>
        {dashboard.active_sessions.length === 0 ? (
          <p className="rounded-lg border border-dashed border-zinc-800 px-4 py-6 text-center text-sm text-zinc-500">
            No active AI sessions
          </p>
        ) : (
          <div className="space-y-2">
            {dashboard.active_sessions.map((s) => (
              <ActiveSessionCard key={s.id} session={s} />
            ))}
          </div>
        )}
      </section>

      <section className="mt-8">
        <h2 className="mb-3 text-sm font-medium text-zinc-300">Usage by Model</h2>
        <div className="grid gap-3 md:grid-cols-2">
          {dashboard.models.slice(0, 6).map((m) => (
            <div
              key={m.model}
              className="rounded-lg border border-zinc-800/60 bg-zinc-900/30 px-4 py-3"
            >
              <div className="mb-2 flex justify-between text-sm">
                <span className="text-zinc-300">{m.model}</span>
                <span className="font-mono text-xs text-zinc-500">
                  {m.session_count} sessions
                </span>
              </div>
              <UsageBar percentage={m.percentage} size="sm" />
            </div>
          ))}
          {dashboard.models.length === 0 && (
            <p className="text-sm text-zinc-500">No model usage data yet</p>
          )}
        </div>
      </section>

      <section className="mt-8">
        <h2 className="mb-3 text-sm font-medium text-zinc-300">
          Recent Sessions
        </h2>
        <div className="rounded-xl border border-zinc-800/80 bg-zinc-900/30 px-4">
          {dashboard.recent_sessions.slice(0, 8).map((s) => (
            <SessionRow key={s.id} session={s} />
          ))}
          {dashboard.recent_sessions.length === 0 && (
            <p className="py-6 text-center text-sm text-zinc-500">
              No sessions recorded yet. Hooks will capture activity automatically.
            </p>
          )}
        </div>
      </section>

      <section className="mt-8">
        <h2 className="mb-3 text-sm font-medium text-zinc-300">Recent Events</h2>
        <div className="rounded-xl border border-zinc-800/80 bg-zinc-900/30">
          {dashboard.recent_events.slice(-10).reverse().map((e) => (
            <div
              key={e.id}
              className="flex items-center gap-3 border-b border-zinc-800/40 px-4 py-2 text-xs last:border-0"
            >
              <span className="font-mono text-zinc-600">
                {formatDateTime(e.timestamp)}
              </span>
              <span className="text-zinc-500">{providerLabel(e.provider)}</span>
              <span className="text-zinc-400">{e.event_type}</span>
              {e.model && <span className="text-zinc-300">{e.model}</span>}
            </div>
          ))}
          {dashboard.recent_events.length === 0 && (
            <p className="px-4 py-6 text-center text-sm text-zinc-500">
              No events yet
            </p>
          )}
        </div>
      </section>
    </div>
  );
}
