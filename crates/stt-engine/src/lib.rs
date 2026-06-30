use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub raw_text: String,
    pub tokens: Vec<TokenConfidence>,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfidence {
    pub token: String,
    pub confidence: f32,
    pub start_time_ms: u64,
    pub end_time_ms: u64,
}

#[async_trait]
pub trait SttEngine: Send + Sync {
    async fn load_model(&mut self, model_path: &str) -> Result<(), String>;
    async fn transcribe(&self, audio_pcm: &[f32], sample_rate: u32) -> Result<TranscriptionResult, String>;
}

// ── Windows SAPI engine (persistent faster-whisper daemon) ─────────────────────
#[cfg(target_os = "windows")]
use std::io::{BufRead, BufReader};
#[cfg(target_os = "windows")]
use std::process::{ChildStdin, ChildStdout, Command, Stdio};

#[cfg(target_os = "windows")]
pub struct WindowsSapiEngine {
    daemon: Mutex<Option<(ChildStdin, BufReader<ChildStdout>)>>,
}

#[cfg(target_os = "windows")]
impl WindowsSapiEngine {
    pub fn new() -> Self {
        Self {
            daemon: Mutex::new(None),
        }
    }
}

#[cfg(target_os = "windows")]
#[async_trait]
impl SttEngine for WindowsSapiEngine {
    async fn load_model(&mut self, _model_path: &str) -> Result<(), String> {
        println!("Spawning persistent Whisper transcription daemon...");
        
        let daemon_script = if std::path::Path::new("transcribe.py").exists() {
            "transcribe.py".to_string()
        } else {
            "C:\\Users\\Gokulprasath K\\.gemini\\antigravity\\brain\\a9f09a62-185f-4d47-af7e-821d8e5206b3\\scratch\\transcribe.py".to_string()
        };
        
        let mut child = Command::new("python")
            .env("PYTHONUNBUFFERED", "1")
            .args([
                &daemon_script,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start transcription daemon: {}", e))?;

        let stdin = child.stdin.take().ok_or("Failed to get daemon stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to get daemon stdout")?;
        let mut reader = BufReader::new(stdout);

        let mut line = String::new();
        reader.read_line(&mut line).map_err(|e| format!("Failed to read READY from daemon: {}", e))?;
        if line.trim() != "READY" {
            return Err(format!("Daemon initialization error, got: {}", line));
        }

        *self.daemon.lock().unwrap() = Some((stdin, reader));
        println!("Whisper transcription daemon initialized and READY.");
        Ok(())
    }

    async fn transcribe(&self, audio_pcm: &[f32], sample_rate: u32) -> Result<TranscriptionResult, String> {
        let start = std::time::Instant::now();

        let mut normalized_pcm = audio_pcm.to_vec();
        let mut max_amp = 0.0f32;
        for &s in normalized_pcm.iter() {
            let abs = s.abs();
            if abs > max_amp {
                max_amp = abs;
            }
        }
        if max_amp > 0.0001 {
            let scale = 0.8 / max_amp;
            for s in normalized_pcm.iter_mut() {
                *s *= scale;
            }
        }

        let wav_path = std::env::temp_dir().join("scriberx_input.wav");
        write_wav(&wav_path, &normalized_pcm, sample_rate)
            .map_err(|e| format!("Failed to write WAV: {}", e))?;

        let wav_path_str = wav_path.to_string_lossy();

        let mut guard = self.daemon.lock().unwrap();
        let (stdin, reader) = guard.as_mut().ok_or("Daemon not initialized")?;

        writeln!(stdin, "{}", wav_path_str).map_err(|e| format!("Failed to write to daemon: {}", e))?;
        stdin.flush().map_err(|e| format!("Failed to flush daemon: {}", e))?;

        let mut raw_text = String::new();
        reader.read_line(&mut raw_text).map_err(|e| format!("Failed to read from daemon: {}", e))?;
        let raw_text = raw_text.trim().to_string();

        let elapsed_ms = start.elapsed().as_millis() as u64;

        Ok(TranscriptionResult {
            raw_text,
            tokens: vec![],
            processing_time_ms: elapsed_ms,
        })
    }
}

// ── Fallback SAPI engine for macOS/Linux ──────────────────────────────────────
#[cfg(not(target_os = "windows"))]
pub struct WindowsSapiEngine {
    mock_transcription: Mutex<String>,
}

#[cfg(not(target_os = "windows"))]
impl WindowsSapiEngine {
    pub fn new() -> Self {
        Self {
            mock_transcription: Mutex::new("test dictation".to_string()),
        }
    }

    pub fn set_mock_transcription(&self, text: &str) {
        if let Ok(mut guard) = self.mock_transcription.lock() {
            *guard = text.to_string();
        }
    }
}

#[cfg(not(target_os = "windows"))]
#[async_trait]
impl SttEngine for WindowsSapiEngine {
    async fn load_model(&mut self, _model_path: &str) -> Result<(), String> {
        println!("Mocking Whisper transcription daemon on macOS/Linux...");
        Ok(())
    }

    async fn transcribe(&self, _audio_pcm: &[f32], _sample_rate: u32) -> Result<TranscriptionResult, String> {
        let text = if let Ok(guard) = self.mock_transcription.lock() {
            guard.clone()
        } else {
            "test dictation".to_string()
        };
        Ok(TranscriptionResult {
            raw_text: text,
            tokens: vec![],
            processing_time_ms: 5,
        })
    }
}

// ── Dummy (kept for tests) ─────────────────────────────────────────────────────
pub struct DummySttEngine;

#[async_trait]
impl SttEngine for DummySttEngine {
    async fn load_model(&mut self, _model_path: &str) -> Result<(), String> {
        Ok(())
    }
    async fn transcribe(&self, _audio_pcm: &[f32], _sample_rate: u32) -> Result<TranscriptionResult, String> {
        Ok(TranscriptionResult {
            raw_text: "test dictation".to_string(),
            tokens: vec![],
            processing_time_ms: 0,
        })
    }
}

// ── WAV writer ─────────────────────────────────────────────────────────────────
fn write_wav(path: &std::path::Path, samples: &[f32], sample_rate: u32) -> std::io::Result<()> {
    let num_channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * u32::from(num_channels) * u32::from(bits_per_sample) / 8;
    let block_align = num_channels * bits_per_sample / 8;
    let pcm_bytes: Vec<u8> = samples
        .iter()
        .flat_map(|&s| {
            let clamped = s.clamp(-1.0, 1.0);
            let i16_val = (clamped * i16::MAX as f32) as i16;
            i16_val.to_le_bytes()
        })
        .collect();
    let data_len = pcm_bytes.len() as u32;
    let mut f = std::fs::File::create(path)?;
    // RIFF header
    f.write_all(b"RIFF")?;
    f.write_all(&(36 + data_len).to_le_bytes())?;
    f.write_all(b"WAVE")?;
    // fmt chunk
    f.write_all(b"fmt ")?;
    f.write_all(&16u32.to_le_bytes())?;     // chunk size
    f.write_all(&1u16.to_le_bytes())?;      // PCM
    f.write_all(&num_channels.to_le_bytes())?;
    f.write_all(&sample_rate.to_le_bytes())?;
    f.write_all(&byte_rate.to_le_bytes())?;
    f.write_all(&block_align.to_le_bytes())?;
    f.write_all(&bits_per_sample.to_le_bytes())?;
    // data chunk
    f.write_all(b"data")?;
    f.write_all(&data_len.to_le_bytes())?;
    f.write_all(&pcm_bytes)?;
    Ok(())
}
