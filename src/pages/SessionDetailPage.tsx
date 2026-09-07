import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { ArrowLeft } from "lucide-react";
import { CopySessionButton } from "../components/CopySessionButton";
import { StopSessionButton } from "../components/StopSessionButton";
import { api } from "../lib/tauri";
import type { AISessionDetails } from "../types";
import {
  formatDateTime,
  formatDuration,
  formatTime,
  providerLabel,
  statusColor,
} from "../lib/utils";

export function SessionDetailPage() {
  const { id } = useParams<{ id: string }>();
  const [details, setDetails] = useState<AISessionDetails | null>(null);

  useEffect(() => {
    if (!id) return;
    api.getSession(id).then(setDetails);
  }, [id]);

  if (!details) {
    return <div className="p-6 text-zinc-500">Loading session…</div>;
  }

  const { session } = details;

  return (
    <div className="p-6">
      <Link
        to="/sessions"
        className="mb-4 inline-flex items-center gap-1 text-sm text-zinc-500 hover:text-zinc-300"
      >
        <ArrowLeft className="h-4 w-4" /> Sessions
      </Link>

      <div className="mb-6 flex items-start justify-between gap-4">
        <div>
          <h1 className="text-xl font-semibold">{session.model ?? "Unknown model"}</h1>
          <p className="text-sm text-zinc-500">{providerLabel(session.provider)}</p>
        </div>
        <div className="flex items-center gap-2">
          {session.status === "running" && (
            <StopSessionButton
              sessionId={session.id}
              onStopped={() => {
                if (id) api.getSession(id).then(setDetails);
              }}
            />
          )}
          <CopySessionButton
            sessionId={session.id}
            currentProvider={session.provider}
          />
        </div>
      </div>

      <div className="mb-8 grid gap-4 md:grid-cols-2">
        <InfoRow label="Project" value={session.project ?? "—"} />
        <InfoRow
          label="Status"
          value={
            <span className={`capitalize ${statusColor(session.status)}`}>
              {session.status}
            </span>
          }
        />
        <InfoRow label="Started" value={formatDateTime(session.started_at)} />
        <InfoRow label="Duration" value={formatDuration(session.duration_secs)} />
        {session.tokens != null && (
          <InfoRow label="Tokens" value={session.tokens.toLocaleString()} />
        )}
        {session.cost != null && (
          <InfoRow label="Cost" value={`$${session.cost.toFixed(4)}`} />
        )}
        {session.conversation_id && (
          <InfoRow label="Session ID" value={session.conversation_id} mono />
        )}
      </div>

      {session.initial_prompt && (
        <section className="mb-8">
          <h2 className="mb-2 text-sm font-medium text-zinc-300">Task</h2>
          <p className="rounded-lg border border-zinc-800 bg-zinc-900/40 p-4 text-sm text-zinc-300">
            {session.initial_prompt}
          </p>
        </section>
      )}

      {details.tools_used.length > 0 && (
        <section className="mb-8">
          <h2 className="mb-2 text-sm font-medium text-zinc-300">Tools Used</h2>
          <div className="flex flex-wrap gap-2">
            {details.tools_used.map((t) => (
              <span
                key={t}
                className="rounded-md bg-zinc-800 px-2 py-1 font-mono text-xs text-zinc-400"
              >
                {t}
              </span>
            ))}
          </div>
        </section>
      )}

      {details.files_modified.length > 0 && (
        <section className="mb-8">
          <h2 className="mb-2 text-sm font-medium text-zinc-300">
            Files Modified
          </h2>
          <ul className="space-y-1 text-sm text-zinc-400">
            {details.files_modified.map((f) => (
              <li key={f} className="font-mono text-xs">
                {f}
              </li>
            ))}
          </ul>
        </section>
      )}

      <section>
        <h2 className="mb-3 text-sm font-medium text-zinc-300">Activity</h2>
        <div className="rounded-xl border border-zinc-800/80 bg-zinc-900/30">
          {details.activity.length === 0 ? (
            <p className="px-4 py-6 text-center text-sm text-zinc-500">
              No activity recorded
            </p>
          ) : (
            details.activity.map((a, i) => (
              <div
                key={`${a.timestamp}-${i}`}
                className="flex gap-4 border-b border-zinc-800/40 px-4 py-2 text-sm last:border-0"
              >
                <span className="font-mono text-xs text-zinc-600">
                  {formatTime(a.timestamp)}
                </span>
                <span className="text-zinc-400">{a.description}</span>
              </div>
            ))
          )}
        </div>
      </section>

      {details.error_message && (
        <section className="mt-6">
          <h2 className="mb-2 text-sm font-medium text-red-400">Error</h2>
          <p className="text-sm text-red-300">{details.error_message}</p>
        </section>
      )}
    </div>
  );
}

function InfoRow({
  label,
  value,
  mono,
}: {
  label: string;
  value: React.ReactNode;
  mono?: boolean;
}) {
  return (
    <div className="rounded-lg border border-zinc-800/60 bg-zinc-900/30 px-4 py-3">
      <p className="text-xs text-zinc-500">{label}</p>
      <p className={`mt-1 text-sm text-zinc-200 ${mono ? "font-mono text-xs break-all" : ""}`}>
        {value}
      </p>
    </div>
  );
}
