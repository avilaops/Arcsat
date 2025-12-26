import { SparklesIcon, ArrowTrendingUpIcon, CurrencyDollarIcon } from "@heroicons/react/24/outline";

const cards = [
  {
    title: "Receita Mensal",
    value: "R$ 482.400",
    delta: "+12%",
    icon: CurrencyDollarIcon,
    description: "Comparado ao último mês"
  },
  {
    title: "Leads Ativos",
    value: "284",
    delta: "+37",
    icon: SparklesIcon,
    description: "Entradas nas últimas 24h"
  },
  {
    title: "Probabilidade Forecast",
    value: "68%",
    delta: "+5pp",
    icon: ArrowTrendingUpIcon,
    description: "Pipeline ponderado"
  }
];

export default function KPIOverview() {
  return (
    <section className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
      {cards.map((card) => {
        const Icon = card.icon;
        return (
          <div
            key={card.title}
            className="rounded-2xl border border-slate-800 bg-gradient-to-br from-slate-900 via-slate-950 to-slate-950 p-6 shadow-lg shadow-primary-900/10"
          >
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium uppercase tracking-wide text-slate-400">
                  {card.title}
                </p>
                <p className="mt-3 text-3xl font-semibold text-slate-50">{card.value}</p>
                <p className="mt-2 text-xs text-slate-500">{card.description}</p>
              </div>
              <span className="rounded-full bg-primary-500/10 p-3 text-primary-300">
                <Icon className="h-6 w-6" />
              </span>
            </div>
            <p className="mt-6 inline-flex items-center rounded-full bg-emerald-500/10 px-3 py-1 text-xs font-semibold text-emerald-300">
              {card.delta}
            </p>
          </div>
        );
      })}
    </section>
  );
}
