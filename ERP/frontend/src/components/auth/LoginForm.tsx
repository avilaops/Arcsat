import { FormEvent, useState } from "react";
import clsx from "clsx";

import { LoginPayloadSchema } from "../../lib/api/auth";
import { useAuth } from "../../context/AuthContext";

export type LoginFormProps = {
  variant?: "default" | "compact";
  onSuccess?: () => void;
  title?: string;
  subtitle?: string;
};

export default function LoginForm({
  variant = "default",
  onSuccess,
  title = "Entrar",
  subtitle = "Insira suas credenciais corporativas para continuar."
}: LoginFormProps) {
  const { signIn, isLoading } = useAuth();
  const [formState, setFormState] = useState({
    email: "",
    password: "",
    tenant_domain: ""
  });
  const [mfaToken, setMfaToken] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = event.currentTarget;
    setFormState((prev) => ({ ...prev, [name]: value }));
  };

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setError(null);

    const payloadCandidate = { ...formState, mfa_token: mfaToken || undefined };
    const parseResult = LoginPayloadSchema.safeParse(payloadCandidate);

    if (!parseResult.success) {
      setError("Dados inválidos. Verifique os campos e tente novamente.");
      return;
    }

    setIsSubmitting(true);
    try {
      await signIn(parseResult.data);
      onSuccess?.();
    } catch (err) {
      console.error("Login failed", err);
      setError("Não foi possível autenticar. Confirme suas credenciais.");
    } finally {
      setIsSubmitting(false);
    }
  };

  const containerClasses = clsx(
    "w-full",
    variant === "compact"
      ? "max-w-md rounded-2xl border border-slate-800/60 bg-slate-900/80 p-8 shadow-xl shadow-slate-950/60"
      : "max-w-lg"
  );

  const headingWrapperClasses = clsx(
    "mb-10",
    variant === "compact" && "text-center"
  );

  const buttonLabel = isSubmitting || isLoading ? "Validando credenciais..." : "Entrar";

  return (
    <div className={containerClasses}>
      <div className={headingWrapperClasses}>
        {variant === "default" && (
          <p className="text-xs uppercase tracking-[0.3em] text-primary-300">Portal Seguro</p>
        )}
        <h2 className="mt-3 text-3xl font-semibold text-slate-100">{title}</h2>
        <p className="mt-2 text-sm text-slate-500">{subtitle}</p>
      </div>

      <form className="space-y-6" onSubmit={handleSubmit}>
        <div className="space-y-2">
          <label className="text-sm font-medium text-slate-300" htmlFor="email">
            E-mail corporativo
          </label>
          <input
            id="email"
            name="email"
            type="email"
            autoComplete="email"
            value={formState.email}
            onChange={handleChange}
            className="w-full rounded-lg border border-slate-800 bg-slate-900/80 px-4 py-3 text-sm text-slate-100 placeholder:text-slate-600 focus:border-primary-400 focus:outline-none focus:ring-2 focus:ring-primary-500/40"
            placeholder="ex: joao@empresa.com"
            required
          />
        </div>

        <div className="space-y-2">
          <label className="text-sm font-medium text-slate-300" htmlFor="password">
            Senha
          </label>
          <input
            id="password"
            name="password"
            type="password"
            autoComplete="current-password"
            value={formState.password}
            onChange={handleChange}
            className="w-full rounded-lg border border-slate-800 bg-slate-900/80 px-4 py-3 text-sm text-slate-100 placeholder:text-slate-600 focus:border-primary-400 focus:outline-none focus:ring-2 focus:ring-primary-500/40"
            placeholder="Digite sua senha"
            required
          />
        </div>

        <div className="space-y-2">
          <label className="text-sm font-medium text-slate-300" htmlFor="tenant_domain">
            Domínio do tenant
          </label>
          <input
            id="tenant_domain"
            name="tenant_domain"
            type="text"
            value={formState.tenant_domain}
            onChange={handleChange}
            className="w-full rounded-lg border border-slate-800 bg-slate-900/80 px-4 py-3 text-sm text-slate-100 placeholder:text-slate-600 focus:border-primary-400 focus:outline-none focus:ring-2 focus:ring-primary-500/40"
            placeholder="ex: fintech-xyz"
            required
          />
        </div>

        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <label className="text-sm font-medium text-slate-300" htmlFor="mfa">
              Token MFA (opcional)
            </label>
            <span className="text-xs text-slate-500">Caso o multi-fator esteja ativo</span>
          </div>
          <input
            id="mfa"
            name="mfa"
            type="text"
            value={mfaToken}
            onChange={(event) => setMfaToken(event.currentTarget.value)}
            className="w-full rounded-lg border border-slate-800 bg-slate-900/80 px-4 py-3 text-sm text-slate-100 placeholder:text-slate-600 focus:border-primary-400 focus:outline-none focus:ring-2 focus:ring-primary-500/40"
            placeholder="000000"
          />
        </div>

        {error && (
          <div className="rounded-lg border border-red-500/40 bg-red-500/10 px-4 py-3 text-sm text-red-200">
            {error}
          </div>
        )}

        <button
          type="submit"
          disabled={isSubmitting || isLoading}
          className="inline-flex w-full items-center justify-center rounded-lg bg-primary-500 px-4 py-3 text-sm font-semibold text-white shadow-lg shadow-primary-900/40 transition hover:bg-primary-400 disabled:cursor-not-allowed disabled:opacity-70"
        >
          {buttonLabel}
        </button>
      </form>

      {variant === "default" ? (
        <p className="mt-10 text-center text-xs text-slate-600">
          Ambiente monitorado. Toda tentativa de acesso é registrada para auditoria.
        </p>
      ) : (
        <p className="mt-6 text-center text-[11px] text-slate-600">
          Protegido por Avila ERP · Auditoria contínua ativada
        </p>
      )}
    </div>
  );
}
