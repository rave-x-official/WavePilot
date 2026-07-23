import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardTitle, CardHeader } from "../components/ui/Card";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { Badge } from "../components/ui/Badge";
import { Modal } from "../components/ui/Modal";
import type { Lyric, Project, CreateLyricRequest } from "../types";

const SECTIONS = ["verse", "chorus", "bridge", "outro", "intro", "pre-chorus", "hook", "other"];
const LANGUAGES = [
  { code: "en", label: "English" },
  { code: "es", label: "Spanish" },
  { code: "fr", label: "French" },
  { code: "de", label: "German" },
  { code: "pt", label: "Portuguese" },
  { code: "ja", label: "Japanese" },
  { code: "ko", label: "Korean" },
  { code: "zh", label: "Chinese" },
  { code: "", label: "Other" },
];

export function Lyrics() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [selectedProject, setSelectedProject] = useState<string>("");
  const [lyrics, setLyrics] = useState<Lyric[]>([]);
  const [selectedLyric, setSelectedLyric] = useState<Lyric | null>(null);
  const [editingContent, setEditingContent] = useState("");
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<Lyric[]>([]);
  const [showNewLyric, setShowNewLyric] = useState(false);
  const [newTitle, setNewTitle] = useState("");
  const [newSection, setNewSection] = useState("");
  const [newLanguage, setNewLanguage] = useState("");
  const [unsaved, setUnsaved] = useState(false);

  useEffect(() => {
    invoke<Project[]>("list_projects", { query: {} }).then(setProjects).catch(console.error);
  }, []);

  const loadLyrics = useCallback(async () => {
    if (!selectedProject) return;
    try {
      const result = await invoke<Lyric[]>("list_lyrics", { projectId: selectedProject });
      setLyrics(result);
    } catch (e) {
      console.error("Failed to load lyrics:", e);
    }
  }, [selectedProject]);

  useEffect(() => {
    loadLyrics();
    setSelectedLyric(null);
    setEditingContent("");
    setUnsaved(false);
  }, [loadLyrics]);

  async function handleSearch() {
    if (!searchQuery.trim()) {
      setSearchResults([]);
      return;
    }
    try {
      const results = await invoke<Lyric[]>("search_lyrics", { query: searchQuery });
      setSearchResults(results);
    } catch (e) {
      console.error("Search failed:", e);
    }
  }

  async function handleCreateLyric() {
    if (!selectedProject) return;
    try {
      const req: CreateLyricRequest = {
        project_id: selectedProject,
        content: "",
        title: newTitle || undefined,
        section: newSection || undefined,
        language: newLanguage || undefined,
      };
      const lyric = await invoke<Lyric>("create_lyric", { req });
      setLyrics((prev) => [lyric, ...prev]);
      setSelectedLyric(lyric);
      setEditingContent("");
      setShowNewLyric(false);
      setNewTitle("");
      setNewSection("");
      setNewLanguage("");
    } catch (e) {
      console.error("Failed to create lyric:", e);
    }
  }

  async function handleSave() {
    if (!selectedLyric) return;
    try {
      const updated = await invoke<Lyric>("update_lyric", {
        req: { id: selectedLyric.id, content: editingContent },
      });
      setLyrics((prev) => prev.map((l) => (l.id === updated.id ? updated : l)));
      setSelectedLyric(updated);
      setUnsaved(false);
    } catch (e) {
      console.error("Failed to save lyric:", e);
    }
  }

  async function handleDelete(lyricId: string) {
    try {
      await invoke("delete_lyric", { id: lyricId });
      setLyrics((prev) => prev.filter((l) => l.id !== lyricId));
      if (selectedLyric?.id === lyricId) {
        setSelectedLyric(null);
        setEditingContent("");
        setUnsaved(false);
      }
    } catch (e) {
      console.error("Failed to delete lyric:", e);
    }
  }

  function selectLyric(lyric: Lyric) {
    if (unsaved) {
      if (!window.confirm("You have unsaved changes. Discard them?")) return;
    }
    setSelectedLyric(lyric);
    setEditingContent(lyric.content);
    setUnsaved(false);
  }

  const selectedProjectName = projects.find((p) => p.id === selectedProject)?.name;

  return (
    <div className="page-container">
      <div className="flex items-center justify-between mb-6">
        <h1 className="page-header mb-0">Lyrics Workspace</h1>
        <div className="flex items-center gap-3">
          {unsaved && <Badge variant="warning">Unsaved</Badge>}
          {selectedLyric && (
            <Button onClick={handleSave} disabled={!unsaved}>
              Save
            </Button>
          )}
        </div>
      </div>

      <div className="grid grid-cols-12 gap-4 h-[calc(100vh-10rem)]">
        {/* Left panel: project selector + lyric list */}
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

              {selectedProject && (
                <div className="flex items-center gap-2">
                  <Input
                    placeholder="Search lyrics..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    onKeyDown={(e) => e.key === "Enter" && handleSearch()}
                    className="flex-1"
                  />
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => {
                      setSearchQuery("");
                      setSearchResults([]);
                    }}
                  >
                    Clear
                  </Button>
                </div>
              )}
            </div>
          </Card>

          {selectedProject && (
            <Card className="flex-1 overflow-hidden flex flex-col">
              <CardHeader className="flex-shrink-0">
                <CardTitle className="text-sm">
                  {searchResults.length > 0 ? "Search Results" : "Lyrics"}
                  <span className="text-text-muted font-normal ml-1">
                    ({searchResults.length > 0 ? searchResults.length : lyrics.length})
                  </span>
                </CardTitle>
                <Button size="sm" onClick={() => setShowNewLyric(true)}>
                  + New
                </Button>
              </CardHeader>

              <div className="flex-1 overflow-y-auto space-y-1">
                {(searchResults.length > 0 ? searchResults : lyrics).map((lyric) => (
                  <div
                    key={lyric.id}
                    className={`group px-3 py-2.5 rounded-lg cursor-pointer transition-colors ${
                      selectedLyric?.id === lyric.id
                        ? "bg-accent/10 border border-accent/30"
                        : "hover:bg-surface-hover"
                    }`}
                    onClick={() => selectLyric(lyric)}
                  >
                    <div className="flex items-center justify-between">
                      <div className="min-w-0">
                        <p className="text-sm font-medium text-text-primary truncate">
                          {lyric.title || "Untitled"}
                        </p>
                        <p className="text-xs text-text-muted truncate mt-0.5">
                          {lyric.content.substring(0, 60) || "Empty"}
                        </p>
                      </div>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDelete(lyric.id);
                        }}
                        className="opacity-0 group-hover:opacity-100 text-text-muted hover:text-danger transition-all p-1"
                        title="Delete"
                      >
                        <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                        </svg>
                      </button>
                    </div>
                    <div className="flex items-center gap-2 mt-1">
                      {lyric.section && (
                        <Badge variant="default" className="text-[10px]">
                          {lyric.section}
                        </Badge>
                      )}
                      {lyric.language && (
                        <span className="text-[10px] text-text-muted">
                          {LANGUAGES.find((l) => l.code === lyric.language)?.label || lyric.language}
                        </span>
                      )}
                    </div>
                  </div>
                ))}

                {lyrics.length === 0 && searchResults.length === 0 && (
                  <p className="text-sm text-text-muted text-center py-8">
                    No lyrics yet. Create one to get started.
                  </p>
                )}
              </div>
            </Card>
          )}
        </div>

        {/* Right panel: editor */}
        <div className="col-span-8">
          {selectedLyric ? (
            <Card className="h-full flex flex-col">
              <CardHeader className="flex-shrink-0">
                <div className="flex items-center gap-3">
                  <CardTitle>{selectedLyric.title || "Untitled Lyric"}</CardTitle>
                  {selectedProjectName && (
                    <Badge variant="default">{selectedProjectName}</Badge>
                  )}
                  {selectedLyric.section && (
                    <Badge variant="success">{selectedLyric.section}</Badge>
                  )}
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-xs text-text-muted">
                    Last saved: {new Date(selectedLyric.updated_at).toLocaleString()}
                  </span>
                </div>
              </CardHeader>

              <textarea
                className="input flex-1 font-mono text-sm resize-none min-h-0"
                placeholder="Write your lyrics here... (plain text)"
                value={editingContent}
                onChange={(e) => {
                  setEditingContent(e.target.value);
                  setUnsaved(true);
                }}
              />
            </Card>
          ) : (
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
                    d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                  />
                </svg>
                <p className="text-text-muted">
                  {selectedProject
                    ? "Select a lyric or create a new one"
                    : "Select a project to get started"}
                </p>
              </div>
            </Card>
          )}
        </div>
      </div>

      {/* New Lyric Modal */}
      <Modal open={showNewLyric} onClose={() => setShowNewLyric(false)} title="New Lyric">
        <div className="flex flex-col gap-4">
          <Input
            label="Title (optional)"
            placeholder="Verse 1, Chorus, etc."
            value={newTitle}
            onChange={(e) => setNewTitle(e.target.value)}
          />
          <div>
            <label className="text-sm font-medium text-text-secondary mb-1.5 block">Section</label>
            <div className="flex flex-wrap gap-2">
              {SECTIONS.map((s) => (
                <button
                  key={s}
                  onClick={() => setNewSection(newSection === s ? "" : s)}
                  className={`px-3 py-1.5 rounded-lg text-xs border transition-colors ${
                    newSection === s
                      ? "bg-accent/20 border-accent/50 text-accent"
                      : "border-surface-border text-text-secondary hover:bg-surface-hover"
                  }`}
                >
                  {s}
                </button>
              ))}
            </div>
          </div>
          <div>
            <label className="text-sm font-medium text-text-secondary mb-1.5 block">Language</label>
            <select
              className="input w-full"
              value={newLanguage}
              onChange={(e) => setNewLanguage(e.target.value)}
            >
              <option value="">Select language...</option>
              {LANGUAGES.map((l) => (
                <option key={l.code} value={l.code}>
                  {l.label}
                </option>
              ))}
            </select>
          </div>
          <div className="flex justify-end gap-2 pt-2">
            <Button variant="secondary" onClick={() => setShowNewLyric(false)}>
              Cancel
            </Button>
            <Button onClick={handleCreateLyric}>Create</Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
