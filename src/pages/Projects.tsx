import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { Card } from "../components/ui/Card";
import { Badge } from "../components/ui/Badge";
import type { Project, ImportProjectRequest } from "../types";
import { formatDate, formatBpm } from "../lib/utils";

export function Projects() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);
  const [showImport, setShowImport] = useState(false);
  const [importForm, setImportForm] = useState<ImportProjectRequest>({
    name: "",
    path: "",
  });

  const loadProjects = useCallback(async () => {
    try {
      const result = await invoke<Project[]>("list_projects");
      setProjects(result);
    } catch (err) {
      console.error("Failed to load projects:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadProjects();
  }, [loadProjects]);

  const handleImport = async () => {
    if (!importForm.name || !importForm.path) return;
    try {
      await invoke("import_project", { req: importForm });
      setImportForm({ name: "", path: "" });
      setShowImport(false);
      loadProjects();
    } catch (err) {
      console.error("Failed to import project:", err);
    }
  };

  if (loading) {
    return (
      <div className="page-container">
        <p className="text-text-muted">Loading projects...</p>
      </div>
    );
  }

  return (
    <div className="page-container">
      <div className="flex items-center justify-between mb-6">
        <h1 className="page-header mb-0">Projects</h1>
        <Button onClick={() => setShowImport(!showImport)}>
          {showImport ? "Cancel" : "Import Project"}
        </Button>
      </div>

      {showImport && (
        <Card className="mb-6">
          <div className="flex items-end gap-4">
            <div className="flex-1">
              <Input
                label="Project Name"
                value={importForm.name}
                onChange={(e) =>
                  setImportForm((f) => ({ ...f, name: e.target.value }))
                }
              />
            </div>
            <div className="flex-1">
              <Input
                label="Project Path"
                value={importForm.path}
                onChange={(e) =>
                  setImportForm((f) => ({ ...f, path: e.target.value }))
                }
              />
            </div>
            <Button onClick={handleImport}>Import</Button>
          </div>
        </Card>
      )}

      {projects.length === 0 ? (
        <Card>
          <p className="text-text-muted text-center py-8">
            No projects yet. Import a project to get started.
          </p>
        </Card>
      ) : (
        <div className="grid gap-3">
          {projects.map((project) => (
            <Card key={project.id} className="flex items-center justify-between">
              <div className="flex-1 min-w-0">
                <h3 className="text-base font-medium text-text-primary truncate">
                  {project.name}
                </h3>
                <div className="flex items-center gap-3 mt-1 text-sm text-text-muted">
                  {project.artist && <span>{project.artist}</span>}
                  {project.bpm && <span>{formatBpm(project.bpm)}</span>}
                  {project.musical_key && (
                    <Badge>{project.musical_key}</Badge>
                  )}
                  <span>{formatDate(project.updated_at)}</span>
                </div>
                <p className="text-xs text-text-muted mt-1 truncate">
                  {project.path}
                </p>
              </div>
              <div className="flex items-center gap-2 ml-4">
                {project.daw_type && <Badge>{project.daw_type}</Badge>}
              </div>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
