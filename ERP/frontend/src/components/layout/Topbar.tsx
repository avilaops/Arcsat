import { MagnifyingGlassIcon, BellIcon } from "@heroicons/react/24/outline";
import { useMemo } from "react";

const quickStats = [
  { label: "MRR", value: "R$ 128K", trend: "up" },
  { label: "Taxa de Conversão", value: "32%", trend: "neutral" },
  { label: "NPS", value: "78", trend: "up" }
];

export default function Topbar() {
  const today = useMemo(
    () =>
      new Intl.DateTimeFormat("pt-BR", {
        weekday: "long",
        day: "2-digit",
        month: "long"
      }).format(new Date()),
    []
  );

  return (
    <header className="border-b border-slate-800 bg-slate-950/60 backdrop-blur">
      <div className="mx-auto flex max-w-7xl items-center justify-between gap-4 px-6 py-4">
        <div>
          <p className="text-xs uppercase tracking-wide text-slate-500">{today}</p>
          <h1 className="text-xl font-semibold text-slate-100">Painel Operacional</h1>
        </div>
        <div className="flex items-center gap-3">
          <div className="relative hidden md:block">
            <MagnifyingGlassIcon className="pointer-events-none absolute left-3 top-2.5 h-5 w-5 text-slate-500" />
            <input
              type="search"
              placeholder="Buscar leads, contas ou atividades"
              className="w-72 rounded-md border border-slate-700 bg-slate-900 py-2 pl-10 pr-3 text-sm text-slate-100 placeholder:text-slate-500 focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-500/40"
            />
          </div>
          <button className="relative flex h-10 w-10 items-center justify-center rounded-full border border-slate-700 bg-slate-900 text-slate-300 transition hover:border-primary-500/50 hover:text-primary-200">
            <BellIcon className="h-5 w-5" />
            <span className="absolute right-2 top-2 block h-2 w-2 rounded-full bg-primary-400" />
          </button>
          <div className="flex items-center gap-3 rounded-full border border-slate-800 bg-slate-900 px-4 py-2">
            <div className="h-8 w-8 rounded-full bg-gradient-to-br from-primary-500 via-primary-600 to-indigo-600" />
            <div>
              <p className="text-sm font-medium text-slate-100">João Silva</p>
              <p className="text-xs text-slate-500">Sales Manager</p>
            </div>
          </div>
        </div>
      </div>
      <div className="border-t border-slate-800 bg-slate-950/80">
        <div className="mx-auto flex max-w-7xl gap-4 px-6 py-3 text-sm text-slate-300">
          {quickStats.map((stat) => (
            <div
              key={stat.label}
              className="flex items-center gap-2 rounded-md border border-slate-800 bg-slate-900/60 px-4 py-2"
            >
              <span className="text-slate-400">{stat.label}</span>
              <strong className="text-slate-100">{stat.value}</strong>
            </div>
          ))}
        </div>
      </div>
    </header>
  );
}
