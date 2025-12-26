export default function ForecastPage() {
  return (
    <section className="rounded-3xl border border-slate-800 bg-slate-950/60 p-6 text-slate-300">
      <h1 className="text-2xl font-semibold text-slate-100">Forecast</h1>
      <p className="mt-2 text-sm text-slate-500">
        Previsão de receita ponderada com base nas probabilidades. Implementaremos gráficos interativos e cache inteligente conectados ao endpoint /crm/opportunities/forecast.
      </p>
    </section>
  );
}
