import React from "react";
import ReactDOM from "react-dom/client";
import type { Root } from "react-dom/client";

import EmbedAuthProvider from "./auth-context-provider";
import LoginForm from "../components/auth/LoginForm";
import { configureApiClient } from "../lib/api/client";
import "./widget.css";

export type LoginWidgetOptions = {
  title?: string;
  subtitle?: string;
  onSuccess?: () => void;
  apiBaseUrl?: string;
};

const mountedRoots = new WeakMap<HTMLElement, Root>();

export function renderLoginWidget(container: HTMLElement, options: LoginWidgetOptions = {}) {
  const { title, subtitle, onSuccess, apiBaseUrl } = options;

  if (!container) {
    throw new Error("Container element is required to render the login widget.");
  }

  if (apiBaseUrl) {
    configureApiClient({ baseURL: apiBaseUrl });
  }

  let root = mountedRoots.get(container);
  if (!root) {
    root = ReactDOM.createRoot(container);
    mountedRoots.set(container, root);
  }

  root.render(
    <React.StrictMode>
      <EmbedAuthProvider>
        <div className="avila-erp-widget-surface bg-slate-950/95 p-6 text-slate-100">
          <LoginForm
            variant="compact"
            title={title ?? "Acesso seguro"}
            subtitle={subtitle ?? "Preencha os dados corporativos para entrar."}
            onSuccess={onSuccess}
          />
        </div>
      </EmbedAuthProvider>
    </React.StrictMode>
  );

  return {
    destroy() {
      root?.unmount();
      mountedRoots.delete(container);
    }
  };
}

function ensureNamespace() {
  if (!window.AvilaERP) {
    window.AvilaERP = {} as AvilaERPGlobal;
  }
  window.AvilaERP.renderLoginWidget = renderLoginWidget;
}

declare global {
  interface AvilaERPGlobal {
    renderLoginWidget: typeof renderLoginWidget;
  }

  interface Window {
    AvilaERP?: AvilaERPGlobal;
  }
}
ensureNamespace();
