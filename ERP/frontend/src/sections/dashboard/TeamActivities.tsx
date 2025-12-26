const activities = [
  {
    id: "ACT-1024",
    subject: "Reunião de demonstração",
    owner: "Marina Costa",
    scheduledAt: "09:30",
    account: "Stone Pagamentos",
    status: "scheduled"
  },
  {
    id: "ACT-1025",
    subject: "Envio de proposta revisada",
    owner: "Paulo Andrade",
    scheduledAt: "11:00",
    account: "XP Investimentos",
    status: "completed"
  },
  {
    id: "ACT-1026",
    subject: "Negociação de cláusula",
    owner: "Fernanda Dias",
    scheduledAt: "15:00",
    account: "Nubank",
    status: "in-progress"
  }
];

const statusMap: Record<string, string> = {
  scheduled: "Agendado",
  completed: "Concluído",
  "in-progress": "Em andamento"
};

export default function TeamActivities() {
  return (
    <section className="rounded-3xl border border-slate-800 bg-slate-950/60 p-6">
      <div className="flex items-center justify-between gap-4">
        <div>
          <h2 className="text-lg font-semibold text-slate-100">Agenda do time hoje</h2>
          <p className="text-sm text-slate-500">Próximas interações com clientes estratégicos</p>
        </div>
        <button className="rounded-md border border-slate-700 px-4 py-2 text-sm font-medium text-slate-300 transition hover:border-primary-500/50 hover:text-primary-200">
          Ver calendário completo
        </button>
      </div>
      <div className="mt-6 space-y-3">
        {activities.map((activity) => (
          <div
            key={activity.id}
            className="flex flex-col gap-3 rounded-2xl border border-slate-800 bg-slate-900/60 px-4 py-3 md:flex-row md:items-center md:justify-between"
          >
            <div>
              <p className="font-medium text-slate-100">{activity.subject}</p>
              <p className="text-sm text-slate-400">
                {activity.owner} • {activity.account}
              </p>
            </div>
            <div className="flex items-center gap-4 text-sm text-slate-400">
              <span className="font-medium text-slate-200">{activity.scheduledAt}</span>
              <span className="rounded-full bg-primary-500/10 px-3 py-1 text-xs font-semibold text-primary-200">
                {statusMap[activity.status]}
              </span>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}
