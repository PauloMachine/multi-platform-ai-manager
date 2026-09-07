import { useEffect, useState } from "react";
import { api } from "../lib/tauri";
import { SessionRow } from "../components/SessionList";
import type { AISession } from "../types";

export function SessionsPage() {
  const [sessions, setSessions] = useState<AISession[]>([]);
  const [filter, setFilter] = useState<string>("all");

  const loadSessions = () => {
    api.getSessions().then(setSessions);
  };

  useEffect(() => {
    loadSessions();
  }, []);

  const filtered =
    filter === "all"
      ? sessions
      : sessions.filter((s) => s.status === filter);

  return (
    <div className="p-6">
      <h1 className="mb-1 text-xl font-semibold">Sessions</h1>
      <p className="mb-4 text-sm text-zinc-500">All recorded AI sessions</p>

      <div className="mb-4 flex gap-2">
        {["all", "running", "completed", "failed", "aborted"].map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={`rounded-md px-2.5 py-1 text-xs capitalize ${
              filter === f
                ? "bg-zinc-800 text-zinc-100"
                : "text-zinc-500 hover:text-zinc-300"
            }`}
          >
            {f}
          </button>
        ))}
      </div>

      <div className="rounded-xl border border-zinc-800/80 bg-zinc-900/30 px-4">
        {filtered.map((s) => (
          <SessionRow key={s.id} session={s} onSessionStopped={loadSessions} />
        ))}
        {filtered.length === 0 && (
          <p className="py-6 text-center text-sm text-zinc-500">
            No sessions match this filter
          </p>
        )}
      </div>
    </div>
  );
}
