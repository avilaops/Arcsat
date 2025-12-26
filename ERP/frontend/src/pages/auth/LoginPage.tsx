import { useNavigate } from "react-router-dom";

import LoginForm from "../../components/auth/LoginForm";

export default function LoginPage() {
  const navigate = useNavigate();

  return (
    <div className="flex min-h-screen bg-slate-950">
      <div className="relative hidden flex-1 items-center justify-center overflow-hidden bg-gradient-to-br from-primary-500 via-indigo-500 to-blue-700 lg:flex">
        <div className="pointer-events-none absolute inset-0 bg-gradient-to-br from-slate-900/40 via-slate-900/20 to-slate-900/80" />
        <div className="relative z-10 max-w-xl px-16 text-slate-100">
          <p className="text-sm uppercase tracking-[0.3em] text-slate-200/70">Avila ERP</p>
          <h1 className="mt-6 text-4xl font-bold leading-tight">Orquestração completa para o ciclo financeiro e comercial</h1>
          <p className="mt-4 text-sm text-slate-100/80">
            Acesse o cockpit inteligente com CRM, financeiro e operações integradas. Segurança corporativa, auditoria completa e insights em tempo real.
          </p>
        </div>
      </div>

      <div className="flex w-full max-w-lg flex-col justify-center px-8 py-12 lg:px-12">
        <LoginForm onSuccess={() => navigate("/", { replace: true })} />
      </div>
    </div>
  );
}
