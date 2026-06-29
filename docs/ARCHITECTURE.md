# ScribeRx Architecture & Module Contracts

This document defines the interface contracts, data models, and crate boundaries for ScribeRx. Subagents and developers must adhere to these trait signatures and data structures to maintain loose coupling across the workspace.

---

## 1. Workspace Layout & Crate Responsibilities

```
/voice-to-text
 ├── Cargo.toml
 ├── crates/
 │    ├── core-hotkey/        # Hotkey detection, caretaker caret positioning, text injection (windows-rs / UI Automation)
 │    ├── core-audio/         # CPAL audio capture stream, WASAPI bindings, silero/energy VAD
 │    ├── stt-engine/         # whisper.cpp C bindings, model loading, quantization (q5_1 GGUF)
 │    ├── drug-match/         # Levenshtein/phonetic drug name matcher, confidence scoring
 │    ├── drug-db-builder/    # Offline utility: ingests CDSCO/NLEM CSV into SQLite DB
 │    ├── storage/            # Rusqlite + SQLCipher encrypted storage for audit logs & user settings
 │    └── app-shell/          # Tauri application entry point, IPC state manager, commands orchestration
 ├── ui/                      # Vanilla HTML/CSS/TS frontend for Tauri webview
 └── docs/                    # Architecture, PRD, Status, Design System specs
```

---

## 2. Core Module Trait Contracts

### 2.1 STT Engine (`crates/stt-engine`)

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub raw_text: String,
    pub tokens: Vec<TokenConfidence>,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfidence {
    pub token: String,
    pub confidence: f32, // 0.0 to 1.0
    pub start_time_ms: u64,
    pub end_time_ms: u64,
}

#[async_trait]
pub trait SttEngine: Send + Sync {
    async fn load_model(&mut self, model_path: &str) -> Result<(), String>;
    async fn transcribe(&self, audio_pcm: &[f32], sample_rate: u32) -> Result<TranscriptionResult, String>;
}
```

### 2.2 Drug Matching & Scorer (`crates/drug-match`)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    High,   // >= 0.85
    Medium, // 0.65 to 0.84 (underlined in UI)
    Low,    // < 0.65 (yellow chip with alternatives in UI)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedDrugToken {
    pub original_word: String,
    pub matched_name: Option<String>,
    pub dosage: Option<String>, // e.g., "650 mg" (NEVER auto-corrected)
    pub confidence: f32,
    pub confidence_level: ConfidenceLevel,
    pub alternatives: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalCorrectionResult {
    pub formatted_text: String,
    pub terms: Vec<MatchedDrugToken>,
    pub has_low_confidence: bool,
}

pub trait DrugMatcher: Send + Sync {
    fn match_text(&self, raw_transcript: &str) -> MedicalCorrectionResult;
    fn add_doctor_vocabulary(&mut self, drug_names: &[String]);
}
```

### 2.3 Audio Capture & VAD (`crates/core-audio`)

```rust
pub trait AudioRecorder: Send + Sync {
    fn start_recording(&mut self) -> Result<(), String>;
    fn stop_recording(&mut self) -> Result<Vec<f32>, String>; // Returns PCM float samples
    fn get_current_rms_level(&self) -> f32; // For live waveform UI feedback
}
```

### 2.4 Text Injector (`crates/core-hotkey`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectionStrategy {
    SendInput,
    ClipboardPaste,
    UiAutomation,
}

pub trait TextInjector: Send + Sync {
    fn inject(&self, text: &str, strategy: InjectionStrategy) -> Result<(), String>;
    fn capture_caret_position(&self) -> Option<(i32, i32)>;
}
```

### 2.5 Encrypted Storage (`crates/storage`)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub timestamp: u64,
    pub emr_app_name: String,
    pub injected_text: String,
    pub had_low_confidence: bool,
}

pub trait StorageEngine: Send + Sync {
    fn log_audit_entry(&self, entry: &AuditLogEntry) -> Result<(), String>;
    fn get_recent_audit_logs(&self, limit: usize) -> Result<Vec<AuditLogEntry>, String>;
}
```

---

## 3. Tauri IPC Command Interface

The frontend (`ui/`) communicates with `crates/app-shell` via Tauri commands:

| Command | Payload | Return | Description |
|---|---|---|---|
| `start_dictation` | None | `Result<(), String>` | Triggered by hotkey or UI button; starts audio stream |
| `stop_dictation` | None | `Result<MedicalCorrectionResult, String>` | Stops audio, runs STT + Drug Matcher, returns result for review |
| `confirm_and_inject` | `{ text: String, strategy: String }` | `Result<(), String>` | Injects resolved text into EMR and logs audit entry |
| `get_audio_level` | None | `f32` | Polled or streamed via Tauri event for live waveform |

---

## 4. Safety Principles

1. **Numeric Dosages are Immutable**: The `DrugMatcher` must never perform fuzzy edits on numeric values (e.g., "500 mg" vs "50 mg"). Numeric discrepancies must lower the confidence score and trigger manual doctor confirmation.
2. **Clipboard Preservation**: If `ClipboardPaste` strategy is used, prior clipboard content must be saved and restored within 100ms post-injection.
