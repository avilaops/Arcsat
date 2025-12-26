import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { ArrowPathIcon } from "@heroicons/react/24/outline";
import dayjs from "../../lib/dayjs";

import { fetchLeads, LeadResponse, LeadStageEnum } from "../../lib/api/crm";

const stageFilters = [
  { label: "Todos", value: undefined },
  { label: "New", value: "new" },
  { label: "Contacted", value: "contacted" },
  { label: "Qualification", value: "qualification" },
  { label: "Proposal", value: "proposal" },
  { label: "Negotiation", value: "negotiation" },
  { label: "Won", value: "won" },
  { label: "Lost", value: "lost" }
] as const;

function formatCurrency(value: string) {
  return new Intl.NumberFormat("pt-BR", {
    style: "currency",
    currency: "BRL"
  }).format(Number(value));
}

export default function LeadsPage() {
  const [stage, setStage] = useState<(typeof stageFilters)[number]["value"]>();

  const queryParams = useMemo(() => {
    const params: Record<string, string> = { page: "1", limit: "20" };
    if (stage) params.stage = stage;
    return params;
  }, [stage]);

  const { data, isLoading, refetch, isRefetching } = useQuery({
    queryKey: ["crm", "leads", queryParams],
    queryFn: () => fetchLeads(queryParams)
  });

  const leads = data?.data ?? [];

  return (
    <section className="space-y-6">
      <header className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-semibold text-slate-100">Leads</h1>
          <p className="text-sm text-slate-500">Monitoramento completo da jornada comercial</p>
        </div>
        <div className="flex items-center gap-3">
          <button
            onClick={() => refetch()}
            className="inline-flex items-center gap-2 rounded-md border border-slate-700 bg-slate-900 px-4 py-2 text-sm font-medium text-slate-300 transition hover:border-primary-500/40 hover:text-primary-200"
          >
            <ArrowPathIcon className={`h-4 w-4 ${isRefetching ? "animate-spin text-primary-300" : "text-slate-400"}`} />
            Atualizar
          </button>
          <button className="rounded-md bg-primary-500 px-4 py-2 text-sm font-semibold text-white shadow-sm shadow-primary-900/40 transition hover:bg-primary-400">
            Novo lead
          </button>
        </div>
      </header>

      <div className="flex flex-wrap items-center gap-3">
        {stageFilters.map((option) => (
          <button
            key={option.label}
            onClick={() => setStage(option.value)}
            className={`rounded-full border px-4 py-1.5 text-xs font-semibold transition ${
              stage === option.value
                ? "border-primary-500 bg-primary-500/20 text-primary-200"
                : "border-slate-800 bg-slate-900/60 text-slate-400 hover:border-primary-500/40 hover:text-primary-100"
            }`}
          >
            {option.label}
          </button>
        ))}
      </div>

      <div className="overflow-hidden rounded-2xl border border-slate-800 bg-slate-950/60">
        <table className="min-w-full divide-y divide-slate-800">
          <thead className="bg-slate-950/80">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">
                Nome
              </th>
              <th className="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">
                Empresa
              </th>
              <th className="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">
                Valor
              </th>
              <th className="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">
                Probabilidade
              </th>
              <th className="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">
                Dono
              </th>
              <th className="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">
                Último contato
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-800/80">
            {isLoading ? (
              <tr>
                <td colSpan={6} className="px-6 py-12 text-center text-sm text-slate-400">
                  Carregando leads...
                </td>
              </tr>
            ) : leads.length === 0 ? (
              <tr>
                <td colSpan={6} className="px-6 py-12 text-center text-sm text-slate-400">
                  Nenhum lead encontrado para os filtros selecionados.
                </td>
              </tr>
            ) : (
              leads.map((lead) => <LeadRow key={lead.id} lead={lead} />)
            )}
          </tbody>
        </table>
      </div>
    </section>
  );
}

function LeadRow({ lead }: { lead: LeadResponse }) {
  const stageLabel = LeadStageEnum.options.find((option) => option === lead.stage) ?? lead.stage;

  return (
    <tr className="hover:bg-slate-900/60">
      <td className="px-6 py-4">
        <div className="flex flex-col">
          <span className="font-medium text-slate-100">{lead.name}</span>
          <span className="text-xs text-slate-500">{lead.email}</span>
        </div>
      </td>
      <td className="px-6 py-4 text-sm text-slate-300">{lead.company ?? "—"}</td>
      <td className="px-6 py-4 text-sm text-slate-300">{formatCurrency(lead.value)}</td>
      <td className="px-6 py-4 text-sm text-slate-300">
        <span className="inline-flex items-center gap-2">
          <span
            className="inline-flex h-2.5 w-2.5 rounded-full"
            style={{
              background: `conic-gradient(from 90deg at 50% 50%, #3b82f6 ${lead.probability}%, rgba(15,23,42,0.4) ${lead.probability}%)`
            }}
          />
          {lead.probability}%
        </span>
      </td>
      <td className="px-6 py-4 text-sm text-slate-300">
        <div className="flex flex-col">
          <span>{lead.owner.name}</span>
          <span className="text-xs text-slate-500">{lead.owner.email}</span>
        </div>
      </td>
      <td className="px-6 py-4 text-sm text-slate-300">
        {lead.last_contact ? dayjs(lead.last_contact).fromNow() : "Sem registro"}
      </td>
    </tr>
  );
}
