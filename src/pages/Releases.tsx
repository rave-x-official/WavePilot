import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardTitle, CardHeader } from "../components/ui/Card";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { Badge } from "../components/ui/Badge";
import { Modal } from "../components/ui/Modal";
import type {
  Project,
  ReleaseChecklist,
  ChecklistItem,
  CreateChecklistRequest,
} from "../types";

export function Releases() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [selectedProject, setSelectedProject] = useState<string>("");
  const [checklist, setChecklist] = useState<ReleaseChecklist | null>(null);
  const [allChecklists, setAllChecklists] = useState<
    (ReleaseChecklist & { project_name?: string })[]
  >([]);
  const [showNewItem, setShowNewItem] = useState(false);
  const [newItemLabel, setNewItemLabel] = useState("");

  useEffect(() => {
    invoke<Project[]>("list_projects", { query: {} }).then(setProjects).catch(console.error);
    loadAllChecklists();
  }, []);

  const loadAllChecklists = useCallback(async () => {
    try {
      const lists = await invoke<ReleaseChecklist[]>("list_checklists");
      const enriched = await Promise.all(
        lists.map(async (c) => {
          try {
            const p = await invoke<Project>("get_project", { id: c.project_id });
            return { ...c, project_name: p.name };
          } catch {
            return { ...c, project_name: "Unknown" };
          }
        }),
      );
      setAllChecklists(enriched);
    } catch (e) {
      console.error("Failed to load checklists:", e);
    }
  }, []);

  const loadChecklist = useCallback(async () => {
    if (!selectedProject) return;
    try {
      const result = await invoke<ReleaseChecklist | null>("get_checklist_for_project", {
        projectId: selectedProject,
      });
      setChecklist(result);
    } catch (e) {
      console.error("Failed to load checklist:", e);
    }
  }, [selectedProject]);

  useEffect(() => {
    loadChecklist();
  }, [loadChecklist]);

  async function handleCreateChecklist() {
    if (!selectedProject) return;
    try {
      const req: CreateChecklistRequest = { project_id: selectedProject };
      const result = await invoke<ReleaseChecklist>("create_checklist", { req });
      setChecklist(result);
      loadAllChecklists();
    } catch (e) {
      console.error("Failed to create checklist:", e);
    }
  }

  async function handleToggleItem(itemId: string) {
    if (!checklist) return;
    try {
      const currentItem = checklist.items.find((i) => i.id === itemId);
      if (!currentItem) return;
      const result = await invoke<ReleaseChecklist>("toggle_checklist_item", {
        req: {
          checklist_id: checklist.id,
          item_id: itemId,
          done: !currentItem.done,
        },
      });
      setChecklist(result);
    } catch (e) {
      console.error("Failed to toggle item:", e);
    }
  }

  async function handleAddItem() {
    if (!checklist || !newItemLabel.trim()) return;
    try {
      const result = await invoke<ReleaseChecklist>("add_checklist_item", {
        req: { checklist_id: checklist.id, label: newItemLabel.trim() },
      });
      setChecklist(result);
      setShowNewItem(false);
      setNewItemLabel("");
    } catch (e) {
      console.error("Failed to add item:", e);
    }
  }

  async function handleRemoveItem(itemId: string) {
    if (!checklist) return;
    try {
      const result = await invoke<ReleaseChecklist>("remove_checklist_item", {
        req: { checklist_id: checklist.id, item_id: itemId },
      });
      setChecklist(result);
    } catch (e) {
      console.error("Failed to remove item:", e);
    }
  }

  async function handleDeleteChecklist() {
    if (!checklist) return;
    if (!window.confirm("Delete this entire checklist?")) return;
    try {
      await invoke("delete_checklist", { id: checklist.id });
      setChecklist(null);
      loadAllChecklists();
    } catch (e) {
      console.error("Failed to delete checklist:", e);
    }
  }

  const items: ChecklistItem[] = checklist?.items || [];
  const doneCount = items.filter((i) => i.done).length;
  const progress = items.length > 0 ? Math.round((doneCount / items.length) * 100) : 0;

  return (
    <div className="page-container">
      <div className="flex items-center justify-between mb-6">
        <h1 className="page-header mb-0">Release Checklist</h1>
        {checklist && (
          <Badge variant={progress === 100 ? "success" : "default"}>
            {doneCount}/{items.length} completed
          </Badge>
        )}
      </div>

      <div className="grid grid-cols-12 gap-4 h-[calc(100vh-10rem)]">
        {/* Left: project selector + all checklists overview */}
        <div className="col-span-4 flex flex-col gap-4">
          <Card className="flex-shrink-0">
            <div className="flex flex-col gap-3">
              <div>
                <label className="text-sm font-medium text-text-secondary mb-1 block">
                  Select Project
                </label>
                <select
                  className="input w-full"
                  value={selectedProject}
                  onChange={(e) => setSelectedProject(e.target.value)}
                >
                  <option value="">Choose a project...</option>
                  {projects.map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.name} {p.artist ? `— ${p.artist}` : ""}
                    </option>
                  ))}
                </select>
              </div>
            </div>
          </Card>

          <Card className="flex-1 overflow-hidden flex flex-col">
            <CardHeader className="flex-shrink-0">
              <CardTitle className="text-sm">
                All Checklists <span className="text-text-muted font-normal ml-1">({allChecklists.length})</span>
              </CardTitle>
            </CardHeader>

            <div className="flex-1 overflow-y-auto space-y-1">
              {allChecklists.map((c) => {
                const done = c.items.filter((i) => i.done).length;
                return (
                  <div
                    key={c.id}
                    className={`px-3 py-2.5 rounded-lg cursor-pointer transition-colors ${
                      checklist?.id === c.id
                        ? "bg-accent/10 border border-accent/30"
                        : "hover:bg-surface-hover"
                    }`}
                    onClick={() => setSelectedProject(c.project_id)}
                  >
                    <p className="text-sm font-medium text-text-primary">
                      {c.project_name || "Unknown Project"}
                    </p>
                    <div className="flex items-center gap-2 mt-1">
                      <div className="flex-1 h-1.5 bg-surface-hover rounded-full overflow-hidden">
                        <div
                          className="h-full bg-accent rounded-full transition-all"
                          style={{ width: `${c.items.length > 0 ? (done / c.items.length) * 100 : 0}%` }}
                        />
                      </div>
                      <span className="text-[10px] text-text-muted">
                        {done}/{c.items.length}
                      </span>
                    </div>
                  </div>
                );
              })}

              {allChecklists.length === 0 && (
                <p className="text-sm text-text-muted text-center py-8">
                  No checklists yet.
                </p>
              )}
            </div>
          </Card>
        </div>

        {/* Right: checklist editor */}
        <div className="col-span-8">
          {selectedProject && !checklist && (
            <Card className="h-full flex items-center justify-center">
              <div className="text-center">
                <svg
                  className="w-16 h-16 mx-auto text-text-muted/30 mb-4"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={1}
                    d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4"
                  />
                </svg>
                <p className="text-text-muted mb-4">No checklist for this project yet.</p>
                <Button onClick={handleCreateChecklist}>Create Checklist</Button>
              </div>
            </Card>
          )}

          {!selectedProject && (
            <Card className="h-full flex items-center justify-center">
              <p className="text-text-muted">Select a project to view its release checklist</p>
            </Card>
          )}

          {checklist && (
            <Card className="h-full flex flex-col">
              <CardHeader className="flex-shrink-0">
                <div className="flex items-center gap-3">
                  <CardTitle>Release Checklist</CardTitle>
                  <div className="flex-1 h-2 bg-surface-hover rounded-full overflow-hidden max-w-[200px]">
                    <div
                      className={`h-full rounded-full transition-all ${
                        progress === 100 ? "bg-success" : "bg-accent"
                      }`}
                      style={{ width: `${progress}%` }}
                    />
                  </div>
                  <span className="text-sm text-text-secondary">{progress}%</span>
                </div>
                <div className="flex items-center gap-2">
                  <Button size="sm" variant="secondary" onClick={() => setShowNewItem(true)}>
                    + Item
                  </Button>
                  <Button size="sm" variant="danger" onClick={handleDeleteChecklist}>
                    Delete
                  </Button>
                </div>
              </CardHeader>

              <div className="flex-1 overflow-y-auto space-y-1">
                {items.map((item) => (
                  <div
                    key={item.id}
                    className="flex items-center gap-3 px-3 py-2.5 rounded-lg hover:bg-surface-hover group transition-colors"
                  >
                    <input
                      type="checkbox"
                      checked={item.done}
                      onChange={() => handleToggleItem(item.id)}
                      className="w-4 h-4 rounded border-surface-border bg-surface text-accent focus:ring-accent/50 focus:ring-offset-0"
                    />
                    <span
                      className={`flex-1 text-sm ${
                        item.done
                          ? "line-through text-text-muted"
                          : "text-text-primary"
                      }`}
                    >
                      {item.label}
                    </span>
                    <button
                      onClick={() => handleRemoveItem(item.id)}
                      className="opacity-0 group-hover:opacity-100 text-text-muted hover:text-danger transition-all p-1"
                      title="Remove item"
                    >
                      <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </button>
                  </div>
                ))}

                {items.length === 0 && (
                  <p className="text-sm text-text-muted text-center py-8">
                    No items yet. Add one to get started.
                  </p>
                )}
              </div>
            </Card>
          )}
        </div>
      </div>

      {/* Add Item Modal */}
      <Modal open={showNewItem} onClose={() => setShowNewItem(false)} title="Add Checklist Item">
        <div className="flex flex-col gap-4">
          <Input
            label="Item Label"
            placeholder="e.g., Master exported, Cover ready..."
            value={newItemLabel}
            onChange={(e) => setNewItemLabel(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleAddItem()}
          />
          <div className="flex justify-end gap-2 pt-2">
            <Button variant="secondary" onClick={() => setShowNewItem(false)}>
              Cancel
            </Button>
            <Button onClick={handleAddItem} disabled={!newItemLabel.trim()}>
              Add
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
