import { Link } from "react-router-dom";

const stages = [
  {
    name: "New",
    count: 42,
    value: "R$ 180K",
    probability: 10,
    gradient: "from-cyan-500/20 to-cyan-500/5"
  },
  {
    name: "Contacted",
    count: 38,
    value: "R$ 220K",
    probability: 18,
    gradient: "from-sky-500/20 to-sky-500/5"
  },
  {
    name: "Qualification",
    count: 27,
    value: "R$ 310K",
    probability: 32,
    gradient: "from-indigo-500/20 to-indigo-500/5"
  },
  {
    name: "Proposal",
    count: 19,
    value: "R$ 260K",
    probability: 58,
    gradient: "from-violet-500/20 to-violet-500/5"
  },
  {
    name: "Negotiation",
    count: 11,
    value: "R$ 190K",
    probability: 76,
    gradient: "from-fuchsia-500/20 to-fuchsia-500/5"
  }
];

export default function PipelineGlance() {
  return (
    <section className="rounded-3xl border border-slate-800 bg-slate-950/60 p-6">
      <div className="flex items-center justify-between gap-4">
        <div>
          <h2 className="text-lg font-semibold text-slate-100">Pipeline em andamento</h2>
          <p className="text-sm text-slate-500">Resumo ponderado das oportunidades abertas</p>
        </div>
        <Link
          to="/crm/pipeline"
          className="rounded-md border border-primary-500/40 bg-primary-500/10 px-4 py-2 text-sm font-semibold text-primary-200 transition hover:bg-primary-500/20"
        >
          Abrir Kanban
        </Link>
      </div>
      <div className="mt-6 grid gap-3 md:grid-cols-2 xl:grid-cols-5">
        {stages.map((stage) => (
          <div
            key={stage.name}
            className={`rounded-2xl border border-slate-800 bg-gradient-to-br ${stage.gradient} p-5`}
          >
            <p className="text-xs uppercase tracking-wide text-slate-400">{stage.name}</p>
            <p className="mt-3 text-2xl font-semibold text-slate-50">{stage.value}</p>
            <p className="text-sm text-slate-400">{stage.count} deals</p>
            <div className="mt-4 h-2 w-full overflow-hidden rounded-full bg-slate-800">
              <div
                className="h-full rounded-full bg-primary-500"
                style={{ width: `${stage.probability}%` }}
              />
            </div>
            <p className="mt-2 text-xs text-slate-500">Prob. {stage.probability}%</p>
          </div>
        ))}
      </div>
    </section>
  );
}
