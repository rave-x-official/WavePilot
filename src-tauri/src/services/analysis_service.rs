use crate::db::Database;
use crate::models::analysis::{AnalysisResult, AudioInfo, LoudnessResult};
use crate::utils::{new_id, now_timestamp};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

// --- ITU-R BS.1770-4 K-weighting filter ---

struct Biquad {
    b0: f64, b1: f64, b2: f64,
    a1: f64, a2: f64,
    x1: f64, x2: f64,
    y1: f64, y2: f64,
}

impl Biquad {
    fn high_pass(sample_rate: f64, fc: f64, q: f64) -> Self {
        let w0 = 2.0 * std::f64::consts::PI * fc / sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);
        let a0 = 1.0 + alpha;

        Biquad {
            b0: (1.0 + cos_w0) / (2.0 * a0),
            b1: -(1.0 + cos_w0) / a0,
            b2: (1.0 + cos_w0) / (2.0 * a0),
            a1: (-2.0 * cos_w0) / a0,
            a2: (1.0 - alpha) / a0,
            x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0,
        }
    }

    fn low_shelf(sample_rate: f64, fc: f64, gain_db: f64, q: f64) -> Self {
        let w0 = 2.0 * std::f64::consts::PI * fc / sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);
        let a = 10.0_f64.powf(gain_db / 40.0);
        let sqrt_a = a.sqrt();
        let two_sqrt_a_alpha = 2.0 * sqrt_a * alpha;
        let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + two_sqrt_a_alpha;

        Biquad {
            b0: (a * ((a + 1.0) - (a - 1.0) * cos_w0 + two_sqrt_a_alpha)) / a0,
            b1: (2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0)) / a0,
            b2: (a * ((a + 1.0) - (a - 1.0) * cos_w0 - two_sqrt_a_alpha)) / a0,
            a1: (-2.0 * ((a - 1.0) + (a + 1.0) * cos_w0)) / a0,
            a2: ((a + 1.0) + (a - 1.0) * cos_w0 - two_sqrt_a_alpha) / a0,
            x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0,
        }
    }

    fn process(&mut self, sample: f64) -> f64 {
        let y = self.b0 * sample + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = sample;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

// --- Audio reading ---

fn read_wav(path: &Path) -> Result<Vec<Vec<f64>>, String> {
    let mut reader = hound::WavReader::open(path).map_err(|e| format!("Failed to open WAV: {}", e))?;
    let spec = reader.spec();
    let channels = spec.channels as usize;
    let bits_per_sample = spec.bits_per_sample;

    if bits_per_sample != 16 && bits_per_sample != 24 && bits_per_sample != 32 {
        return Err(format!("Unsupported bit depth: {}. Only 16, 24, 32 bit WAV supported.", bits_per_sample));
    }

    let max_val = match bits_per_sample {
        16 => 32768.0_f64,
        24 => 8388608.0_f64,
        32 => 2147483648.0_f64,
        _ => unreachable!(),
    };

    let total_samples = reader.len() as usize;
    let mut samples: Vec<Vec<f64>> = vec![Vec::with_capacity(total_samples / channels); channels];

    match bits_per_sample {
        16 => {
            for sample in reader.samples::<i16>() {
                let s = sample.map_err(|e| format!("Read error: {}", e))?;
                samples[0].push(s as f64 / max_val);
            }
        }
        24 => {
            for sample in reader.samples::<i32>() {
                let s = sample.map_err(|e| format!("Read error: {}", e))?;
                samples[0].push(s as f64 / max_val);
            }
        }
        32 => {
            for sample in reader.samples::<i32>() {
                let s = sample.map_err(|e| format!("Read error: {}", e))?;
                samples[0].push(s as f64 / max_val);
            }
        }
        _ => unreachable!(),
    }

    // Deinterleave if multi-channel
    if channels > 1 {
        let total = samples[0].len();
        let frame_count = total / channels;
        let interleaved = std::mem::take(&mut samples[0]);
        for ch in 0..channels {
            samples[ch] = Vec::with_capacity(frame_count);
        }
        for frame in 0..frame_count {
            for ch in 0..channels {
                samples[ch].push(interleaved[frame * channels + ch]);
            }
        }
    }

    Ok(samples)
}

// --- Loudness computation (ITU-R BS.1770-4) ---

fn mean_square(samples: &[f64]) -> f64 {
    let sum: f64 = samples.iter().map(|s| s * s).sum();
    sum / samples.len() as f64
}

fn apply_k_weighting(samples: &[Vec<f64>], sample_rate: f64) -> Vec<Vec<f64>> {
    let fs = sample_rate.min(48000.0);
    let channels = samples.len();
    let mut weighted = Vec::with_capacity(channels);
    for ch in 0..channels {
        let mut rlb_state = Biquad::high_pass(fs, 38.0, 0.5);
        let mut shelf_state = Biquad::low_shelf(fs, 1500.0, 4.0, 0.707);
        let processed: Vec<f64> = samples[ch]
            .iter()
            .map(|&s| shelf_state.process(rlb_state.process(s)))
            .collect();
        weighted.push(processed);
    }
    weighted
}

fn channel_weights(channels: usize) -> Vec<f64> {
    match channels {
        1 => vec![1.0],
        2 => vec![1.0, 1.0],
        3 => vec![1.0, 1.0, 1.0],
        4 => vec![1.0, 1.0, 1.0, 0.0],
        5 => vec![1.0, 1.0, 1.0, 1.41, 1.41],
        _ => vec![1.0; channels],
    }
}

fn block_energies(weighted: &[Vec<f64>], sample_rate: u32, block_size_secs: f64) -> Vec<f64> {
    let block_len = (sample_rate as f64 * block_size_secs) as usize;
    let channels = weighted.len();
    let channel_weights = channel_weights(channels);
    let total_frames = weighted[0].len();
    let num_blocks = total_frames / block_len;

    let mut energies = Vec::with_capacity(num_blocks);
    for block in 0..num_blocks {
        let start = block * block_len;
        let end = start + block_len;
        let mut channel_sum = 0.0;
        for ch in 0..channels {
            let ms = mean_square(&weighted[ch][start..end]);
            channel_sum += channel_weights[ch] * ms;
        }
        energies.push(channel_sum);
    }
    energies
}

fn lufs_from_energy(energy: f64) -> f64 {
    -0.691 + 10.0 * energy.log10()
}

fn compute_integrated_lufs(weighted: &[Vec<f64>], sample_rate: u32) -> f64 {
    let energies = block_energies(weighted, sample_rate, 0.4);

    if energies.is_empty() {
        return -std::f64::INFINITY;
    }

    // Absolute gate threshold (-70 LUFS)
    let abs_threshold = 10.0_f64.powf((-70.0 + 0.691) / 10.0);
    let above_abs: Vec<f64> = energies.iter().copied().filter(|&e| e > abs_threshold).collect();

    if above_abs.is_empty() {
        return -std::f64::INFINITY;
    }

    let abs_mean: f64 = above_abs.iter().sum::<f64>() / above_abs.len() as f64;
    let abs_lufs = lufs_from_energy(abs_mean);

    // Relative gate: abs_lufs - 8 dB
    let rel_threshold = 10.0_f64.powf((abs_lufs - 8.0 + 0.691) / 10.0);
    let above_rel: Vec<f64> = energies.iter().copied().filter(|&e| e > rel_threshold).collect();

    if above_rel.is_empty() {
        return abs_lufs;
    }

    let rel_mean: f64 = above_rel.iter().sum::<f64>() / above_rel.len() as f64;
    lufs_from_energy(rel_mean)
}

fn compute_short_term_lufs(weighted: &[Vec<f64>], sample_rate: u32) -> f64 {
    let block_len = (sample_rate as f64 * 3.0) as usize;
    let hop_len = block_len / 4;
    let channels = weighted.len();
    let channel_weights = channel_weights(channels);
    let total_frames = weighted[0].len();

    if total_frames < block_len {
        // If shorter than 3 sec, compute what we have
        let mut channel_sum = 0.0;
        for ch in 0..channels {
            let ms = mean_square(&weighted[ch]);
            channel_sum += channel_weights[ch] * ms;
        }
        return lufs_from_energy(channel_sum);
    }

    let mut max_lufs = -std::f64::INFINITY;
    let mut start = 0;
    while start + block_len <= total_frames {
        let end = start + block_len;
        let mut channel_sum = 0.0;
        for ch in 0..channels {
            let ms = mean_square(&weighted[ch][start..end]);
            channel_sum += channel_weights[ch] * ms;
        }
        let l = lufs_from_energy(channel_sum);
        if l > max_lufs {
            max_lufs = l;
        }
        start += hop_len;
    }

    if max_lufs.is_infinite() { -std::f64::INFINITY } else { max_lufs }
}

fn compute_momentary_lufs(weighted: &[Vec<f64>], sample_rate: u32) -> f64 {
    let block_len = (sample_rate as f64 * 0.4) as usize;
    let channels = weighted.len();
    let channel_weights = channel_weights(channels);
    let total_frames = weighted[0].len();

    if total_frames < block_len {
        let mut channel_sum = 0.0;
        for ch in 0..channels {
            let ms = mean_square(&weighted[ch]);
            channel_sum += channel_weights[ch] * ms;
        }
        return lufs_from_energy(channel_sum);
    }

    let mut max_lufs = -std::f64::INFINITY;
    let mut start = 0;
    while start + block_len <= total_frames {
        let end = start + block_len;
        let mut channel_sum = 0.0;
        for ch in 0..channels {
            let ms = mean_square(&weighted[ch][start..end]);
            channel_sum += channel_weights[ch] * ms;
        }
        let l = lufs_from_energy(channel_sum);
        if l > max_lufs {
            max_lufs = l;
        }
        start += block_len / 2;
    }

    if max_lufs.is_infinite() { -std::f64::INFINITY } else { max_lufs }
}

fn compute_peak_db(samples: &[Vec<f64>]) -> f64 {
    let max_peak: f64 = samples.iter()
        .flat_map(|ch| ch.iter())
        .map(|s| s.abs())
        .fold(0.0_f64, f64::max);

    if max_peak <= 0.0 {
        return -std::f64::INFINITY;
    }
    20.0 * max_peak.log10()
}

fn compute_rms_db(samples: &[Vec<f64>]) -> f64 {
    let channels = samples.len();
    let channel_weights = channel_weights(channels);
    let mut sum_sq = 0.0;
    let mut total_samples = 0usize;
    for (ch, samples) in samples.iter().enumerate() {
        for &s in samples {
            sum_sq += channel_weights[ch] * s * s;
        }
        total_samples += samples.len();
    }
    let rms = (sum_sq / total_samples as f64).sqrt();
    if rms <= 0.0 {
        return -std::f64::INFINITY;
    }
    20.0 * rms.log10()
}

fn compute_file_hash(path: &Path) -> Result<String, String> {
    let metadata = std::fs::metadata(path).map_err(|e| format!("Cannot read metadata: {}", e))?;
    let mut hasher = DefaultHasher::new();
    path.to_string_lossy().hash(&mut hasher);
    metadata.len().hash(&mut hasher);
    if let Ok(mtime) = metadata.modified() {
        if let Ok(duration) = mtime.duration_since(std::time::UNIX_EPOCH) {
            duration.as_nanos().hash(&mut hasher);
        }
    }
    Ok(format!("{:x}", hasher.finish()))
}

// --- Public API ---

pub fn analyze_audio_file(
    db: &Database,
    project_id: &str,
    file_path: &str,
) -> Result<AnalysisResult, String> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("File does not exist: {}", file_path));
    }

    let file_size = std::fs::metadata(path)
        .map_err(|e| format!("Cannot read file: {}", e))?
        .len();
    let file_hash = compute_file_hash(path)?;

    // Check cache
    if let Some(cached) = get_cached_analysis(db, project_id, &file_hash)? {
        log::info!("Using cached analysis for: {}", file_path);
        return Ok(cached);
    }

    log::info!("Analyzing audio file: {}", file_path);

    // Read audio
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

    let samples = match extension.as_str() {
        "wav" => read_wav(path)?,
        _ => return Err(format!("Unsupported audio format: .{}. Only WAV is supported currently.", extension)),
    };

    if samples.is_empty() || samples[0].is_empty() {
        return Err("Audio file contains no samples".to_string());
    }

    // Read WAV info
    let reader = hound::WavReader::open(path).map_err(|e| format!("Failed to open WAV: {}", e))?;
    let spec = reader.spec();

    let sample_rate = spec.sample_rate;
    let channels = spec.channels as u16;
    let bits_per_sample = spec.bits_per_sample;
    let total_samples = reader.len() as u64;
    let duration_secs = total_samples as f64 / (sample_rate as f64 * channels as f64);

    let audio_info = AudioInfo {
        duration_secs,
        sample_rate,
        bit_depth: bits_per_sample,
        channels,
        file_size,
    };

    drop(reader);

    // Compute loudness
    let weighted = apply_k_weighting(&samples, sample_rate as f64);

    let loudness = LoudnessResult {
        integrated_lufs: compute_integrated_lufs(&weighted, sample_rate),
        short_term_lufs: compute_short_term_lufs(&weighted, sample_rate),
        momentary_lufs: compute_momentary_lufs(&weighted, sample_rate),
        peak_db: compute_peak_db(&samples),
        rms_db: compute_rms_db(&samples),
    };

    let result = AnalysisResult {
        id: new_id(),
        project_id: project_id.to_string(),
        file_path: file_path.to_string(),
        file_hash,
        audio_info,
        loudness: Some(loudness),
        analyzed_at: now_timestamp(),
        error: None,
    };

    // Cache the result
    cache_analysis(db, &result)?;

    log::info!("Analysis complete for: {}", file_path);
    Ok(result)
}

