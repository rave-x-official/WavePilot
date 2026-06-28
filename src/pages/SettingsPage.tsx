import { Card, CardTitle, CardHeader } from "../components/ui/Card";
import { Button } from "../components/ui/Button";
import { useSettings } from "../hooks/useSettings";

export function SettingsPage() {
  const { settings, loading, updateSetting, resetSettings } = useSettings();

  if (loading) {
    return (
      <div className="page-container">
        <p className="text-text-muted">Loading settings...</p>
      </div>
    );
  }

  return (
    <div className="page-container">
      <h1 className="page-header">Settings</h1>

      <div className="space-y-4">
        <Card>
          <CardHeader>
            <CardTitle>General</CardTitle>
          </CardHeader>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-text-primary">
                  Projects Directory
                </p>
                <p className="text-xs text-text-muted">
                  Default location for importing projects
                </p>
              </div>
              <Button
                size="sm"
                variant="secondary"
                onClick={() =>
                  updateSetting(
                    "projects_directory",
                    settings.projects_directory || "",
                  )
                }
              >
                Browse
              </Button>
            </div>

            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-text-primary">
                  Default Backup Count
                </p>
                <p className="text-xs text-text-muted">
                  Keep this many recent backups
                </p>
              </div>
              <input
                type="number"
                className="input w-20 text-center"
                value={settings.default_backup_count}
                min={1}
                max={99}
                onChange={(e) =>
                  updateSetting(
                    "default_backup_count",
                    e.target.value,
                  )
                }
              />
            </div>

            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-text-primary">
                  Audio Analysis
                </p>
                <p className="text-xs text-text-muted">
                  Enable BPM and key detection
                </p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  className="sr-only peer"
                  checked={settings.analysis_enabled}
                  onChange={(e) =>
                    updateSetting(
                      "analysis_enabled",
                      String(e.target.checked),
                    )
                  }
                />
                <div className="w-9 h-5 bg-surface-border rounded-full peer peer-checked:bg-accent peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all" />
              </label>
            </div>
          </div>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Reset</CardTitle>
          </CardHeader>
          <p className="text-sm text-text-muted mb-4">
            Reset all settings to their default values. This cannot be undone.
          </p>
          <Button variant="danger" onClick={resetSettings}>
            Reset Settings
          </Button>
        </Card>
      </div>
    </div>
  );
}
