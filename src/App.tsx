import { useEffect, useState } from "react";
import {
  BrowserRouter,
  Navigate,
  Route,
  Routes,
  useNavigate,
} from "react-router-dom";
import { Layout } from "./components/Layout";
import { onNavigate } from "./lib/tauri";
import { DashboardPage } from "./pages/DashboardPage";
import { ModelDetailPage } from "./pages/ModelDetailPage";
import { ModelsPage } from "./pages/ModelsPage";
import { OnboardingPage } from "./pages/OnboardingPage";
import { ProviderDetailPage } from "./pages/ProviderDetailPage";
import { ProvidersPage } from "./pages/ProvidersPage";
import { SessionDetailPage } from "./pages/SessionDetailPage";
import { SessionsPage } from "./pages/SessionsPage";
import { SettingsPage } from "./pages/SettingsPage";
import { APP_NAME } from "./lib/constants";
import { useAppStore } from "./stores/appStore";

function AppRoutes() {
  const navigate = useNavigate();
  const { config, auth, init, loading } = useAppStore();
  const [ready, setReady] = useState(false);

  useEffect(() => {
    init().finally(() => setReady(true));
  }, [init]);

  useEffect(() => {
    const unlisten = onNavigate((path) => navigate(path));
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [navigate]);

  if (!ready || loading) {
    return (
      <div className="flex h-screen items-center justify-center bg-zinc-950 text-zinc-500">
        Loading {APP_NAME}…
      </div>
    );
  }

  if (config && !config.onboarding_completed) {
    return (
      <OnboardingPage
        auth={auth}
        onComplete={() => window.location.reload()}
      />
    );
  }

  return (
    <Routes>
      <Route element={<Layout />}>
        <Route index element={<DashboardPage />} />
        <Route path="providers" element={<ProvidersPage />} />
        <Route path="providers/:id" element={<ProviderDetailPage />} />
        <Route path="models" element={<ModelsPage />} />
        <Route path="models/:model" element={<ModelDetailPage />} />
        <Route path="sessions" element={<SessionsPage />} />
        <Route path="sessions/:id" element={<SessionDetailPage />} />
        <Route path="settings" element={<SettingsPage />} />
      </Route>
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}

export default function App() {
  return (
    <BrowserRouter>
      <AppRoutes />
    </BrowserRouter>
  );
}
