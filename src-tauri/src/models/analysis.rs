use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalyzeAudioRequest {
    pub project_id: String,
    pub file_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioInfo {
    pub duration_secs: f64,
    pub sample_rate: u32,
    pub bit_depth: u16,
    pub channels: u16,
    pub file_size: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoudnessResult {
    pub integrated_lufs: f64,
    pub short_term_lufs: f64,
    pub momentary_lufs: f64,
    pub peak_db: f64,
    pub rms_db: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalysisResult {
    pub id: String,
    pub project_id: String,
    pub file_path: String,
    pub file_hash: String,
    pub audio_info: AudioInfo,
    pub loudness: Option<LoudnessResult>,
    pub analyzed_at: String,
    pub error: Option<String>,
}
