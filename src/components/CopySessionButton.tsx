import { useEffect, useRef, useState } from "react";
import { Copy, Loader2 } from "lucide-react";
import { api } from "../lib/tauri";
import { AVAILABLE_PROVIDERS } from "../types";
import { providerLabel } from "../lib/utils";

interface CopySessionButtonProps {
  sessionId: string;
  currentProvider: string;
  compact?: boolean;
}

export function CopySessionButton({
  sessionId,
  currentProvider,
  compact,
}: CopySessionButtonProps) {
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  const otherProviders = AVAILABLE_PROVIDERS.filter(
    (p) => p.id !== currentProvider,
  );

  useEffect(() => {
    if (!open) return;
    const handleClick = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, [open]);

  const handleSelect = async (providerId: string) => {
    setLoading(providerId);
    setError(null);
    try {
      await api.runSessionOnProvider(sessionId, providerId);
      setOpen(false);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(null);
    }
  };

  if (otherProviders.length === 0) return null;

  return (
    <div className="relative" ref={menuRef}>
      <button
        type="button"
        onClick={(e) => {
          e.preventDefault();
          e.stopPropagation();
          setOpen((v) => !v);
          setError(null);
        }}
        className={
          compact
            ? "inline-flex items-center gap-1 rounded-md border border-zinc-700 bg-zinc-900 px-2 py-1 text-xs text-zinc-300 transition-colors hover:border-zinc-600 hover:bg-zinc-800"
            : "inline-flex items-center gap-2 rounded-lg border border-zinc-700 bg-zinc-900 px-3 py-1.5 text-sm text-zinc-300 transition-colors hover:border-zinc-600 hover:bg-zinc-800"
        }
      >
        <Copy className={compact ? "h-3 w-3" : "h-4 w-4"} />
        Copy session
      </button>

      {open && (
        <div className="absolute right-0 z-20 mt-1 min-w-[180px] rounded-lg border border-zinc-700 bg-zinc-900 py-1 shadow-xl">
          <p className="px-3 py-1.5 text-[10px] uppercase tracking-wide text-zinc-500">
            Run on
          </p>
          {otherProviders.map((p) => (
            <button
              key={p.id}
              type="button"
              disabled={loading !== null}
              onClick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                void handleSelect(p.id);
              }}
              className="flex w-full items-center gap-2 px-3 py-2 text-left text-sm text-zinc-300 transition-colors hover:bg-zinc-800 disabled:opacity-50"
            >
              {loading === p.id ? (
                <Loader2 className="h-3.5 w-3.5 animate-spin text-zinc-400" />
              ) : (
                <span className="h-3.5 w-3.5" />
              )}
              {providerLabel(p.id)}
            </button>
          ))}
        </div>
      )}

      {error && (
        <p className="absolute right-0 top-full z-10 mt-1 max-w-xs rounded-md border border-red-900/60 bg-red-950/80 px-2 py-1 text-xs text-red-300">
          {error}
        </p>
      )}
    </div>
  );
}
