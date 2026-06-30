# Project: ScribeRx

## Architecture
ScribeRx is a production-grade clinical automation platform composed of Tauri frontend (HTML/CSS/TS) and a multi-crate Rust workspace:
- `crates/app-shell`: Orchestrates IPC commands, dictation session states, and EMR adaptors.
- `crates/core-audio`: Captures audio using `cpal` and detects voice activity (VAD).
- `crates/stt-engine`: Runs the local Whisper transcription daemon with medical token biasing.
- `crates/drug-match`: Performs phonetic and Levenshtein matching against CDSCO/NLEM databases, validates dosages with strict immutability, and executes drug safety checks (interactions, allergies).
- `crates/core-hotkey`: Installs global hotkeys, tracks carets, and injects text via UI Automation, SendInput, or Clipboard.
- `crates/storage`: Encrypts clinical audit logs and dictionaries at rest via SQLite + SQLCipher using DPAPI-derived keys.
- `crates/core-wakeword`: Lightweight on-device wake-word detector ("Hey ScribeRx") and state machine transitions.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Security & Storage | SQLCipher, DPAPI key derivation, Windows Hello integration | None | PLANNED |
| M2 | EMR Adapter & Validation | Browser and Win32/WPF adapters, caret/window context checks, allowlisted commands | M1 | PLANNED |
| M3 | Terminology & Safety | CDSCO SQLite DB integration, layered vocabularies, drug-drug safety checker, dosage immutability | M2 | PLANNED |
| M4 | Ambient Mode & FHIR | Speaker separation, SOAP clinical draft generator, transcript tracing, FHIR R4 compliance | M3 | PLANNED |
| M5 | UI & E2E Verification | Consent UI, settings UI, full integration, 100% E2E test pass | M4 | PLANNED |
| M6 | Wake-Word Detection | core-wakeword crate, state transitions, sliding window privacy, mute control | M2 | PLANNED |

## Interface Contracts
### 1. Storage (`crates/storage`)
```rust
pub trait EncryptedStorage: Send + Sync {
    fn initialize_cipher(&mut self, user_pin: Option<&str>) -> Result<(), String>; // DPAPI/Hello key derivation
    fn log_audit_event(&self, audit_event_json: &str) -> Result<(), String>; // FHIR AuditEvent log
    fn save_doctor_dict(&self, dict: &[String]) -> Result<(), String>;
    fn load_doctor_dict(&self) -> Result<Vec<String>, String>;
}
```

### 2. EMR Adapter & Injector (`crates/core-hotkey`)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmrContext {
    pub patient_name: String,
    pub encounter_id: String,
    pub window_title: String,
    pub hwnd: usize,
    pub caret_bounds: Option<(i32, i32, i32, i32)>,
}

pub trait EmrAdapter: Send + Sync {
    fn capture_context(&self) -> Result<EmrContext, String>;
    fn validate_context(&self, expected: &EmrContext) -> Result<bool, String>;
    fn inject_text(&self, text: &str, context: &EmrContext) -> Result<(), String>;
}
```

### 3. Terminology & Drug Safety (`crates/drug-match`)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyWarning {
    pub warning_level: String, // "High", "Medium", "Low"
    pub description: String,
}

pub trait ClinicalSafetyEngine: Send + Sync {
    fn check_safety(&self, drug_name: &str, dosage: &str, patient_context_json: &str) -> Vec<SafetyWarning>;
    fn match_terminology(&self, raw_text: &str, specialty: &str) -> MedicalCorrectionResult;
}
```

### 4. Wake-Word Engine (`crates/core-wakeword`)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DictationState {
    Armed,
    WakeDetected,
    Listening,
    Processing,
    Review,
    Injecting,
}

pub trait WakeWordListener: Send + Sync {
    fn start_listening(&mut self) -> Result<(), String>;
    fn stop_listening(&mut self) -> Result<(), String>;
    fn check_wake_word(&self, pcm_chunk: &[f32]) -> Result<bool, String>;
    fn set_mute(&mut self, muted: bool);
    fn get_state(&self) -> DictationState;
}
```

## Code Layout
The workspace layout matches Section 1 of `docs/ARCHITECTURE.md`.
Source files to modify/add:
- `crates/storage/src/lib.rs` (implement SQLCipher + DPAPI / Hello)
- `crates/core-hotkey/src/lib.rs` (implement EMR adapters, caret position, and verification)
- `crates/drug-match/src/lib.rs` (implement CDSCO DB loading, safety checker, and layered vocabularies)
- `crates/core-wakeword/src/lib.rs` (implement "Hey ScribeRx" detector, audio slider window, mute control)
- `crates/app-shell/src/main.rs` (integrate speaker separation, session state machine, FHIR JSON generation)
- `ui/` (modify main.js, index.html, style.css to support consent and alternatives display)
