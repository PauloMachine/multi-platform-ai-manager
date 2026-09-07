import { cn, usageLevel } from "../lib/utils";

interface UsageBarProps {
  percentage?: number | null;
  label?: string;
  resetText?: string | null;
  unavailable?: boolean;
  size?: "sm" | "md";
}

export function UsageBar({
  percentage,
  label,
  resetText,
  unavailable,
  size = "md",
}: UsageBarProps) {
  const level = usageLevel(percentage);
  const pct = percentage ?? 0;

  if (unavailable || percentage == null) {
    return (
      <div className="space-y-1.5">
        {label && (
          <div className="flex justify-between text-xs text-zinc-500">
            <span>{label}</span>
          </div>
        )}
        <div className="h-2 rounded-full bg-zinc-800/80">
          <div className="flex h-full items-center justify-center text-[10px] text-zinc-500">
            Usage limit unavailable
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-1.5">
      <div className="flex justify-between text-xs">
        <span className="text-zinc-400">{label}</span>
        <span
          className={cn(
            "font-mono tabular-nums",
            level === "critical" && "text-red-400",
            level === "warning" && "text-amber-400",
            level === "normal" && "text-zinc-300",
          )}
        >
          {Math.round(pct)}%
        </span>
      </div>
      <div
        className={cn(
          "overflow-hidden rounded-full bg-zinc-800/80",
          size === "sm" ? "h-1.5" : "h-2",
        )}
      >
        <div
          className={cn(
            "h-full rounded-full transition-all duration-500",
            level === "critical" && "bg-red-500",
            level === "warning" && "bg-amber-500",
            level === "normal" && "bg-emerald-500",
          )}
          style={{ width: `${Math.min(100, Math.max(0, pct))}%` }}
        />
      </div>
      {resetText && (
        <p className="text-[11px] text-zinc-500">resets in {resetText}</p>
      )}
    </div>
  );
}
