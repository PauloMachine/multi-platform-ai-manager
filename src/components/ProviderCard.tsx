import { Link } from "react-router-dom";
import { ChevronRight, Wifi, WifiOff, AlertCircle } from "lucide-react";
import type { ProviderUsage } from "../types";
import { UsageBar } from "./UsageBar";
import { cn, formatResetTime } from "../lib/utils";

interface ProviderCardProps {
  provider: ProviderUsage;
  compact?: boolean;
}

function AuthBadge({ status }: { status: ProviderUsage["auth_status"] }) {
  if (status === "connected") {
    return (
      <span className="inline-flex items-center gap-1 text-xs text-emerald-400">
        <Wifi className="h-3 w-3" /> Connected
      </span>
    );
  }
  if (status === "notinstalled") {
    return (
      <span className="inline-flex items-center gap-1 text-xs text-zinc-500">
        <AlertCircle className="h-3 w-3" /> Not installed
      </span>
    );
  }
  return (
    <span className="inline-flex items-center gap-1 text-xs text-amber-400">
      <WifiOff className="h-3 w-3" /> Not connected
    </span>
  );
}

export function ProviderCard({ provider, compact }: ProviderCardProps) {
  const resetText = formatResetTime(provider.five_hour?.resets_at);

  return (
    <Link
      to={`/providers/${provider.provider}`}
      className={cn(
        "group block rounded-xl border border-zinc-800/80 bg-zinc-900/50 p-4 transition-colors hover:border-zinc-700 hover:bg-zinc-900",
        compact && "p-3",
      )}
    >
      <div className="mb-3 flex items-start justify-between">
        <div>
          <h3 className="text-sm font-medium text-zinc-100">{provider.name}</h3>
          <AuthBadge status={provider.auth_status} />
        </div>
        <ChevronRight className="h-4 w-4 text-zinc-600 transition-transform group-hover:translate-x-0.5 group-hover:text-zinc-400" />
      </div>

      {provider.usage_available && (
        <UsageBar
          percentage={provider.five_hour?.percentage}
          label={provider.five_hour?.label ?? "5 hour usage"}
          resetText={resetText}
          size={compact ? "sm" : "md"}
        />
      )}

      {provider.seven_day?.percentage != null && (
        <div className="mt-3">
          <UsageBar
            percentage={provider.seven_day.percentage}
            label={provider.seven_day.label ?? "7 day usage"}
            resetText={formatResetTime(provider.seven_day.resets_at)}
            size="sm"
          />
        </div>
      )}

      {!provider.installed && (
        <p className="mt-3 text-xs text-zinc-500">
          Install the CLI to enable monitoring.
        </p>
      )}
      {provider.installed && !provider.usage_available && provider.auth_status === "connected" && (
        <p className="mt-3 text-xs text-zinc-500">
          Sessions monitoring enabled. Usage quota unavailable via official API.
        </p>
      )}
    </Link>
  );
}
