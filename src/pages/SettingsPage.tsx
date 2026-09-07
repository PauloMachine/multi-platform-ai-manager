import { useState } from "react";
import { RefreshCw, Plug, RotateCcw } from "lucide-react";
import { useAppStore } from "../stores/appStore";
import { api } from "../lib/tauri";
import { REFRESH_INTERVALS } from "../types";
import { providerLabel } from "../lib/utils";

export function SettingsPage() {
  const { config, updateConfig, auth, connectProvider, loadAuth, refresh } =
    useAppStore();
  const [message, setMessage] = useState<string | null>(null);

  if (!config) return <div className="p-6 text-zinc-500">Loading…</div>;

  const set = async (patch: Partial<typeof config>) => {
    await updateConfig({ ...config, ...patch });
  };

  const handleConnect = async (provider: string) => {
    try {
      const result = await connectProvider(provider);
      setMessage(result);
    } catch (e) {
      setMessage(String(e));
    }
  };

  const handleInstallHooks = async (provider: string) => {
    const result = await api.installProviderHooks(provider);
    setMessage(result.message);
  };

  return (
    <div className="p-6 max-w-2xl">
      <h1 className="mb-1 text-xl font-semibold">Settings</h1>
      <p className="mb-6 text-sm text-zinc-500">Application preferences</p>

      {message && (
        <div className="mb-4 rounded-lg border border-zinc-700 bg-zinc-900 px-4 py-2 text-sm text-zinc-300">
          {message}
        </div>
      )}

      <section className="mb-8 space-y-4">
        <h2 className="text-sm font-medium text-zinc-300">General</h2>

        <SettingRow label="Refresh usage every">
          <select
            value={config.refresh_interval_secs}
            onChange={(e) => set({ refresh_interval_secs: Number(e.target.value) })}
            className="rounded-lg border border-zinc-700 bg-zinc-900 px-3 py-1.5 text-sm text-zinc-200"
          >
            {REFRESH_INTERVALS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </SettingRow>

        <SettingRow label="Minimize to tray on close">
          <Toggle
            checked={config.minimize_to_tray}
            onChange={(v) => set({ minimize_to_tray: v })}
          />
        </SettingRow>

        <SettingRow label="Desktop notifications">
          <Toggle
            checked={config.desktop_notifications}
            onChange={(v) => set({ desktop_notifications: v })}
          />
        </SettingRow>

        <SettingRow label="Start with system">
          <Toggle
            checked={config.start_with_system}
            onChange={(v) => set({ start_with_system: v })}
          />
        </SettingRow>

        <button
          onClick={() => refresh()}
          className="inline-flex items-center gap-2 rounded-lg border border-zinc-700 px-3 py-1.5 text-sm text-zinc-300 hover:bg-zinc-900"
        >
          <RefreshCw className="h-4 w-4" /> Refresh now
        </button>
      </section>

      <section className="space-y-4">
        <h2 className="text-sm font-medium text-zinc-300">Providers</h2>
        {auth.map((a) => (
          <div
            key={a.provider}
            className="rounded-xl border border-zinc-800 bg-zinc-900/40 p-4"
          >
            <div className="mb-3 flex items-center justify-between">
              <div>
                <p className="text-sm font-medium">{providerLabel(a.provider)}</p>
                <p className="text-xs capitalize text-zinc-500">
                  {a.status.replace("not", "not ")}
                  {a.version && ` · ${a.version}`}
                </p>
              </div>
              <div className="flex gap-2">
                {a.status !== "connected" && a.status !== "notinstalled" && (
                  <button
                    onClick={() => handleConnect(a.provider)}
                    className="inline-flex items-center gap-1 rounded-md bg-emerald-600 px-2.5 py-1 text-xs text-white hover:bg-emerald-500"
                  >
                    <Plug className="h-3 w-3" /> Connect
                  </button>
                )}
                <button
                  onClick={() => {
                    loadAuth();
                    refresh();
                  }}
                  className="inline-flex items-center gap-1 rounded-md border border-zinc-700 px-2.5 py-1 text-xs text-zinc-400 hover:bg-zinc-800"
                >
                  <RotateCcw className="h-3 w-3" /> Recheck
                </button>
                <button
                  onClick={() => handleInstallHooks(a.provider)}
                  className="inline-flex items-center gap-1 rounded-md border border-zinc-700 px-2.5 py-1 text-xs text-zinc-400 hover:bg-zinc-800"
                >
                  Install hooks
                </button>
              </div>
            </div>
            {a.message && (
              <p className="text-xs text-zinc-600">{a.message}</p>
            )}
          </div>
        ))}
      </section>
    </div>
  );
}

function SettingRow({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between rounded-lg border border-zinc-800/60 bg-zinc-900/30 px-4 py-3">
      <span className="text-sm text-zinc-300">{label}</span>
      {children}
    </div>
  );
}

function Toggle({
  checked,
  onChange,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <button
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      className={`relative h-6 w-11 rounded-full transition-colors ${
        checked ? "bg-emerald-600" : "bg-zinc-700"
      }`}
    >
      <span
        className={`absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform ${
          checked ? "translate-x-5" : ""
        }`}
      />
    </button>
  );
}