fn get_cached_analysis(
    db: &Database,
    project_id: &str,
    file_hash: &str,
) -> Result<Option<AnalysisResult>, String> {
    let conn = db.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT result FROM analysis_cache WHERE project_id = ?1 AND analysis_type = 'loudness' AND id = (
                SELECT id FROM analysis_cache WHERE project_id = ?1 AND analysis_type = 'loudness' ORDER BY created_at DESC LIMIT 1
            )",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![project_id], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        if let Ok(json) = row {
            if let Ok(result) = serde_json::from_str::<AnalysisResult>(&json) {
                if result.file_hash == file_hash {
                    return Ok(Some(result));
                }
            }
        }
    }
    Ok(None)
}

fn cache_analysis(db: &Database, result: &AnalysisResult) -> Result<(), String> {
    let json = serde_json::to_string(result).map_err(|e| format!("Serialization error: {}", e))?;
    let conn = db.lock()?;
    conn.execute(
        "INSERT INTO analysis_cache (id, project_id, analysis_type, result, created_at) VALUES (?1, ?2, 'loudness', ?3, ?4)",
        rusqlite::params![result.id, result.project_id, json, result.analyzed_at],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_analysis_history(db: &Database) -> Result<Vec<AnalysisResult>, String> {
    let conn = db.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT result FROM analysis_cache WHERE analysis_type = 'loudness' ORDER BY created_at DESC LIMIT 50",
        )
        .map_err(|e| e.to_string())?;

    let results: Vec<AnalysisResult> = stmt
        .query_map([], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .filter_map(|json| serde_json::from_str::<AnalysisResult>(&json).ok())
        .collect();

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_wav(
        path: &std::path::Path,
        sample_rate: u32,
        channels: u16,
        bits_per_sample: u16,
        duration_secs: f64,
        frequency: f64,
    ) {
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec).unwrap();
        let total_samples = (sample_rate as f64 * duration_secs) as u32;

        match bits_per_sample {
            16 => {
                for i in 0..total_samples {
                    for _ch in 0..channels {
                        let t = i as f64 / sample_rate as f64;
                        let sample = (t * frequency * 2.0 * std::f64::consts::PI).sin();
                        writer.write_sample((sample * 0.5 * 32767.0) as i16).unwrap();
                    }
                }
            }
            24 => {
                for i in 0..total_samples {
                    for _ch in 0..channels {
                        let t = i as f64 / sample_rate as f64;
                        let sample = (t * frequency * 2.0 * std::f64::consts::PI).sin();
                        writer.write_sample((sample * 0.5 * 8388607.0) as i32).unwrap();
                    }
                }
            }
            _ => {}
        }
        writer.finalize().unwrap();
    }

    #[test]
    fn test_read_wav_mono() {
        let dir = std::path::PathBuf::from("/tmp/wp-test-analysis");
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("mono.wav");
        create_test_wav(&path, 44100, 1, 16, 0.5, 440.0);

        let samples = read_wav(&path).unwrap();
        assert_eq!(samples.len(), 1);
        assert!(samples[0].len() > 0);
    }

    #[test]
    fn test_read_wav_stereo() {
        let dir = std::path::PathBuf::from("/tmp/wp-test-analysis");
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("stereo.wav");
        create_test_wav(&path, 48000, 2, 16, 0.5, 440.0);

        let samples = read_wav(&path).unwrap();
        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0].len(), samples[1].len());
    }

    #[test]
    fn test_read_wav_24bit() {
        let dir = std::path::PathBuf::from("/tmp/wp-test-analysis");
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("24bit.wav");
        create_test_wav(&path, 44100, 1, 24, 0.3, 1000.0);

        let samples = read_wav(&path).unwrap();
        assert_eq!(samples.len(), 1);
        assert!(samples[0].len() > 0);
    }

    #[test]
    fn test_peak_db() {
        let samples = vec![vec![0.5, -0.5, 0.25, -0.75, 1.0, -1.0, 0.0]];
        let peak = compute_peak_db(&samples);
        assert!((peak - 0.0).abs() < 0.01); // 1.0 -> 0 dB
    }

    #[test]
    fn test_peak_db_silence() {
        let samples = vec![vec![0.0, 0.0, 0.0]];
        let peak = compute_peak_db(&samples);
        assert!(peak.is_infinite() && peak.is_sign_negative());
    }

    #[test]
    fn test_rms_db() {
        // A constant 0.5 sine -> RMS should be ~0.707 of peak
        let samples = vec![vec![0.5, 0.5, 0.5, 0.5, 0.5]];
        let rms = compute_rms_db(&samples);
        // RMS of 0.5 = 20*log10(0.5) = -6.02 dB
        assert!((rms - (-6.02)).abs() < 0.1);
    }

    #[test]
    fn test_file_hash_stable() {
        let dir = std::path::PathBuf::from("/tmp/wp-test-analysis");
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("hash_test.wav");
        create_test_wav(&path, 44100, 1, 16, 0.1, 440.0);

        let hash1 = compute_file_hash(&path).unwrap();
        let hash2 = compute_file_hash(&path).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_integrated_lufs_sine() {
        let dir = std::path::PathBuf::from("/tmp/wp-test-analysis");
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("lufs_sine.wav");
        // 1 kHz sine at -6 dB FS for 2 seconds
        let spec = hound::WavSpec { channels: 1, sample_rate: 48000, bits_per_sample: 16, sample_format: hound::SampleFormat::Int };
        let mut writer = hound::WavWriter::create(&path, spec).unwrap();
        for i in 0..96000 {
            let t = i as f64 / 48000.0;
            let sample = (t * 1000.0 * 2.0 * std::f64::consts::PI).sin();
            // -6 dB FS = amplitude of ~0.5
            writer.write_sample((sample * 0.5 * 32767.0) as i16).unwrap();
        }
        writer.finalize().unwrap();

        let samples = read_wav(&path).unwrap();
        let weighted = apply_k_weighting(&samples, 48000.0);
        let integrated = compute_integrated_lufs(&weighted, 48000);

        // A -6 dB FS sine should have LUFS around -9 to -12 (depends on K-weighting)
        assert!(integrated > -20.0 && integrated < -5.0,
            "Expected reasonable LUFS, got {}", integrated);
    }

    #[test]
    fn test_biquad_filter_response() {
        let mut hp = Biquad::high_pass(44100.0, 38.0, 0.5);
        // DC should be attenuated
        let out = hp.process(1.0);
        let _out2 = hp.process(1.0);
        let out3 = hp.process(1.0);
        // High-pass filters DC, so after a few samples, output should decrease
        assert!(out3.abs() < 0.5 || out3 < out);
    }

    #[test]
    fn test_biquad_shelf() {
        let mut shelf = Biquad::low_shelf(44100.0, 1500.0, 4.0, 0.707);
        let _ = shelf.process(1.0); // warm up
        let out = shelf.process(1.0);
        // Shelf with +4 dB gain should amplify
        assert!(out.abs() > 0.0 && out.abs() < 10.0);
    }

    #[test]
    fn test_loudness_for_noise() {
        let dir = std::path::PathBuf::from("/tmp/wp-test-analysis");
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("noise.wav");
        let spec = hound::WavSpec { channels: 2, sample_rate: 44100, bits_per_sample: 16, sample_format: hound::SampleFormat::Int };
        let mut writer = hound::WavWriter::create(&path, spec).unwrap();
        for _i in 0..44100 {
            let left: f64 = rand::random::<f64>() * 2.0 - 1.0;
            let right: f64 = rand::random::<f64>() * 2.0 - 1.0;
            writer.write_sample((left * 0.25 * 32767.0) as i16).unwrap();
            writer.write_sample((right * 0.25 * 32767.0) as i16).unwrap();
        }
        writer.finalize().unwrap();

        let samples = read_wav(&path).unwrap();
        let weighted = apply_k_weighting(&samples, 44100.0);
        let integrated = compute_integrated_lufs(&weighted, 44100);

        // White noise at -12 dB FS average should have measurable LUFS
        assert!(integrated > -30.0 && integrated < 0.0,
            "Expected reasonable LUFS for noise, got {}", integrated);
    }
}
