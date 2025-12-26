import { Navigate, Route, Routes } from "react-router-dom";

import AppShell from "./components/layout/AppShell";
import DashboardPage from "./pages/dashboard/DashboardPage";
import LeadsPage from "./pages/crm/LeadsPage";
import AccountsPage from "./pages/crm/AccountsPage";
import ActivitiesPage from "./pages/crm/ActivitiesPage";
import ForecastPage from "./pages/crm/ForecastPage";
import LoginPage from "./pages/auth/LoginPage";

export default function App() {
  return (
    <Routes>
      <Route path="/auth/login" element={<LoginPage />} />

      <Route element={<AppShell />}>
        <Route path="/" element={<DashboardPage />} />
        <Route path="/crm/leads" element={<LeadsPage />} />
        <Route path="/crm/accounts" element={<AccountsPage />} />
        <Route path="/crm/activities" element={<ActivitiesPage />} />
        <Route path="/crm/forecast" element={<ForecastPage />} />
      </Route>

      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}
