import { useState } from "react";
import { Layout } from "./components/ui/Layout";
import { Projects } from "./pages/Projects";
import { Search } from "./pages/Search";
import { Analysis } from "./pages/Analysis";
import { Lyrics } from "./pages/Lyrics";
import { Releases } from "./pages/Releases";
import { BackupCleaner } from "./pages/BackupCleaner";
import { SettingsPage } from "./pages/SettingsPage";
import type { NavPage } from "./types";

function App() {
  const [activePage, setActivePage] = useState<NavPage>("projects");
  const [analysisProjectId, setAnalysisProjectId] = useState<string | null>(null);

  function navigateToAnalysis(projectId: string) {
    setAnalysisProjectId(projectId);
    setActivePage("analysis");
  }

  function renderPage() {
    switch (activePage) {
      case "projects":
        return <Projects onNavigateToAnalysis={navigateToAnalysis} />;
      case "search":
        return <Search />;
      case "analysis":
        return (
          <Analysis
            selectedProjectId={analysisProjectId}
            onClearSelected={() => setAnalysisProjectId(null)}
          />
        );
      case "lyrics":
        return <Lyrics />;
      case "releases":
        return <Releases />;
      case "backup":
        return <BackupCleaner />;
      case "settings":
        return <SettingsPage />;
    }
  }

  return (
    <Layout activePage={activePage} onNavigate={setActivePage}>
      {renderPage()}
    </Layout>
  );
}

export default App;
