import { Sidebar } from "./Sidebar";
import type { NavPage } from "../../types";

interface LayoutProps {
  activePage: NavPage;
  onNavigate: (page: NavPage) => void;
  children: React.ReactNode;
}

export function Layout({ activePage, onNavigate, children }: LayoutProps) {
  return (
    <div className="flex h-screen overflow-hidden">
      <Sidebar activePage={activePage} onNavigate={onNavigate} />
      <main className="flex-1 overflow-y-auto bg-surface">
        {children}
      </main>
    </div>
  );
}
