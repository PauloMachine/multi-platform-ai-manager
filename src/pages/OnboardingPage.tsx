import { useState } from "react";
import { CheckCircle2, Circle } from "lucide-react";
import { api } from "../lib/tauri";
import { useAppStore } from "../stores/appStore";
import type { ProviderAuthStatus } from "../types";
import { providerLabel } from "../lib/utils";
import { APP_NAME } from "../lib/constants";

interface OnboardingPageProps {
  auth: ProviderAuthStatus[];
  onComplete: () => void;
}

export function OnboardingPage({ auth, onComplete }: OnboardingPageProps) {
  const { connectProvider } = useAppStore();
  const [connecting, setConnecting] = useState<string | null>(null);

  const handleConnect = async (provider: string) => {
    setConnecting(provider);
    try {
      await connectProvider(provider);
    } finally {
      setConnecting(null);
    }
  };

  const handleContinue = async () => {
    await api.completeOnboarding();
    onComplete();
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-zinc-950 p-6">
      <div className="w-full max-w-md">
        <div className="mb-8 text-center">
          <h1 className="text-2xl font-semibold text-zinc-100">
            Welcome to {APP_NAME}
          </h1>
          <p className="mt-2 text-sm text-zinc-500">
            Monitor your AI coding tools locally. No login required for this app.
          </p>
        </div>

        <div className="space-y-3">
          {auth.map((a) => (
            <div
              key={a.provider}
              className="rounded-xl border border-zinc-800 bg-zinc-900/50 p-4"
            >
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-medium text-zinc-200">
                    {providerLabel(a.provider)}
                  </p>
                  <p className="mt-1 flex items-center gap-1 text-xs text-zinc-500">
                    {a.status === "notinstalled" ? (
                      <>
                        <Circle className="h-3 w-3" /> Not detected
                      </>
                    ) : (
                      <>
                        <CheckCircle2 className="h-3 w-3 text-emerald-400" />{" "}
                        Detected
                      </>
                    )}
                  </p>
                </div>
                {a.status !== "notinstalled" && a.status !== "connected" && (
                  <button
                    onClick={() => handleConnect(a.provider)}
                    disabled={connecting === a.provider}
                    className="rounded-lg bg-zinc-800 px-3 py-1.5 text-sm text-zinc-200 hover:bg-zinc-700 disabled:opacity-50"
                  >
                    Connect
                  </button>
                )}
                {a.status === "connected" && (
                  <span className="text-xs text-emerald-400">Connected</span>
                )}
              </div>
            </div>
          ))}
        </div>

        <button
          onClick={handleContinue}
          className="mt-8 w-full rounded-lg bg-emerald-600 py-2.5 text-sm font-medium text-white hover:bg-emerald-500"
        >
          Continue to Dashboard
        </button>
      </div>
    </div>
  );
}
