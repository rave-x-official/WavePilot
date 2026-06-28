import { useState } from "react";
import { Card, CardTitle } from "../components/ui/Card";
import { Button } from "../components/ui/Button";

export function Lyrics() {
  const [content, setContent] = useState("");
  const [saved, setSaved] = useState(true);

  function handleChange(value: string) {
    setContent(value);
    setSaved(false);
  }

  function handleSave() {
    setSaved(true);
  }

  return (
    <div className="page-container">
      <div className="flex items-center justify-between mb-6">
        <h1 className="page-header mb-0">Lyrics</h1>
        <div className="flex items-center gap-3">
          {!saved && (
            <span className="text-sm text-warning">Unsaved changes</span>
          )}
          <Button onClick={handleSave} disabled={saved}>
            Save
          </Button>
        </div>
      </div>

      <Card>
        <CardTitle>Project Notes & Lyrics</CardTitle>
        <p className="text-sm text-text-muted mt-1 mb-4">
          Write lyrics and notes with Markdown support. Autosave coming soon.
        </p>
        <textarea
          className="input min-h-[400px] font-mono resize-y"
          placeholder="Start writing your lyrics or project notes here... (Markdown supported)"
          value={content}
          onChange={(e) => handleChange(e.target.value)}
        />
      </Card>
    </div>
  );
}
