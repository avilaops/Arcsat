import KPIOverview from "../../sections/dashboard/KPIOverview";
import PipelineGlance from "../../sections/dashboard/PipelineGlance";
import TeamActivities from "../../sections/dashboard/TeamActivities";

export default function DashboardPage() {
  return (
    <div className="space-y-6">
      <KPIOverview />
      <PipelineGlance />
      <TeamActivities />
    </div>
  );
}
