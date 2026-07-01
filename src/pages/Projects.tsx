import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { Badge } from "../components/ui/Badge";
import { Modal } from "../components/ui/Modal";
import type {
  Project,
  ImportProjectRequest,
  ListProjectsQuery,
  SortField,
  SortOrder,
  ViewMode,
} from "../types";
import { formatDate, formatBpm } from "../lib/utils";

type PageState = "loading" | "loaded" | "error";

export function Projects() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [state, setState] = useState<PageState>("loading");
  const [error, setError] = useState<string | null>(null);

  const [view, setView] = useState<ViewMode>("Grid");
  const [search, setSearch] = useState("");
  const [sortBy, setSortBy] = useState<SortField>("DateAdded");
  const [sortOrder, setSortOrder] = useState<SortOrder>("Desc");
  const [favoriteOnly, setFavoriteOnly] = useState(false);

  const [showImport, setShowImport] = useState(false);
  const [importName, setImportName] = useState("");
  const [importPath, setImportPath] = useState("");
  const [importArtist, setImportArtist] = useState("");
  const [importTags, setImportTags] = useState("");
  const [importing, setImporting] = useState(false);
  const [importError, setImportError] = useState<string | null>(null);

  const loadProjects = useCallback(async () => {
    setState("loading");
    setError(null);
    try {
      const query: ListProjectsQuery = {
        search: search || undefined,
        sort_by: sortBy,
        sort_order: sortOrder,
        favorite_only: favoriteOnly || undefined,
      };
      const result = await invoke<Project[]>("list_projects", { query });
      setProjects(result);
      setState("loaded");
    } catch (err) {
      setError(String(err));
      setState("error");
    }
  }, [search, sortBy, sortOrder, favoriteOnly]);

  useEffect(() => {
    loadProjects();
  }, [loadProjects]);

  async function handlePickFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Project Folder",
      });
      if (selected) {
        setImportPath(selected);
        if (!importName) {
          const parts = selected.replace(/\\/g, "/").split("/");
          const folderName = parts[parts.length - 1] || parts[parts.length - 2] || "";
          setImportName(folderName);
        }
      }
    } catch (err) {
      setImportError(String(err));
    }
  }

  async function handleImport() {
    if (!importName.trim() || !importPath.trim()) return;
    setImporting(true);
    setImportError(null);
    try {
      const tags = importTags
        .split(",")
        .map((t) => t.trim())
        .filter(Boolean);
      const req: ImportProjectRequest = {
        name: importName.trim(),
        path: importPath.trim(),
        artist: importArtist.trim() || undefined,
        tags: tags.length > 0 ? tags : undefined,
      };
      await invoke("import_project", { req });
      setShowImport(false);
      resetImportForm();
      loadProjects();
    } catch (err) {
      setImportError(String(err));
    } finally {
      setImporting(false);
    }
  }

  function resetImportForm() {
    setImportName("");
    setImportPath("");
    setImportArtist("");
    setImportTags("");
    setImportError(null);
  }

  async function handleToggleFavorite(id: string) {
    try {
      const isFav = await invoke<boolean>("toggle_favorite", { id });
      setProjects((prev) =>
        prev.map((p) => (p.id === id ? { ...p, favorite: isFav } : p)),
      );
    } catch (err) {
      console.error("Failed to toggle favorite:", err);
    }
  }

  async function handleDelete(id: string, name: string) {
    if (!window.confirm(`Delete "${name}"? This cannot be undone.`)) return;
    try {
      await invoke("delete_project", { id });
      loadProjects();
    } catch (err) {
      console.error("Failed to delete project:", err);
    }
  }

  function activeSort(label: string, field: SortField) {
    const isActive = sortBy === field;
    return (
      <button
        key={field}
        onClick={() => {
          if (isActive) {
            setSortOrder((o) => (o === "Asc" ? "Desc" : "Asc"));
          } else {
            setSortBy(field);
            setSortOrder("Desc");
          }
        }}
        className={`px-3 py-1.5 rounded-md text-xs font-medium transition-colors ${
          isActive
            ? "bg-accent/10 text-accent"
            : "text-text-secondary hover:text-text-primary hover:bg-surface-hover"
        }`}
      >
        {label}
        {isActive && (sortOrder === "Asc" ? " ↑" : " ↓")}
      </button>
    );
  }

  function renderGrid() {
    if (projects.length === 0) {
      return (
        <Card>
          <div className="flex flex-col items-center justify-center py-16 text-center">
            <svg
              className="w-16 h-16 text-text-muted mb-4"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1}
                d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
              />
            </svg>
            <h3 className="text-lg font-medium text-text-primary mb-1">
              No projects yet
            </h3>
            <p className="text-sm text-text-muted mb-4 max-w-sm">
              Import a music project folder to get started. WavePilot will index
              and organize your projects.
            </p>
            <Button onClick={() => setShowImport(true)}>
              Import Your First Project
            </Button>
          </div>
        </Card>
      );
    }

    return (
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        {projects.map((project) => (
          <Card key={project.id} className="group relative">
            <div className="flex items-start justify-between mb-3">
              <div className="min-w-0 flex-1">
                <h3 className="font-medium text-text-primary truncate pr-6">
                  {project.name}
                </h3>
                {project.artist && (
                  <p className="text-sm text-text-muted truncate">
                    {project.artist}
                  </p>
                )}
              </div>
              <button
                onClick={() => handleToggleFavorite(project.id)}
                className="shrink-0 p-1 -mr-1 -mt-1 rounded-md hover:bg-surface-hover transition-colors"
                title={project.favorite ? "Remove from favorites" : "Add to favorites"}
              >
                <svg
                  className={`w-5 h-5 ${
                    project.favorite
                      ? "text-warning fill-warning"
                      : "text-text-muted"
                  }`}
                  fill={project.favorite ? "currentColor" : "none"}
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z"
                  />
                </svg>
              </button>
            </div>

            <div className="flex flex-wrap items-center gap-2 mb-3">
              {project.bpm && <Badge>{formatBpm(project.bpm)}</Badge>}
              {project.musical_key && <Badge>{project.musical_key}</Badge>}
              {project.daw_type && <Badge variant="warning">{project.daw_type}</Badge>}
            </div>

            <p className="text-xs text-text-muted truncate mb-1" title={project.path}>
              {project.path}
            </p>
            <p className="text-xs text-text-muted">
              Added {formatDate(project.created_at)}
            </p>

            <button
              onClick={() => handleDelete(project.id, project.name)}
              className="absolute top-3 right-10 p-1 rounded-md opacity-0 group-hover:opacity-100 hover:bg-danger/10 text-text-muted hover:text-danger transition-all"
              title="Delete project"
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
            </button>
          </Card>
        ))}
      </div>
    );
  }

  function renderList() {
    if (projects.length === 0) {
      return renderGrid();
    }

    return (
      <div className="space-y-1">
        {projects.map((project) => (
          <div
            key={project.id}
            className="flex items-center gap-4 px-4 py-3 rounded-lg hover:bg-surface-hover transition-colors group"
          >
            <button
              onClick={() => handleToggleFavorite(project.id)}
              className="shrink-0"
              title={project.favorite ? "Remove from favorites" : "Add to favorites"}
            >
              <svg
                className={`w-4 h-4 ${
                  project.favorite
                    ? "text-warning fill-warning"
                    : "text-text-muted"
                }`}
                fill={project.favorite ? "currentColor" : "none"}
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z"
                />
              </svg>
            </button>

            <div className="flex-1 min-w-0 grid grid-cols-12 gap-4 items-center">
              <div className="col-span-4">
                <p className="text-sm font-medium text-text-primary truncate">
                  {project.name}
                </p>
              </div>
              <div className="col-span-2">
                <p className="text-sm text-text-muted truncate">
                  {project.artist || "--"}
                </p>
              </div>
              <div className="col-span-2">
                <p className="text-sm text-text-muted">
                  {project.bpm ? formatBpm(project.bpm) : "--"}
                </p>
              </div>
              <div className="col-span-2">
                <div className="flex gap-1 flex-wrap">
                  {project.musical_key && <Badge>{project.musical_key}</Badge>}
                  {project.daw_type && (
                    <Badge variant="warning">{project.daw_type}</Badge>
                  )}
                </div>
              </div>
              <div className="col-span-2 text-right">
                <p className="text-xs text-text-muted">
                  {formatDate(project.created_at)}
                </p>
              </div>
            </div>

            <button
              onClick={() => handleDelete(project.id, project.name)}
              className="shrink-0 p-1 rounded-md opacity-0 group-hover:opacity-100 hover:bg-danger/10 text-text-muted hover:text-danger transition-all"
              title="Delete project"
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
            </button>
          </div>
        ))}
      </div>
    );
  }

  return (
    <div className="page-container">
      <div className="flex items-center justify-between mb-6">
        <h1 className="page-header mb-0">Projects</h1>
        <Button onClick={() => setShowImport(true)}>Import Project</Button>
      </div>

      <div className="flex flex-col sm:flex-row items-start sm:items-center gap-3 mb-6">
        <div className="relative flex-1 w-full max-w-md">
          <svg
            className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-text-muted pointer-events-none"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
            />
          </svg>
          <input
            className="input pl-9"
            placeholder="Search projects..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>

        <div className="flex items-center gap-2 flex-wrap">
          {activeSort("Name", "Name")}
          {activeSort("Date", "DateAdded")}
          {activeSort("BPM", "Bpm")}
          {activeSort("Artist", "Artist")}
        </div>

        <div className="flex items-center gap-2 ml-auto">
          <label className="flex items-center gap-2 text-sm text-text-secondary cursor-pointer">
            <input
              type="checkbox"
              checked={favoriteOnly}
              onChange={(e) => setFavoriteOnly(e.target.checked)}
              className="w-4 h-4 rounded border-surface-border bg-surface text-accent focus:ring-accent/50 focus:ring-offset-0"
            />
            Favorites
          </label>

          <div className="flex border border-surface-border rounded-lg overflow-hidden">
            <button
              onClick={() => setView("Grid")}
              className={`p-2 ${
                view === "Grid"
                  ? "bg-accent text-white"
                  : "bg-surface-alt text-text-muted hover:text-text-primary"
              } transition-colors`}
              title="Grid view"
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
              </svg>
            </button>
            <button
              onClick={() => setView("List")}
              className={`p-2 ${
                view === "List"
                  ? "bg-accent text-white"
                  : "bg-surface-alt text-text-muted hover:text-text-primary"
              } transition-colors`}
              title="List view"
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
              </svg>
            </button>
          </div>
        </div>
      </div>

      {state === "loading" && projects.length === 0 && (
        <div className="flex items-center justify-center py-16">
          <div className="flex items-center gap-3 text-text-muted">
            <svg
              className="w-5 h-5 animate-spin"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                className="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="4"
              />
              <path
                className="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
              />
            </svg>
            <span className="text-sm">Loading projects...</span>
          </div>
        </div>
      )}

      {state === "loaded" && (view === "Grid" ? renderGrid() : renderList())}

      {error && (
        <Card className="border-danger/30 bg-danger/5 mt-4">
          <p className="text-sm text-danger">{error}</p>
        </Card>
      )}

      <Modal
        open={showImport}
        onClose={() => {
          setShowImport(false);
          resetImportForm();
        }}
        title="Import Project"
      >
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-text-secondary mb-1.5">
              Project Folder <span className="text-danger">*</span>
            </label>
            <div className="flex gap-2">
              <input
                className="input flex-1"
                placeholder="Click Browse to select..."
                value={importPath}
                onChange={(e) => setImportPath(e.target.value)}
              />
              <Button variant="secondary" onClick={handlePickFolder}>
                Browse
              </Button>
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-text-secondary mb-1.5">
              Project Name <span className="text-danger">*</span>
            </label>
            <input
              className="input"
              placeholder="My Awesome Track"
              value={importName}
              onChange={(e) => setImportName(e.target.value)}
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-text-secondary mb-1.5">
              Artist
            </label>
            <input
              className="input"
              placeholder="Artist name"
              value={importArtist}
              onChange={(e) => setImportArtist(e.target.value)}
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-text-secondary mb-1.5">
              Tags (comma-separated)
            </label>
            <input
              className="input"
              placeholder="electronic, ambient, experimental"
              value={importTags}
              onChange={(e) => setImportTags(e.target.value)}
            />
          </div>

          {importError && (
            <p className="text-sm text-danger bg-danger/5 rounded-lg px-3 py-2">
              {importError}
            </p>
          )}

          <div className="flex justify-end gap-3 pt-2">
            <Button
              variant="secondary"
              onClick={() => {
                setShowImport(false);
                resetImportForm();
              }}
            >
              Cancel
            </Button>
            <Button
              onClick={handleImport}
              disabled={!importName.trim() || !importPath.trim() || importing}
            >
              {importing ? "Importing..." : "Import"}
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
