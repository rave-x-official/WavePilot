import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Input } from "../components/ui/Input";
import { Card } from "../components/ui/Card";
import { Badge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import type { Project, ListProjectsQuery, SortField, SortOrder } from "../types";
import { formatDate, formatBpm } from "../lib/utils";

const MUSICAL_KEYS = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
const KEY_MODES = ["Major", "Minor"];

export function Search() {
  const [query, setQuery] = useState("");
  const [artist, setArtist] = useState("");
  const [musicalKey, setMusicalKey] = useState("");
  const [rootNote, setRootNote] = useState("");
  const [bpmMin, setBpmMin] = useState("");
  const [bpmMax, setBpmMax] = useState("");
  const [tagsFilter, setTagsFilter] = useState("");
  const [keywords, setKeywords] = useState("");
  const [favoriteOnly, setFavoriteOnly] = useState(false);
  const [sortBy, setSortBy] = useState<SortField>("DateAdded");
  const [sortOrder, setSortOrder] = useState<SortOrder>("Desc");
  const [showFilters, setShowFilters] = useState(false);

  const [results, setResults] = useState<Project[]>([]);
  const [searched, setSearched] = useState(false);

  const [allTags, setAllTags] = useState<string[]>([]);

  useEffect(() => {
    invoke<Project[]>("list_projects", { query: {} }).then((projects) => {
      const tagSet = new Set<string>();
      projects.forEach((p) => {
        if (p.tags) {
          try {
            const parsed = JSON.parse(p.tags) as string[];
            parsed.forEach((t) => tagSet.add(t));
          } catch { /* ignore */ }
        }
      });
      setAllTags([...tagSet].sort());
    }).catch(console.error);
  }, []);

  const handleSearch = useCallback(async () => {
    const searchQuery: ListProjectsQuery = {
      search: query || undefined,
      artist: artist || undefined,
      musical_key: musicalKey || undefined,
      root_note: rootNote || undefined,
      bpm_min: bpmMin ? parseFloat(bpmMin) : undefined,
      bpm_max: bpmMax ? parseFloat(bpmMax) : undefined,
      tags: tagsFilter
        ? tagsFilter.split(",").map((t) => t.trim()).filter(Boolean)
        : undefined,
      keywords: keywords || undefined,
      favorite_only: favoriteOnly || undefined,
      sort_by: sortBy,
      sort_order: sortOrder,
    };

    try {
      const result = await invoke<Project[]>("list_projects", {
        query: searchQuery,
      });
      setResults(result);
      setSearched(true);
    } catch (err) {
      console.error("Search failed:", err);
    }
  }, [query, artist, musicalKey, rootNote, bpmMin, bpmMax, tagsFilter, keywords, favoriteOnly, sortBy, sortOrder]);

  function handleClear() {
    setQuery("");
    setArtist("");
    setMusicalKey("");
    setRootNote("");
    setBpmMin("");
    setBpmMax("");
    setTagsFilter("");
    setKeywords("");
    setFavoriteOnly(false);
    setSortBy("DateAdded");
    setSortOrder("Desc");
    setResults([]);
    setSearched(false);
  }

  async function toggleFavorite(id: string) {
    try {
      await invoke("toggle_favorite", { id });
      setResults((prev) =>
        prev.map((p) => (p.id === id ? { ...p, favorite: !p.favorite } : p)),
      );
    } catch (e) {
      console.error("Failed to toggle favorite:", e);
    }
  }

  const activeFilterCount = [
    musicalKey,
    rootNote,
    bpmMin,
    bpmMax,
    tagsFilter,
    keywords,
    favoriteOnly,
  ].filter(Boolean).length;

  return (
    <div className="page-container">
      <h1 className="page-header">Search Projects</h1>

      <Card className="mb-6">
        <div className="grid grid-cols-12 gap-4 items-end">
          <div className="col-span-5">
            <Input
              label="Search"
              placeholder="Name, artist, keywords..."
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleSearch()}
            />
          </div>
          <div className="col-span-3">
            <Input
              label="Artist"
              placeholder="Filter by artist..."
              value={artist}
              onChange={(e) => setArtist(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleSearch()}
            />
          </div>
          <div className="col-span-2">
            <label className="text-sm font-medium text-text-secondary mb-1.5 block">Sort By</label>
            <select
              className="input w-full"
              value={sortBy}
              onChange={(e) => setSortBy(e.target.value as SortField)}
            >
              <option value="DateAdded">Date Added</option>
              <option value="Name">Name</option>
              <option value="LastOpened">Last Opened</option>
              <option value="Bpm">BPM</option>
              <option value="Artist">Artist</option>
            </select>
          </div>
          <div className="col-span-1">
            <label className="text-sm font-medium text-text-secondary mb-1.5 block">Order</label>
            <select
              className="input w-full"
              value={sortOrder}
              onChange={(e) => setSortOrder(e.target.value as SortOrder)}
            >
              <option value="Desc">↓</option>
              <option value="Asc">↑</option>
            </select>
          </div>
          <div className="col-span-1 flex gap-2">
            <Button onClick={handleSearch} className="flex-1">
              Search
            </Button>
          </div>
        </div>

        <div className="flex items-center gap-3 mt-3">
          <button
            onClick={() => setShowFilters(!showFilters)}
            className="text-sm text-accent hover:text-accent/80 transition-colors flex items-center gap-1"
          >
            <svg className={`w-4 h-4 transition-transform ${showFilters ? "rotate-180" : ""}`} fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
            </svg>
            Advanced Filters
            {activeFilterCount > 0 && (
              <Badge variant="success" className="ml-1">{activeFilterCount}</Badge>
            )}
          </button>
          {activeFilterCount > 0 && (
            <button
              onClick={handleClear}
              className="text-sm text-text-muted hover:text-danger transition-colors"
            >
              Clear all
            </button>
          )}
        </div>

        {showFilters && (
          <div className="mt-4 pt-4 border-t border-surface-border">
            <div className="grid grid-cols-4 gap-4">
              <div>
                <label className="text-sm font-medium text-text-secondary mb-1.5 block">Musical Key</label>
                <div className="flex flex-wrap gap-1.5">
                  {MUSICAL_KEYS.map((k) => (
                    <button
                      key={k}
                      onClick={() => setMusicalKey(musicalKey === k ? "" : k)}
                      className={`px-2 py-1 rounded text-xs border transition-colors ${
                        musicalKey === k
                          ? "bg-accent/20 border-accent/50 text-accent"
                          : "border-surface-border text-text-secondary hover:bg-surface-hover"
                      }`}
                    >
                      {k}
                    </button>
                  ))}
                </div>
                <div className="flex gap-1.5 mt-1.5">
                  {KEY_MODES.map((m) => (
                    <button
                      key={m}
                      onClick={() => setMusicalKey(musicalKey === m ? "" : m)}
                      className={`px-2 py-1 rounded text-xs border transition-colors ${
                        musicalKey === m
                          ? "bg-accent/20 border-accent/50 text-accent"
                          : "border-surface-border text-text-secondary hover:bg-surface-hover"
                      }`}
                    >
                      {m}
                    </button>
                  ))}
                </div>
              </div>

              <div>
                <Input
                  label="Root Note"
                  placeholder="e.g. C, F#..."
                  value={rootNote}
                  onChange={(e) => setRootNote(e.target.value)}
                />
                <div className="mt-3">
                  <Input
                    label="Keywords"
                    placeholder="Search keywords..."
                    value={keywords}
                    onChange={(e) => setKeywords(e.target.value)}
                  />
                </div>
              </div>

              <div>
                <label className="text-sm font-medium text-text-secondary mb-1.5 block">BPM Range</label>
                <div className="flex items-center gap-2">
                  <input
                    type="number"
                    className="input w-full"
                    placeholder="Min"
                    value={bpmMin}
                    onChange={(e) => setBpmMin(e.target.value)}
                    min="20"
                    max="300"
                  />
                  <span className="text-text-muted">–</span>
                  <input
                    type="number"
                    className="input w-full"
                    placeholder="Max"
                    value={bpmMax}
                    onChange={(e) => setBpmMax(e.target.value)}
                    min="20"
                    max="300"
                  />
                </div>
                <div className="mt-3">
                  <Input
                    label="Tags"
                    placeholder="Comma-separated..."
                    value={tagsFilter}
                    onChange={(e) => setTagsFilter(e.target.value)}
                  />
                  {allTags.length > 0 && (
                    <div className="flex flex-wrap gap-1 mt-1.5">
                      {allTags.slice(0, 12).map((tag) => (
                        <button
                          key={tag}
                          onClick={() => {
                            const current = tagsFilter
                              .split(",")
                              .map((t) => t.trim())
                              .filter(Boolean);
                            if (current.includes(tag)) {
                              setTagsFilter(current.filter((t) => t !== tag).join(", "));
                            } else {
                              setTagsFilter([...current, tag].join(", "));
                            }
                          }}
                          className={`px-2 py-0.5 rounded text-[10px] border transition-colors ${
                            tagsFilter.includes(tag)
                              ? "bg-accent/20 border-accent/50 text-accent"
                              : "border-surface-border text-text-muted hover:bg-surface-hover"
                          }`}
                        >
                          {tag}
                        </button>
                      ))}
                    </div>
                  )}
                </div>
              </div>

              <div>
                <label className="text-sm font-medium text-text-secondary mb-1.5 block">Filters</label>
                <label className="flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-surface-hover cursor-pointer transition-colors">
                  <input
                    type="checkbox"
                    checked={favoriteOnly}
                    onChange={(e) => setFavoriteOnly(e.target.checked)}
                    className="w-4 h-4 rounded border-surface-border bg-surface text-accent focus:ring-accent/50 focus:ring-offset-0"
                  />
                  <span className="text-sm text-text-secondary">Favorites only</span>
                </label>
              </div>
            </div>
          </div>
        )}
      </Card>

      {searched && results.length === 0 && (
        <Card>
          <p className="text-text-muted text-center py-8">
            No projects found matching your search.
          </p>
        </Card>
      )}

      {results.length > 0 && (
        <div className="mb-3 text-sm text-text-muted">
          {results.length} project{results.length !== 1 ? "s" : ""} found
        </div>
      )}

      <div className="grid gap-2">
        {results.map((project) => (
          <Card key={project.id} className="flex items-center gap-4 group hover:border-accent/30 transition-colors">
            <button
              onClick={() => toggleFavorite(project.id)}
              className="shrink-0 p-1 transition-colors"
              title={project.favorite ? "Unfavorite" : "Favorite"}
            >
              <svg
                className={`w-5 h-5 ${project.favorite ? "text-warning fill-warning" : "text-text-muted/30"}`}
                fill={project.favorite ? "currentColor" : "none"}
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={1.5}
                  d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z"
                />
              </svg>
            </button>

            <div className="flex-1 min-w-0">
              <h3 className="text-base font-medium text-text-primary truncate">
                {project.name}
              </h3>
              <div className="flex items-center gap-3 mt-1 text-sm text-text-muted">
                {project.artist && <span>{project.artist}</span>}
                {project.bpm && <span>{formatBpm(project.bpm)}</span>}
                {project.musical_key && (
                  <Badge>{project.musical_key}{project.root_note ? ` ${project.root_note}` : ""}</Badge>
                )}
                {project.daw_type && (
                  <Badge variant="default">{project.daw_type}</Badge>
                )}
              </div>
              {project.tags && (() => {
                try {
                  const parsed = JSON.parse(project.tags) as string[];
                  if (parsed.length > 0) {
                    return (
                      <div className="flex items-center gap-1.5 mt-1.5">
                        {parsed.map((tag) => (
                          <Badge key={tag} variant="default" className="text-[10px]">{tag}</Badge>
                        ))}
                      </div>
                    );
                  }
                } catch { /* ignore */ }
                return null;
              })()}
              {project.keywords && (
                <p className="text-xs text-text-muted mt-1 truncate">
                  Keywords: {project.keywords}
                </p>
              )}
            </div>

            <div className="text-right shrink-0 text-xs text-text-muted">
              {formatDate(project.updated_at)}
            </div>
          </Card>
        ))}
      </div>
    </div>
  );
}
