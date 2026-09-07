import { useState } from "react";
import { Loader2, Square } from "lucide-react";
import { api } from "../lib/tauri";
import { useAppStore } from "../stores/appStore";

interface StopSessionButtonProps {
  sessionId: string;
  compact?: boolean;
  onStopped?: () => void;
}

export function StopSessionButton({
  sessionId,
  compact,
  onStopped,
}: StopSessionButtonProps) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const refresh = useAppStore((s) => s.refresh);

  const handleStop = async (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setLoading(true);
    setError(null);
    try {
      await api.stopSession(sessionId);
      await refresh();
      onStopped?.();
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="relative">
      <button
        type="button"
        onClick={handleStop}
        disabled={loading}
        title="Mark session as stopped"
        className={
          compact
            ? "inline-flex items-center gap-1 rounded-md border border-red-900/50 bg-red-950/30 px-2 py-1 text-xs text-red-300 transition-colors hover:border-red-800 hover:bg-red-950/50 disabled:opacity-50"
            : "inline-flex items-center gap-2 rounded-lg border border-red-900/50 bg-red-950/30 px-3 py-1.5 text-sm text-red-300 transition-colors hover:border-red-800 hover:bg-red-950/50 disabled:opacity-50"
        }
      >
        {loading ? (
          <Loader2 className={compact ? "h-3 w-3 animate-spin" : "h-4 w-4 animate-spin"} />
        ) : (
          <Square className={compact ? "h-3 w-3 fill-current" : "h-4 w-4 fill-current"} />
        )}
        Stop
      </button>
      {error && (
        <p className="absolute right-0 top-full z-10 mt-1 max-w-xs rounded-md border border-red-900/60 bg-red-950/80 px-2 py-1 text-xs text-red-300">
          {error}
        </p>
      )}
    </div>
  );
}
