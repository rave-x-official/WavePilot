import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "../components/ui/Button";
import { Card, CardHeader, CardTitle } from "../components/ui/Card";
import { Badge } from "../components/ui/Badge";
import { Modal } from "../components/ui/Modal";
import type {
  BackupDirectory,
  BackupScanResult,
  CleanupPreview,
  BackupHistoryEntry,
  BackupSettings,
  BackupFileEntry,
  ExecuteCleanupRequest,
} from "../types";
import { formatDate } from "../lib/utils";

type Tab = "directories" | "history";

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const val = bytes / Math.pow(1024, i);
  return `${val.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

export function BackupCleaner() {
  const [tab, setTab] = useState<Tab>("directories");
  const [directories, setDirectories] = useState<BackupDirectory[]>([]);
  const [history, setHistory] = useState<BackupHistoryEntry[]>([]);
  const [settings, setSettings] = useState<BackupSettings>({
    backups_to_keep: 5,
    min_file_age_days: 0,
    recursive_scan: true,
    confirm_before_delete: true,
  });

  const [selectedDirId, setSelectedDirId] = useState<string | null>(null);
  const [scanResult, setScanResult] = useState<BackupScanResult | null>(null);
  const [preview, setPreview] = useState<CleanupPreview | null>(null);
  const [scanning, setScanning] = useState(false);
  const [scanError, setScanError] = useState<string | null>(null);

  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [cleanupRunning, setCleanupRunning] = useState(false);
  const [cleanupResult, setCleanupResult] = useState<string | null>(null);

  const loadDirectories = useCallback(async () => {
    try {
      const result = await invoke<BackupDirectory[]>("list_backup_directories");
      setDirectories(result);
    } catch (err) {
      console.error("Failed to load directories:", err);
    }
  }, []);

  const loadHistory = useCallback(async () => {
    try {
      const result = await invoke<BackupHistoryEntry[]>(
        "get_backup_cleanup_history",
      );
      setHistory(result);
    } catch (err) {
      console.error("Failed to load history:", err);
    }
  }, []);

  useEffect(() => {
    loadDirectories();
    loadHistory();
  }, [loadDirectories, loadHistory]);

  async function handleAddDirectory() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Backup Directory",
      });
      if (!selected) return;

      await invoke("add_backup_directory", {
        req: {
          path: selected,
          label: null,
          recursive: settings.recursive_scan,
        },
      });
      loadDirectories();
    } catch (err) {
      console.error("Failed to add directory:", err);
    }
  }

  async function handleRemoveDirectory(id: string) {
    try {
      await invoke("remove_backup_directory", { id });
      if (selectedDirId === id) {
        setSelectedDirId(null);
        setScanResult(null);
        setPreview(null);
      }
      loadDirectories();
    } catch (err) {
      console.error("Failed to remove directory:", err);
    }
  }

  async function handleScan() {
    if (!selectedDirId) return;
    setScanning(true);
    setScanError(null);
    setScanResult(null);
    setPreview(null);
    try {
      const result = await invoke<BackupScanResult>("scan_backup_directory", {
        directoryId: selectedDirId,
        minFileAgeDays: settings.min_file_age_days,
      });
      setScanResult(result);

      const previewResult = await invoke<CleanupPreview>(
        "preview_backup_cleanup",
        {
          scanResult: result,
          backupsToKeep: settings.backups_to_keep,
        },
      );
      setPreview(previewResult);
    } catch (err) {
      setScanError(String(err));
    } finally {
      setScanning(false);
    }
  }

  async function handleExecuteCleanup() {
    if (!preview || !selectedDirId) return;

    if (settings.confirm_before_delete) {
      setShowDeleteConfirm(true);
      return;
    }
    await runCleanup();
  }

  async function runCleanup() {
    if (!preview || !selectedDirId) return;
    setCleanupRunning(true);
    setShowDeleteConfirm(false);
    try {
      const req: ExecuteCleanupRequest = {
        directory_id: selectedDirId,
        file_paths: preview.files_to_delete.map((f) => f.path),
      };
      const result = await invoke<{
        files_deleted: number;
        files_failed: number;
        space_freed_bytes: number;
        errors: string[];
      }>("execute_backup_cleanup", {
        directoryId: selectedDirId,
        req,
      });
      setCleanupResult(
        `Deleted ${result.files_deleted} files, freed ${formatBytes(result.space_freed_bytes)}${result.files_failed > 0 ? `. ${result.files_failed} failed.` : ""}`,
      );
      setScanResult(null);
      setPreview(null);
      loadHistory();
    } catch (err) {
      setCleanupResult(`Cleanup failed: ${err}`);
    } finally {
      setCleanupRunning(false);
    }
  }

  return (
    <div className="page-container">
      <div className="flex items-center justify-between mb-6">
        <h1 className="page-header mb-0">Backup Cleaner</h1>
        <div className="flex gap-2">
          <button
            onClick={() => setTab("directories")}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
              tab === "directories"
                ? "bg-accent text-white"
                : "bg-surface-hover text-text-secondary hover:text-text-primary"
            }`}
          >
            Directories
          </button>
          <button
            onClick={() => setTab("history")}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
              tab === "history"
                ? "bg-accent text-white"
                : "bg-surface-hover text-text-secondary hover:text-text-primary"
            }`}
          >
            History
          </button>
        </div>
      </div>

      {tab === "directories" && (
        <>
          {/* Directory Management */}
          <Card className="mb-6">
            <CardHeader>
              <CardTitle>Backup Directories</CardTitle>
              <Button onClick={handleAddDirectory}>Add Directory</Button>
            </CardHeader>

            {directories.length === 0 ? (
              <p className="text-sm text-text-muted py-4 text-center">
                No directories configured. Add a folder containing project
                backups to get started.
              </p>
            ) : (
              <div className="space-y-2">
                {directories.map((dir) => (
                  <div
                    key={dir.id}
                    className={`flex items-center justify-between p-3 rounded-lg border transition-colors cursor-pointer ${
                      selectedDirId === dir.id
                        ? "border-accent/50 bg-acent/5"
                        : "border-surface-border hover:bg-surface-hover"
                    }`}
                    onClick={() => {
                      setSelectedDirId(dir.id);
                      setScanResult(null);
                      setPreview(null);
                      setScanError(null);
                    }}
                  >
                    <div className="min-w-0 flex-1">
                      <p className="text-sm font-medium text-text-primary truncate">
                        {dir.label || dir.path}
                      </p>
                      <p className="text-xs text-text-muted truncate mt-0.5">
                        {dir.path}
                      </p>
                      <div className="flex gap-2 mt-1">
                        <Badge>{dir.recursive ? "Recursive" : "Flat"}</Badge>
                      </div>
                    </div>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleRemoveDirectory(dir.id);
                      }}
                      className="shrink-0 p-1.5 rounded-md hover:bg-danger/10 text-text-muted hover:text-danger transition-colors"
                      title="Remove directory"
                    >
                      <svg
                        className="w-4 h-4"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth={2}
                          d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                        />
                      </svg>
                    </button>
                  </div>
                ))}
              </div>
            )}
          </Card>

          {/* Scan & Clean */}
          {selectedDirId && (
            <Card className="mb-6">
              <CardHeader>
                <CardTitle>Scan & Clean</CardTitle>
                <div className="flex items-center gap-3">
                  <label className="flex items-center gap-2 text-xs text-text-secondary">
                    Min age (days):
                    <input
                      type="number"
                      className="input w-16 text-center text-xs"
                      min={0}
                      value={settings.min_file_age_days}
                      onChange={(e) =>
                        setSettings((s) => ({
                          ...s,
                          min_file_age_days: Number(e.target.value),
                        }))
                      }
                    />
                  </label>
                  <label className="flex items-center gap-2 text-xs text-text-secondary">
                    Keep:
                    <input
                      type="number"
                      className="input w-14 text-center text-xs"
                      min={1}
                      value={settings.backups_to_keep}
                      onChange={(e) =>
                        setSettings((s) => ({
                          ...s,
                          backups_to_keep: Number(e.target.value),
                        }))
                      }
                    />
                  </label>
                  <Button
                    onClick={handleScan}
                    disabled={scanning}
                    size="sm"
                  >
                    {scanning ? "Scanning..." : "Scan"}
                  </Button>
                </div>
              </CardHeader>

              {scanning && (
                <div className="flex items-center gap-3 py-6 text-text-muted">
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
                  <span className="text-sm">Scanning for backup files...</span>
                </div>
              )}

              {scanError && (
                <p className="text-sm text-danger bg-danger/5 rounded-lg px-3 py-2">
                  {scanError}
                </p>
              )}

              {scanResult && !scanning && (
                <div className="space-y-4">
                  <div className="flex gap-4 text-sm">
                    <span className="text-text-muted">
                      Found:{" "}
                      <strong className="text-text-primary">
                        {scanResult.total_files}
                      </strong>{" "}
                      files
                    </span>
                    <span className="text-text-muted">
                      Total size:{" "}
                      <strong className="text-text-primary">
                        {formatBytes(scanResult.total_size_bytes)}
                      </strong>
                    </span>
                    {scanResult.skipped_count > 0 && (
                      <span className="text-text-muted">
                        Skipped:{" "}
                        <strong className="text-warning">
                          {scanResult.skipped_count}
                        </strong>
                      </span>
                    )}
                  </div>

                  {preview && (
                    <div className="border border-surface-border rounded-lg overflow-hidden">
                      <div className="bg-surface-hover px-4 py-2 text-sm font-medium text-text-primary flex justify-between">
                        <span>
                          Cleanup Preview — {preview.total_files} files to
                          delete ({formatBytes(preview.total_size_bytes)})
                        </span>
                        <span className="text-text-muted">
                          {preview.kept_files} files will be kept
                        </span>
                      </div>

                      {preview.files_to_delete.length > 50 ? (
                        <div className="p-4 text-sm text-text-muted">
                          Showing first 50 of {preview.files_to_delete.length}{" "}
                          files...
                          <div className="mt-2 space-y-1 max-h-48 overflow-y-auto">
                            {preview.files_to_delete
                              .slice(0, 50)
                              .map((f, i) => (
                                <FileRow key={i} file={f} />
                              ))}
                          </div>
                        </div>
                      ) : (
                        <div className="divide-y divide-surface-border max-h-64 overflow-y-auto">
                          {preview.files_to_delete.map((f, i) => (
                            <FileRow key={i} file={f} />
                          ))}
                        </div>
                      )}

                      <div className="px-4 py-3 bg-surface-hover flex justify-end gap-3">
                        <Button
                          variant="secondary"
                          onClick={() => {
                            setScanResult(null);
                            setPreview(null);
                          }}
                        >
                          Cancel
                        </Button>
                        <Button
                          variant="danger"
                          onClick={handleExecuteCleanup}
                          disabled={cleanupRunning}
                        >
                          {cleanupRunning
                            ? "Deleting..."
                            : `Delete ${preview.total_files} files`}
                        </Button>
                      </div>
                    </div>
                  )}

                  {cleanupResult && (
                    <div
                      className={`px-4 py-3 rounded-lg text-sm ${
                        cleanupResult.includes("failed")
                          ? "bg-danger/5 text-danger"
                          : "bg-success/5 text-success"
                      }`}
                    >
                      {cleanupResult}
                      <button
                        className="ml-3 underline"
                        onClick={() => setCleanupResult(null)}
                      >
                        Dismiss
                      </button>
                    </div>
                  )}
                </div>
              )}
            </Card>
          )}

          {/* Settings summary */}
          <Card>
            <CardTitle>Settings</CardTitle>
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-4 mt-3">
              <div>
                <p className="text-xs text-text-muted">Backups to keep</p>
                <p className="text-sm font-medium text-text-primary">
                  {settings.backups_to_keep}
                </p>
              </div>
              <div>
                <p className="text-xs text-text-muted">Min file age</p>
                <p className="text-sm font-medium text-text-primary">
                  {settings.min_file_age_days > 0
                    ? `${settings.min_file_age_days} days`
                    : "None"}
                </p>
              </div>
              <div>
                <p className="text-xs text-text-muted">Recursive scan</p>
                <p className="text-sm font-medium text-text-primary">
                  {settings.recursive_scan ? "On" : "Off"}
                </p>
              </div>
              <div>
                <p className="text-xs text-text-muted">Ask before delete</p>
                <p className="text-sm font-medium text-text-primary">
                  {settings.confirm_before_delete ? "On" : "Off"}
                </p>
              </div>
            </div>
          </Card>
        </>
      )}

      {tab === "history" && (
        <Card>
          <CardHeader>
            <CardTitle>Cleanup History</CardTitle>
          </CardHeader>

          {history.length === 0 ? (
            <p className="text-sm text-text-muted py-6 text-center">
              No cleanup history yet. Run a cleanup to see results here.
            </p>
          ) : (
            <div className="space-y-2">
              {history.map((entry) => (
                <div
                  key={entry.id}
                  className="flex items-center justify-between p-3 rounded-lg border border-surface-border"
                >
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-text-primary">
                      {entry.directory_path || "Unknown directory"}
                    </p>
                    <p className="text-xs text-text-muted mt-0.5">
                      {formatDate(entry.scanned_at)} — {entry.files_deleted}{" "}
                      files deleted, {formatBytes(entry.space_freed_bytes)}{" "}
                      freed
                    </p>
                  </div>
                  <Badge
                    variant={
                      entry.status === "completed" ? "success" : "warning"
                    }
                  >
                    {entry.status}
                  </Badge>
                </div>
              ))}
            </div>
          )}
        </Card>
      )}

      {/* Delete confirmation modal */}
      <Modal
        open={showDeleteConfirm}
        onClose={() => setShowDeleteConfirm(false)}
        title="Confirm Cleanup"
      >
        <p className="text-sm text-text-secondary mb-4">
          You are about to permanently delete{" "}
          <strong className="text-text-primary">
            {preview?.total_files ?? 0}
          </strong>{" "}
          backup files, freeing approximately{" "}
          <strong className="text-text-primary">
            {formatBytes(preview?.total_size_bytes ?? 0)}
          </strong>
          . This action cannot be undone.
        </p>
        <div className="flex justify-end gap-3">
          <Button
            variant="secondary"
            onClick={() => setShowDeleteConfirm(false)}
          >
            Cancel
          </Button>
          <Button variant="danger" onClick={runCleanup} disabled={cleanupRunning}>
            {cleanupRunning ? "Deleting..." : "Delete Files"}
          </Button>
        </div>
      </Modal>
    </div>
  );
}

function FileRow({ file }: { file: BackupFileEntry }) {
  return (
    <div className="flex items-center gap-3 px-4 py-2 text-sm hover:bg-surface-hover">
      <svg
        className="w-4 h-4 shrink-0 text-text-muted"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={1.5}
          d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
        />
      </svg>
      <div className="flex-1 min-w-0">
        <p className="text-text-primary truncate" title={file.path}>
          {file.name}
        </p>
        <p className="text-xs text-text-muted truncate">{file.path}</p>
      </div>
      <div className="text-right shrink-0">
        <p className="text-text-muted">{formatBytes(file.size_bytes)}</p>
        {file.parent_project && (
          <p className="text-xs text-text-muted">{file.parent_project}</p>
        )}
      </div>
    </div>
  );
}
