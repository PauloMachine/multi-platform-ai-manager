import { NavLink } from "react-router-dom";
import {
  LayoutDashboard,
  Boxes,
  Cpu,
  History,
  Settings,
  Activity,
} from "lucide-react";
import { cn } from "../lib/utils";

const links = [
  { to: "/", icon: LayoutDashboard, label: "Dashboard" },
  { to: "/providers", icon: Boxes, label: "Providers" },
  { to: "/models", icon: Cpu, label: "Models" },
  { to: "/sessions", icon: History, label: "Sessions" },
  { to: "/settings", icon: Settings, label: "Settings" },
];

export function Sidebar() {
  return (
    <aside className="flex w-52 shrink-0 flex-col border-r border-zinc-800/80 bg-zinc-950/80">
      <div className="flex items-center gap-2 border-b border-zinc-800/80 px-4 py-4">
        <Activity className="h-5 w-5 text-emerald-400" />
        <div>
          <p className="text-sm font-semibold text-zinc-100">Multi-platform</p>
          <p className="text-[10px] text-zinc-500">AI Manager</p>
        </div>
      </div>
      <nav className="flex flex-1 flex-col gap-0.5 p-2">
        {links.map(({ to, icon: Icon, label }) => (
          <NavLink
            key={to}
            to={to}
            end={to === "/"}
            className={({ isActive }) =>
              cn(
                "flex items-center gap-2.5 rounded-lg px-3 py-2 text-sm transition-colors",
                isActive
                  ? "bg-zinc-800/80 text-zinc-100"
                  : "text-zinc-400 hover:bg-zinc-900 hover:text-zinc-200",
              )
            }
          >
            <Icon className="h-4 w-4" />
            {label}
          </NavLink>
        ))}
      </nav>
      <div className="border-t border-zinc-800/80 p-3">
        <p className="text-[10px] leading-relaxed text-zinc-600">
          Local-first. No data leaves your machine.
        </p>
      </div>
    </aside>
  );
}
