import { useState, useRef } from "react";
import { Card, CardTitle } from "../components/ui/Card";
import { Button } from "../components/ui/Button";
import { Badge } from "../components/ui/Badge";
import { invoke } from "@tauri-apps/api/core";
import { AnalysisResult, AudioInfo, LoudnessResult } from "../types";

function formatDuration(secs: number): string {
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  return `${m}m ${s}s`;
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatLoudness(v: number | null | undefined): string {
  if (v == null || !isFinite(v)) return "—";
  return v.toFixed(1);
}

function loudnessColor(lufs: number | null | undefined): string {
  if (lufs == null || !isFinite(lufs)) return "text-text-muted";
  if (lufs > -9) return "text-green-400";
  if (lufs > -14) return "text-yellow-400";
  if (lufs > -20) return "text-orange-400";
  return "text-red-400";
}

function loudnessLabel(lufs: number | null | undefined): string {
  if (lufs == null || !isFinite(lufs)) return "N/A";
  if (lufs > -9) return "Loud";
  if (lufs > -14) return "Moderate";
  if (lufs > -20) return "Quiet";
  return "Very Quiet";
}

function peakColor(db: number | null | undefined): string {
  if (db == null || !isFinite(db)) return "text-text-muted";
  if (db > -1) return "text-red-400";
  if (db > -6) return "text-yellow-400";
  return "text-green-400";
}

export function Analysis({
  selectedProjectId,
  onClearSelected,
}: {
  selectedProjectId?: string | null;
  onClearSelected?: () => void;
}) {
  const [result, setResult] = useState<AnalysisResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  async function handleAnalyze(filePath: string) {
    setLoading(true);
    setError(null);
    setResult(null);

    try {
      const res = await invoke<AnalysisResult>("analyze_audio", {
        request: {
          project_id: selectedProjectId || "direct",
          file_path: filePath,
        },
      });
      setResult(res);
    } catch (e: any) {
      setError(typeof e === "string" ? e : e?.message || "Analysis failed");
    } finally {
      setLoading(false);
    }
  }

  async function handleFileSelect() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "Audio",
            extensions: ["wav", "mp3", "flac", "ogg", "aiff"],
          },
        ],
      });
      if (selected) {
        await handleAnalyze(selected);
      }
    } catch {
      fileInputRef.current?.click();
    }
  }

  function handleNativeFileChange(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (file) {
      handleAnalyze(file.name);
    }
  }

  function renderAudioInfo(info: AudioInfo) {
    return (
      <div className="grid grid-cols-2 md:grid-cols-5 gap-3">
        <Card>
          <p className="text-xs text-text-muted">Duration</p>
          <p className="text-lg font-semibold text-text-primary">
            {formatDuration(info.duration_secs)}
          </p>
        </Card>
        <Card>
          <p className="text-xs text-text-muted">Sample Rate</p>
          <p className="text-lg font-semibold text-text-primary">
            {info.sample_rate / 1000} kHz
          </p>
        </Card>
        <Card>
          <p className="text-xs text-text-muted">Bit Depth</p>
          <p className="text-lg font-semibold text-text-primary">
            {info.bit_depth}-bit
          </p>
        </Card>
        <Card>
          <p className="text-xs text-text-muted">Channels</p>
          <p className="text-lg font-semibold text-text-primary">
            {info.channels === 1 ? "Mono" : info.channels === 2 ? "Stereo" : `${info.channels}`}
          </p>
        </Card>
        <Card>
          <p className="text-xs text-text-muted">File Size</p>
          <p className="text-lg font-semibold text-text-primary">
            {formatFileSize(info.file_size)}
          </p>
        </Card>
      </div>
    );
  }

  function renderLoudness(l: LoudnessResult) {
    const bars = [
      {
        label: "Integrated LUFS",
        value: l.integrated_lufs,
        colorClass: loudnessColor(l.integrated_lufs),
        badge: loudnessLabel(l.integrated_lufs),
        min: -30,
        max: 0,
      },
      {
        label: "Short-Term LUFS",
        value: l.short_term_lufs,
        colorClass: loudnessColor(l.short_term_lufs),
        badge: loudnessLabel(l.short_term_lufs),
        min: -30,
        max: 0,
      },
      {
        label: "Momentary LUFS",
        value: l.momentary_lufs,
        colorClass: loudnessColor(l.momentary_lufs),
        badge: loudnessLabel(l.momentary_lufs),
        min: -30,
        max: 0,
      },
    ];

    function pct(v: number, min: number, max: number) {
      const clamped = Math.max(min, Math.min(max, v));
      return ((clamped - min) / (max - min)) * 100;
    }

    return (
      <div className="space-y-6">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {bars.map((b) => {
            const p = pct(b.value, b.min, b.max);
            return (
              <Card key={b.label}>
                <div className="flex items-center justify-between mb-2">
                  <p className="text-sm text-text-muted">{b.label}</p>
                  <Badge variant={b.value > -9 ? "warning" : b.value > -14 ? "default" : "default"}>
                    {b.badge}
                  </Badge>
                </div>
                <p className={`text-3xl font-bold ${b.colorClass}`}>
                  {formatLoudness(b.value)}
                </p>
                <p className="text-xs text-text-muted mt-1">LUFS</p>
                <div className="mt-3 h-2 w-full bg-bg-tertiary rounded-full overflow-hidden">
                  <div
                    className={`h-full rounded-full transition-all duration-500 ${
                      p > 70 ? "bg-green-500" : p > 50 ? "bg-yellow-500" : p > 30 ? "bg-orange-500" : "bg-red-500"
                    }`}
                    style={{ width: `${Math.max(2, p)}%` }}
                  />
                </div>
              </Card>
            );
          })}
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <Card>
            <CardTitle>Peak Level</CardTitle>
            <p className={`text-3xl font-bold mt-2 ${peakColor(l.peak_db)}`}>
              {formatLoudness(l.peak_db)} dB
            </p>
            <div className="mt-3 h-2 w-full bg-bg-tertiary rounded-full overflow-hidden">
              <div
                className={`h-full rounded-full ${
                  l.peak_db > -1 ? "bg-red-500" : l.peak_db > -6 ? "bg-yellow-500" : "bg-green-500"
                }`}
                style={{ width: `${Math.max(2, Math.min(100, ((l.peak_db + 30) / 30) * 100))}%` }}
              />
            </div>
          </Card>
          <Card>
            <CardTitle>RMS Level</CardTitle>
            <p className="text-3xl font-bold text-text-primary mt-2">
              {formatLoudness(l.rms_db)} dB
            </p>
            <div className="mt-3 h-2 w-full bg-bg-tertiary rounded-full overflow-hidden">
              <div
                className="h-full rounded-full bg-blue-500"
                style={{ width: `${Math.max(2, Math.min(100, ((l.rms_db + 30) / 30) * 100))}%` }}
              />
            </div>
          </Card>
        </div>
      </div>
    );
  }

  return (
    <div className="page-container">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-3">
          {selectedProjectId && onClearSelected && (
            <button
              onClick={onClearSelected}
              className="text-text-muted hover:text-text-primary transition-colors"
              title="Back"
            >
              <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
              </svg>
            </button>
          )}
          <h1 className="page-header mb-0">
            Analysis{selectedProjectId ? " — Project" : ""}
          </h1>
        </div>
        <Button onClick={handleFileSelect} disabled={loading}>
          {loading ? "Analyzing…" : "Analyze Audio"}
        </Button>
        <input
          ref={fileInputRef}
          type="file"
          accept=".wav,.mp3,.flac,.ogg,.aiff"
          className="hidden"
          onChange={handleNativeFileChange}
        />
      </div>

      {error && (
        <Card className="mb-6 border border-red-500/30 bg-red-500/5">
          <p className="text-red-400 text-sm">{error}</p>
        </Card>
      )}

      {loading && (
        <Card className="mb-6">
          <div className="flex items-center gap-3">
            <div className="w-5 h-5 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
            <p className="text-text-muted">Analyzing audio file…</p>
          </div>
        </Card>
      )}

      {result && !loading && (
        <div className="space-y-6">
          <div>
            <h2 className="text-sm font-semibold text-text-muted uppercase tracking-wider mb-3">
              Audio Information
            </h2>
            {renderAudioInfo(result.audio_info)}
          </div>

          {result.loudness && (
            <div>
              <h2 className="text-sm font-semibold text-text-muted uppercase tracking-wider mb-3">
                Loudness Analysis
              </h2>
              {renderLoudness(result.loudness)}
            </div>
          )}

          {!result.loudness && result.error && (
            <Card className="border border-red-500/30 bg-red-500/5">
              <p className="text-red-400 text-sm">{result.error}</p>
            </Card>
          )}
        </div>
      )}

      {!result && !loading && !error && (
        <div className="text-center py-20">
          <p className="text-lg text-text-muted mb-2">No analysis yet</p>
          <p className="text-sm text-text-muted">
            Select an audio file to analyze loudness, peak, and RMS levels.
          </p>
        </div>
      )}
    </div>
  );
}
