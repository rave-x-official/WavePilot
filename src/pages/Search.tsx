import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Input } from "../components/ui/Input";
import { Card } from "../components/ui/Card";
import { Badge } from "../components/ui/Badge";
import type { Project, ProjectSearchQuery } from "../types";
import { formatDate, formatBpm } from "../lib/utils";

export function Search() {
  const [query, setQuery] = useState("");
  const [artist, setArtist] = useState("");
  const [musicalKey, setMusicalKey] = useState("");
  const [results, setResults] = useState<Project[]>([]);
  const [searched, setSearched] = useState(false);

  const handleSearch = useCallback(async () => {
    const searchQuery: ProjectSearchQuery = {
      query: query || undefined,
      artist: artist || undefined,
      musical_key: musicalKey || undefined,
    };

    try {
      const result = await invoke<Project[]>("search_projects", {
        query: searchQuery,
      });
      setResults(result);
      setSearched(true);
    } catch (err) {
      console.error("Search failed:", err);
    }
  }, [query, artist, musicalKey]);

  return (
    <div className="page-container">
      <h1 className="page-header">Search Projects</h1>

      <Card className="mb-6">
        <div className="grid grid-cols-3 gap-4">
          <Input
            label="Search"
            placeholder="Name, artist, tags..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSearch()}
          />
          <Input
            label="Artist"
            placeholder="Filter by artist..."
            value={artist}
            onChange={(e) => setArtist(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSearch()}
          />
          <Input
            label="Key"
            placeholder="e.g. Cm, F#..."
            value={musicalKey}
            onChange={(e) => setMusicalKey(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSearch()}
          />
        </div>
        <div className="mt-4">
          <button
            onClick={handleSearch}
            className="btn-primary w-full"
          >
            Search
          </button>
        </div>
      </Card>

      {searched && results.length === 0 && (
        <Card>
          <p className="text-text-muted text-center py-8">
            No projects found matching your search.
          </p>
        </Card>
      )}

      <div className="grid gap-3">
        {results.map((project) => (
          <Card key={project.id} className="flex items-center justify-between">
            <div className="flex-1 min-w-0">
              <h3 className="text-base font-medium text-text-primary truncate">
                {project.name}
              </h3>
              <div className="flex items-center gap-3 mt-1 text-sm text-text-muted">
                {project.artist && <span>{project.artist}</span>}
                {project.bpm && <span>{formatBpm(project.bpm)}</span>}
                {project.musical_key && <Badge>{project.musical_key}</Badge>}
                <span>{formatDate(project.updated_at)}</span>
              </div>
            </div>
          </Card>
        ))}
      </div>
    </div>
  );
}
