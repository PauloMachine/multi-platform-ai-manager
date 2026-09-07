import { ProviderCard } from "../components/ProviderCard";
import { useAppStore } from "../stores/appStore";

export function ProvidersPage() {
  const { dashboard } = useAppStore();

  return (
    <div className="p-6">
      <h1 className="mb-1 text-xl font-semibold">Providers</h1>
      <p className="mb-6 text-sm text-zinc-500">
        AI coding tools monitored on this machine
      </p>
      <div className="grid gap-4 md:grid-cols-2">
        {(dashboard?.providers ?? []).map((p) => (
          <ProviderCard key={p.provider} provider={p} />
        ))}
      </div>
    </div>
  );
}
