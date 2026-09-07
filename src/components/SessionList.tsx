import { Link } from "react-router-dom";
import type { AISession } from "../types";
import { cn, formatDuration, providerLabel, statusColor } from "../lib/utils";
import { useEffect, useState } from "react";
import { CopySessionButton } from "./CopySessionButton";
import { StopSessionButton } from "./StopSessionButton";

interface ActiveSessionCardProps {
  session: AISession;
}

export function ActiveSessionCard({ session }: ActiveSessionCardProps) {
  const [elapsed, setElapsed] = useState(session.duration_secs ?? 0);

  useEffect(() => {
    const start = new Date(session.started_at).getTime();
    const tick = () => {
      setElapsed(Math.max(0, Math.floor((Date.now() - start) / 1000)));
    };
    tick();
    const id = setInterval(tick, 1000);
    return () => clearInterval(id);
  }, [session.started_at, session.duration_secs]);

  return (
    <div className="flex items-center gap-2 rounded-lg border border-zinc-800/60 bg-zinc-900/40 px-3 py-2.5 transition-colors hover:border-zinc-700">
      <Link
        to={`/sessions/${session.id}`}
        className="flex min-w-0 flex-1 items-center gap-3"
      >
        <span className="relative flex h-2 w-2 shrink-0">
          <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-blue-400 opacity-60" />
          <span className="relative inline-flex h-2 w-2 rounded-full bg-blue-500" />
        </span>
        <div className="min-w-0 flex-1">
          <p className="truncate text-sm text-zinc-200">
            {session.model ?? "Unknown model"}
          </p>
          <p className="truncate text-xs text-zinc-500">
            {providerLabel(session.provider)}
            {session.project ? ` · ${session.project.split("/").pop()}` : ""}
          </p>
        </div>
        <span className="font-mono text-xs tabular-nums text-zinc-400">
          {formatDuration(elapsed)}
        </span>
      </Link>
      <StopSessionButton sessionId={session.id} compact />
    </div>
  );
}

interface SessionRowProps {
  session: AISession;
  onSessionStopped?: () => void;
}

export function SessionRow({ session, onSessionStopped }: SessionRowProps) {
  return (
    <div className="flex items-center gap-3 border-b border-zinc-800/50 px-1 py-3 transition-colors last:border-0 hover:bg-zinc-900/30">
      <Link
        to={`/sessions/${session.id}`}
        className="flex min-w-0 flex-1 items-center gap-3"
      >
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <span className={cn("text-xs font-medium capitalize", statusColor(session.status))}>
              {session.status}
            </span>
            <span className="text-sm text-zinc-300">
              {session.model ?? "Unknown"}
            </span>
          </div>
          <p className="mt-0.5 truncate text-xs text-zinc-500">
            {session.initial_prompt ?? session.project ?? "No description"}
          </p>
        </div>
        <div className="text-right text-xs text-zinc-500">
          <div>{providerLabel(session.provider)}</div>
          <div className="font-mono tabular-nums">
            {formatDuration(session.duration_secs)}
          </div>
        </div>
      </Link>
      {session.status === "running" && (
        <StopSessionButton sessionId={session.id} compact onStopped={onSessionStopped} />
      )}
      <CopySessionButton
        sessionId={session.id}
        currentProvider={session.provider}
        compact
      />
    </div>
  );
}
