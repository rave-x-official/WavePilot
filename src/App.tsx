import { useState } from "react";
import { Layout } from "./components/ui/Layout";
import { Projects } from "./pages/Projects";
import { Search } from "./pages/Search";
import { Analysis } from "./pages/Analysis";
import { Lyrics } from "./pages/Lyrics";
import { Releases } from "./pages/Releases";
import { SettingsPage } from "./pages/SettingsPage";
import type { NavPage } from "./types";

function App() {
  const [activePage, setActivePage] = useState<NavPage>("projects");

  function renderPage() {
    switch (activePage) {
      case "projects":
        return <Projects />;
      case "search":
        return <Search />;
      case "analysis":
        return <Analysis />;
      case "lyrics":
        return <Lyrics />;
      case "releases":
        return <Releases />;
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
