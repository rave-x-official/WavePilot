import { useState } from "react";
import { Card, CardTitle, CardHeader } from "../components/ui/Card";
import { Button } from "../components/ui/Button";
import { Badge } from "../components/ui/Badge";

const defaultChecklist = [
  { id: "1", label: "Mix finished", done: false },
  { id: "2", label: "Master exported", done: false },
  { id: "3", label: "Cover ready", done: false },
  { id: "4", label: "Metadata completed", done: false },
  { id: "5", label: "Uploaded", done: false },
  { id: "6", label: "Scheduled", done: false },
  { id: "7", label: "Released", done: false },
];

export function Releases() {
  const [items, setItems] = useState(defaultChecklist);

  function toggleItem(id: string) {
    setItems((prev) =>
      prev.map((item) =>
        item.id === id ? { ...item, done: !item.done } : item,
      ),
    );
  }

  const doneCount = items.filter((i) => i.done).length;

  return (
    <div className="page-container">
      <div className="flex items-center justify-between mb-6">
        <h1 className="page-header mb-0">Release Checklist</h1>
        <Badge variant={doneCount === items.length ? "success" : "default"}>
          {doneCount}/{items.length} completed
        </Badge>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Release Checklist</CardTitle>
          <Button
            size="sm"
            variant="secondary"
            onClick={() => setItems(defaultChecklist.map((i) => ({ ...i, done: false })))}
          >
            Reset
          </Button>
        </CardHeader>

        <div className="space-y-2">
          {items.map((item) => (
            <label
              key={item.id}
              className="flex items-center gap-3 px-3 py-2.5 rounded-lg hover:bg-surface-hover cursor-pointer transition-colors"
            >
              <input
                type="checkbox"
                checked={item.done}
                onChange={() => toggleItem(item.id)}
                className="w-4 h-4 rounded border-surface-border bg-surface text-accent focus:ring-accent/50 focus:ring-offset-0"
              />
              <span
                className={`text-sm ${
                  item.done
                    ? "line-through text-text-muted"
                    : "text-text-primary"
                }`}
              >
                {item.label}
              </span>
            </label>
          ))}
        </div>
      </Card>
    </div>
  );
}
