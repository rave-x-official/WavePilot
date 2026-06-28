import { Card, CardTitle } from "../components/ui/Card";

export function Analysis() {
  return (
    <div className="page-container">
      <h1 className="page-header">Analysis</h1>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
        <Card>
          <CardTitle>BPM Detection</CardTitle>
          <p className="text-sm text-text-muted mt-2">
            Detect tempo from audio files. Coming soon.
          </p>
        </Card>
        <Card>
          <CardTitle>Key Detection</CardTitle>
          <p className="text-sm text-text-muted mt-2">
            Detect musical key from audio files. Coming soon.
          </p>
        </Card>
        <Card>
          <CardTitle>Loudness Analyzer</CardTitle>
          <p className="text-sm text-text-muted mt-2">
            Measure LUFS and peak levels. Coming soon.
          </p>
        </Card>
      </div>

      <Card>
        <CardTitle>Analysis History</CardTitle>
        <p className="text-sm text-text-muted mt-2">
          No analyses yet. Import a project and run analysis to get started.
        </p>
      </Card>
    </div>
  );
}
