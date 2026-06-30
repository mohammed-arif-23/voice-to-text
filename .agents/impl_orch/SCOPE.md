# Scope: Implementation Track for ScribeRx

## Architecture
ScribeRx is composed of:
- `crates/core-audio`: Captures audio via CPAL, manages VAD.
- `crates/core-hotkey`: Handles Windows global hotkey and text injection.
- `crates/storage`: SQLCipher database, DPAPI key derivation, macOS mock.
- `crates/drug-match`: CDSCO matching, dosage safety, and layered dictionaries.
- `crates/stt-engine`: Whisper transcription bindings.
- `crates/app-shell`: Tauri frontend shell, IPC command coordinator.
- `crates/core-wakeword` (Planned M6): On-device "Hey ScribeRx" wake-word detection.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Security & Storage | SQLCipher integration, DPAPI key wrapping, macOS key store mock | None | IN_PROGRESS |
| M2 | EMR Adapter & Validation | Caret validation, Win32/WPF/browser EMR adapters, allowlisted navigation commands | M1 | PLANNED |
| M3 | Terminology & Safety | CDSCO SQLite loader, layered dictionary hierarchy, dosage immutability, safety checker | M2 | PLANNED |
| M4 | Ambient Mode & FHIR | SOAP clinical draft generator, transcript tracing, speaker separation, FHIR R4 JSON logs | M3 | PLANNED |
| M5 | UI & E2E Verification | Floating popup UI integration, settings view, 100% E2E test suite pass | M4 | PLANNED |
| M6 | Wake-Word Detection | Crate `core-wakeword` (Hey ScribeRx), CPAL integration, verbal commands, state machine, privacy guardrails | M2, M5 | PLANNED |

## Interface Contracts
### 1. Storage (`crates/storage`)
```rust
pub trait StorageEngine: Send + Sync {
    fn log_audit_entry(&self, entry: &AuditLogEntry) -> Result<(), String>;
    fn get_recent_audit_logs(&self, limit: usize) -> Result<Vec<AuditLogEntry>, String>;
}
```

### 2. Wake-Word Engine (`crates/core-wakeword`)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WakeWordState {
    Armed,
    WakeDetected,
    Listening,
    Processing,
    Review,
    Injecting,
}

pub trait WakeWordDetector: Send + Sync {
    fn feed_audio(&mut self, samples: &[f32]) -> Result<bool, String>;
    fn set_muted(&mut self, muted: bool);
    fn get_state(&self) -> WakeWordState;
    fn set_state(&mut self, state: WakeWordState);
}
```
