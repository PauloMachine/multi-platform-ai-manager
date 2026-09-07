import { useEffect, useState } from "react";
import { Link, useParams, useSearchParams } from "react-router-dom";
import { ArrowLeft } from "lucide-react";
import { SessionRow } from "../components/SessionList";
import { api } from "../lib/tauri";
import type { AISession } from "../types";

export function ModelDetailPage() {
  const { model } = useParams<{ model: string }>();
  const [search] = useSearchParams();
  const provider = search.get("provider") ?? undefined;
  const decodedModel = model ? decodeURIComponent(model) : "";
  const [sessions, setSessions] = useState<AISession[]>([]);

  useEffect(() => {
    if (!decodedModel) return;
    api.getSessions(provider, decodedModel).then(setSessions);
  }, [decodedModel, provider]);

  return (
    <div className="p-6">
      <Link
        to="/models"
        className="mb-4 inline-flex items-center gap-1 text-sm text-zinc-500 hover:text-zinc-300"
      >
        <ArrowLeft className="h-4 w-4" /> Models
      </Link>
      <h1 className="mb-1 text-xl font-semibold">{decodedModel}</h1>
      <p className="mb-6 text-sm text-zinc-500">
        Session history for this model
      </p>
      <div className="rounded-xl border border-zinc-800/80 bg-zinc-900/30 px-4">
        {sessions.map((s) => (
          <SessionRow key={s.id} session={s} />
        ))}
        {sessions.length === 0 && (
          <p className="py-6 text-center text-sm text-zinc-500">
            No sessions found for this model
          </p>
        )}
      </div>
    </div>
  );
}
