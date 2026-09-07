import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatDuration(secs?: number | null): string {
  if (secs == null) return "—";
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}

export function formatResetTime(resetsAt?: number | null): string | null {
  if (!resetsAt) return null;
  const now = Date.now();
  const resetMs = resetsAt * 1000;
  const diff = resetMs - now;
  if (diff <= 0) return "soon";
  const hours = Math.floor(diff / 3600000);
  const mins = Math.floor((diff % 3600000) / 60000);
  if (hours > 0) return `${hours}h ${mins}m`;
  return `${mins}m`;
}

function pad2(n: number): string {
  return n.toString().padStart(2, "0");
}

export function formatDateTime(iso: string): string {
  try {
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return iso;
    return `${d.getFullYear()}-${pad2(d.getMonth() + 1)}-${pad2(d.getDate())} ${pad2(d.getHours())}:${pad2(d.getMinutes())}:${pad2(d.getSeconds())}`;
  } catch {
    return iso;
  }
}

export function formatTime(iso: string): string {
  return formatDateTime(iso);
}

export function usageLevel(pct?: number | null): "normal" | "warning" | "critical" | "none" {
  if (pct == null) return "none";
  if (pct >= 90) return "critical";
  if (pct >= 80) return "warning";
  return "normal";
}

export function providerLabel(id: string): string {
  switch (id) {
    case "claude":
      return "Claude Code";
    case "cursor":
      return "Cursor";
    case "codex":
      return "Codex";
    default:
      return id;
  }
}

export function statusColor(status: string): string {
  switch (status) {
    case "running":
      return "text-blue-400";
    case "completed":
      return "text-emerald-400";
    case "failed":
      return "text-red-400";
    case "aborted":
      return "text-amber-400";
    default:
      return "text-zinc-400";
  }
}
