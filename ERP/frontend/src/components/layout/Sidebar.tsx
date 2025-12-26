import { NavLink } from "react-router-dom";
import { clsx } from "clsx";
import {
  ChartPieIcon,
  UserGroupIcon,
  BuildingOfficeIcon,
  Squares2X2Icon,
  ArrowTrendingUpIcon
} from "@heroicons/react/24/outline";

const navItems = [
  { to: "/", label: "Visão Geral", icon: Squares2X2Icon },
  { to: "/crm/leads", label: "Leads", icon: UserGroupIcon },
  { to: "/crm/accounts", label: "Contas", icon: BuildingOfficeIcon },
  { to: "/crm/activities", label: "Atividades", icon: ChartPieIcon },
  { to: "/crm/forecast", label: "Forecast", icon: ArrowTrendingUpIcon }
];

export default function Sidebar() {
  return (
    <aside className="w-64 bg-slate-950 border-r border-slate-800 hidden lg:flex flex-col">
      <div className="px-6 py-5 border-b border-slate-800">
        <span className="text-lg font-semibold text-primary-300">Avila ERP</span>
        <p className="text-sm text-slate-400">CRM Inteligente</p>
      </div>
      <nav className="flex-1 overflow-y-auto px-2 py-4 space-y-1">
        {navItems.map(({ to, label, icon: Icon }) => (
          <NavLink
            key={to}
            to={to}
            end={to === "/"}
            className={({ isActive }) =>
              clsx(
                "flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition",
                isActive
                  ? "bg-primary-500/10 text-primary-200 border border-primary-500/40"
                  : "text-slate-400 hover:text-slate-200 hover:bg-slate-800/60"
              )
            }
          >
            <Icon className="h-5 w-5" />
            <span>{label}</span>
          </NavLink>
        ))}
      </nav>
    </aside>
  );
}
