import { PropsWithChildren } from "react";
import { Outlet } from "react-router-dom";

import Sidebar from "./Sidebar";
import Topbar from "./Topbar";

export default function AppShell({ children }: PropsWithChildren) {
  return (
    <div className="min-h-screen bg-slate-950 text-slate-100 flex">
      <Sidebar />
      <div className="flex-1 flex flex-col min-w-0">
        <Topbar />
        <main className="flex-1 overflow-y-auto bg-slate-900 px-8 py-6">
          <div className="max-w-7xl mx-auto w-full space-y-6">{children ?? <Outlet />}</div>
        </main>
      </div>
    </div>
  );
}
