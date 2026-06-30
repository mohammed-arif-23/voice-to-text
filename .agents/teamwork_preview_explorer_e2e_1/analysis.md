# ScribeRx E2E Test Strategy & Workspace Verification Report

## Executive Summary
This report presents the verification results for compiling and testing ScribeRx on macOS, outlines a platform-independent abstraction and mocking strategy for Windows-specific components (including the new R6 Wake-Word Detection requirement), designs a Tauri E2E test harness using Playwright/tauri-driver, and plans a comprehensive multi-tier test suite covering 30+ core functional, non-functional, and safety-critical scenarios.

---

## 1. Workspace Build & Test Verification (macOS)

### 1.1 Direct Observations & Command Output
Running compilation commands in the workspace directory `/Users/mohammedarif/voice-to-text` resulted in shell command-not-found errors:
```bash
$ cargo check --workspace
zsh:1: command not found: cargo

$ rustc --version
rustc not found
```
**Conclusion 1**: The current macOS environment lacks the Rust toolchain (`cargo` / `rustc`).

### 1.2 Static Analysis of Platform Dependencies
A deep static analysis of the workspace crates reveals that even with Rust installed, compilation of the workspace on macOS will fail immediately due to hard platform coupling:

1. **`crates/core-hotkey`**:
   - **Dependency**: Directly lists `windows` crate (version `0.54.0`) under general `[dependencies]` in `Cargo.toml`. The `windows` crate is Windows-only and fails to compile or link on macOS.
   - **Imports**: Direct imports from `windows::Win32` (e.g., `SendInput`, `OpenClipboard`, `GetGUIThreadInfo`, `GetAsyncKeyState`) at the top of `src/lib.rs` without any conditional compilation gating (`#[cfg(target_os = "windows")]`).
   - **API Usage**: Platform-specific Win32 logic (e.g., polling key states, keyboard injection, Win32 Clipboard memory allocation) is implemented directly.
2. **`crates/app-shell`**:
   - **Dependency**: Directly lists `windows` crate under general dependencies in `Cargo.toml`.
   - **Imports & Logic**: Calls Win32 APIs like `GetForegroundWindow` and `SetForegroundWindow` in `src/main.rs` directly.
3. **`crates/storage`**:
   - Uses standard SQLite (`rusqlite`) with `"bundled"` feature, which compiles on macOS. However, the planned DPAPI key derivation (Milestone 1) is Windows-specific and will fail to compile on macOS without conditional compilation.
4. **`crates/stt-engine`**:
   - Spawns Python's `faster-whisper` daemon via `Command::new("python")`. This compiles on macOS but would fail at runtime if `python` or `faster-whisper` is missing.

### 1.3 Recommended Remediation for Cross-Platform Compilation
To enable testing and development of the app-shell and terminology matcher on macOS, we propose introducing a target-gated module architecture.

**Proposed Cargo.toml Target-Gating (`crates/core-hotkey/Cargo.toml`)**:
```toml
[package]
name = "core-hotkey"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
thiserror = { workspace = true }

[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.54.0", features = ["Win32_UI_Input_KeyboardAndMouse", "Win32_UI_WindowsAndMessaging", "Win32_Foundation", "Win32_System_DataExchange", "Win32_System_Memory", "Win32_Graphics_Gdi"] }
```

**Proposed Conditional Compilation Structure (`crates/core-hotkey/src/lib.rs`)**:
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

// Windows implementation
#[cfg(target_os = "windows")]
mod windows_impl {
    use super::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::SendInput;
    // ... Win32 imports and logic ...
    pub struct WindowsTextInjector;
    impl TextInjector for WindowsTextInjector { ... }
}

// Non-Windows (macOS/Linux) mock implementation
#[cfg(not(target_os = "windows"))]
mod fallback_impl {
    use super::*;
    pub struct FallbackTextInjector;
    impl TextInjector for FallbackTextInjector {
        fn inject(&self, text: &str, _strategy: InjectionStrategy) -> Result<(), String> {
            println!("[Mock Injection]: {}", text);
            Ok(())
        }
        fn capture_caret_position(&self) -> Option<(i32, i32)> {
            Some((100, 100))
        }
    }
}
```

---

## 2. Windows-Specific Components Mocking Strategy

To test ScribeRx locally and in CI/CD without physical hardware, Windows OS dependencies, or GPU resources, we define the following mocking strategies:

```
┌────────────────────────────────────────────────────────────────────────┐
│                              ScribeRx APP                              │
│                                                                        │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────────┐  │
│  │   core-hotkey    │  │   stt-engine     │  │      core-audio      │  │
│  ├──────────────────┤  ├──────────────────┤  ├──────────────────────┤  │
│  │ [TextInjector]   │  │ [SttEngine]      │  │ [AudioRecorder]      │  │
│  │  ▲            ▲  │  │  ▲            ▲  │  │  ▲            ▲      │  │
│  └──│────────────│──┘  └──│────────────│──┘  └──│────────────│──────┘  │
└─────│────────────│────────│────────────│────────│────────────│─────────┘
      │ (Prod)     │ (Test) │ (Prod)     │ (Test) │ (Prod)     │ (Test)
┌───────────┐┌───────────┐┌───────────┐┌───────────┐┌───────────┐┌───────────┐
│Win32 APIs ││Mock       ││Python     ││Mock STT   ││CPAL       ││Mock Wave  │
│Clipboard  ││Injector   ││Whisper    ││Engine     ││WASAPI     ││Reader     │
└───────────┘└───────────┘└───────────┘└───────────┘└───────────┘└───────────┘
```

### 2.1 core-hotkey (windows-rs, Keyboard, Clipboard, and Caret Position)
- **Problem**: Accessing `windows-rs` APIs for keystroke simulation, clipboard manipulation, active window HWND querying, and caretaker bounds results in compile/runtime crashes outside Windows.
- **Strategy**:
  1. Define a `ClipboardDevice` trait to isolate OS clipboard interactions.
  2. Implement `MockTextInjector` that records the injected string, strategy used, and simulates caret coordinates.
  3. Create an active window hook mock that returns a static mock HWND handle (`0xDEADC0DE`) and mocks EMR application titles (e.g. `"Chrome - PracticeFusion EHR"`).
  4. Build a programmatic `MockHotkeyListener` that can be triggered via Tauri IPC command `trigger_mock_hotkey` during E2E test runs.

### 2.2 storage (DPAPI & SQLCipher Encryption)
- **Problem**: SQLCipher utilizes Windows DPAPI (`CryptProtectData`) to derive database keys using Windows User Credentials and Windows Hello PINs, which are missing on macOS.
- **Strategy**:
  1. Create a `KeyDerivationProvider` trait.
  2. In production, the provider executes Windows DPAPI calls.
  3. In test/fallback environments, it uses a mock provider that returns a fixed SHA-256 derived hash of the test PIN, bypassing DPAPI.
  4. Use an in-memory SQLite storage database (`:memory:`) or a temporary unencrypted file database during automated tests to verify query logic without encryption latency or credential prompts.

### 2.3 stt-engine & transcribe.py (Whisper Python Daemon)
- **Problem**: Running a persistent Python daemon loading a GGUF Whisper model takes ~1-2 GB RAM, requires Python dependencies, and takes 2-3 seconds per transaction, making E2E tests slow and flakey.
- **Strategy**:
  1. Refactor `SttEngine` with a dependency injection structure.
  2. Implement `MockSttEngine`: It ignores raw PCM float arrays and instead inspects a test-configured global registry or extracts a mock payload mapped to the input.
  3. For E2E daemon interface testing, write a `mock_transcribe.py` script that conforms to the stdin/stdout text protocol. When it reads a WAV file path, it matches the filename against a lookup table (e.g., `test_audio_1.wav` -> `"Tab Amlokind 5 mg"`) and outputs the transcription immediately.

### 2.4 core-audio (CPAL Audio Stream)
- **Problem**: Automated test agents running headlessly have no microphone access and cannot output natural human speech.
- **Strategy**:
  1. Introduce `MockAudioRecorder` which implements `AudioRecorder`.
  2. The mock recorder is initialized with a specific WAV audio fixture file.
  3. When `start_recording` is called, it spawns a background thread that reads the WAV file chunks and streams the float PCM samples at the configured rate, calculating mock RMS level updates.
  4. When `stop_recording` is called, it returns the PCM samples parsed from the WAV fixture file.

### 2.5 R6: On-Device Wake-Word Detection
- **Problem**: The Wake-Word Engine operates in an "Armed" background listening state, constantly parsing input to find the wake phrase ("Hey ScribeRx" or "Start ScribeRx"). We must verify transitions without speaking.
- **Strategy**:
  1. Create a `WakeWordEngine` trait with states: `Armed`, `WakeDetected`.
  2. The `MockWakeWordEngine` supports programmatic trigger via Tauri IPC command `trigger_wake_word`.
  3. To test ward noise and masks, feed specific audio sample fixtures (e.g., `wake_word_ward_noise_80db.wav`) into the `MockWakeWordEngine` and assert whether it successfully transitions to `WakeDetected` or remains `Armed`.

---

## 3. E2E Test Harness & Mock Runner Design (Tauri App-Shell)

We design a Tauri-focused test harness that couples Rust compile-time feature flags with a headless Node.js test runner (Playwright).

### 3.1 Crate Dependency Injection Architecture
In `crates/app-shell/src/main.rs`, we define the `AppState` using traits rather than concrete structs to support dependency injection:

```rust
pub struct AppState {
    pub recording: Mutex<bool>,
    pub audio: Box<dyn AudioRecorder>,
    pub stt: Box<dyn SttEngine>,
    pub matcher: AdvancedDrugMatcher,
    pub injector: Box<dyn TextInjector>,
    pub storage: Box<dyn StorageEngine>,
}
```

We instantiate these based on the `e2e-test` compile feature flag:
```rust
#[cfg(feature = "e2e-test")]
fn initialize_components() -> AppState {
    AppState {
        recording: Mutex::new(false),
        audio: Box::new(core_audio::DummyAudioRecorder::new()),
        stt: Box::new(stt_engine::DummySttEngine),
        matcher: AdvancedDrugMatcher::new(),
        injector: Box::new(core_hotkey::DummyInjector),
        storage: Box::new(storage::DummyStorageEngine),
    }
}

#[cfg(not(feature = "e2e-test"))]
fn initialize_components() -> AppState {
    AppState {
        recording: Mutex::new(false),
        audio: Box::new(core_audio::CpalAudioRecorder::new()),
        stt: Box::new(stt_engine::WindowsSapiEngine::new()),
        matcher: AdvancedDrugMatcher::new(),
        injector: Box::new(core_hotkey::WindowsTextInjector::new()),
        storage: Box::new(storage::SqliteStorageEngine::new("scriberx_audit.db").unwrap()),
    }
}
```

### 3.2 IPC Testing Bridge (Tauri Commands)
For E2E tests, we expose special commands to control the mocks from the test runner:
```rust
#[cfg(feature = "e2e-test")]
#[tauri::command]
async fn e2e_set_mock_transcription(text: String, state: tauri::State<'_, Arc<AppState>>) -> Result<(), String> {
    // Dynamically set the text that DummySttEngine will return on next transcribe call
    Ok(())
}

#[cfg(feature = "e2e-test")]
#[tauri::command]
async fn e2e_trigger_hotkey(state: tauri::State<'_, Arc<AppState>>, app_handle: tauri::AppHandle) -> Result<(), String> {
    // Programmatically trigger toggle_dictation simulation
    toggle_dictation(&app_handle, &state).await
}
```

### 3.3 Playwright Test Runner Specification
The E2E test runner controls the Tauri application window via `tauri-driver` (which implements the WebDriver protocol for Tauri Webview).

**`tests/e2e/launcher.spec.ts`**:
```typescript
import { _test as tauriTest, expect } from '@tauri-apps/moby';
import { chromium, Page } from 'playwright';

// Setup tauri-driver connection
const test = tauriTest.extend({
  page: async ({}, use) => {
    const browser = await chromium.connectOverCDP('http://localhost:4444'); // tauri-driver port
    const contexts = browser.contexts();
    const page = contexts[0].pages()[0];
    await use(page);
  }
});

test.describe('ScribeRx E2E Clinical Dictation Workflow', () => {
  test('should display listening visualizer on hotkey trigger', async ({ page }) => {
    // 1. Simulate hotkey press via IPC bridge
    await page.evaluate(() => window.__TAURI__.invoke('e2e_trigger_hotkey'));
    
    // 2. Assert body class changes to listening state (Design System compliant)
    await expect(page.locator('body')).toHaveClass('state-listening');
    
    // 3. Verify Aurora canvas element is rendered
    const canvas = page.locator('#sine-canvas');
    await expect(canvas).toBeVisible();
  });

  test('should format clinical transcription and handle low-confidence alternatives', async ({ page }) => {
    // Pre-seed mock STT engine with a low-confidence transcript containing "paracetamal" (fuzzy match to Paracetamol)
    await page.evaluate(() => 
      window.__TAURI__.invoke('e2e_set_mock_transcription', { text: "Tab paracetamal 650 mg once daily" })
    );

    // Trigger capture cycle
    await page.evaluate(() => window.__TAURI__.invoke('e2e_trigger_hotkey')); // Start
    await page.waitForTimeout(500);
    await page.evaluate(() => window.__TAURI__.invoke('e2e_trigger_hotkey')); // Stop -> processing -> review

    // Assert UI expands to review low confidence alternatives
    await expect(page.locator('body')).toHaveClass('state-review-low');
    
    // Assert alternative chip exists for the misspelled drug
    const altChip = page.locator('#alt-chips button').first();
    await expect(altChip).toHaveText('Paracetamol');
    
    // Click alternative chip to select and inject
    await altChip.click();
    
    // Assert state resets to idle after injection
    await expect(page.locator('body')).toHaveClass('state-idle');
  });
});
```

---

## 4. Core Features of ScribeRx

Synthesized from `PROJECT.md`, `PRD.md`, and the parent agent's instruction, the core features under test are:

1. **F1: On-Device Wake-Word Detection (R6)**
   - Local low-power wake phrase monitoring ("Armed" state, <2% CPU overhead).
   - State transition: Armed -> Wake Detected -> Listening.
   - Screen summon and audio chime alerts on wake-word detection.
   - Robustness under background ward noise and masks.
2. **F2: Global Hotkey Summons & Session Management**
   - Summon visualizer near active caret position using configurable hotkeys (`Ctrl+Alt+Space` or `Ctrl+Alt+F9`).
   - Prevent hotkey conflicts with native EMR keyboard shortcuts.
   - Target focus window capture and automatic restoration.
3. **F3: Ambient Audio Capture & VAD**
   - CPAL input device routing (USB headsets, Bluetooth codecs, default mics).
   - Voice Activity Detection (8s silence timeout, 2s pause autostop).
   - Live microphone RMS level visualizer (aurora liquid gradients).
4. **F4: Speech-to-Text Transcription**
   - Execution of local whisper.cpp/faster-whisper python daemon.
   - Inject initial medical prompt tokens to bias transcription.
5. **F5: Medical Terminology Correction & Dosage Immutability**
   - Phonetic/Levenshtein matching against CDSCO/NLEM databases.
   - Strict confidence categorization (High, Medium, Low).
   - Immutability of numeric dosages (never fuzzy-matched or auto-corrected).
   - User-defined custom doctor vocabulary integration.
6. **F6: Target-Field Text Injection**
   - Multiple injection strategies: `SendInput`, `ClipboardPaste`, and `UiAutomation`.
   - Clipboard preservation: save and restore original clipboard content within 100ms.
   - Maintain the host EMR's native undo (`Ctrl+Z`) stack.
7. **F7: Local Encrypted Storage & Compliance**
   - SQLite + SQLCipher logging of clinical audits (FHIR compliance).
   - DPAPI credential-derived key encryption.
   - Zero-retention of raw audio data post-transcription.

---

## 5. Comprehensive Test Plan (Tiers 1-4)

### Tier 1: Feature Coverage (>=5 per feature)

#### F1: Wake-Word Detection (R6)
- **T1.F1.1**: Transition from `Armed` to `Wake Detected` to `Listening` upon feeding wake-word WAV.
- **T1.F1.2**: Assert active window displays floating capsule UI immediately on wake detection.
- **T1.F1.3**: Assert system audio chime plays on transition to `Listening`.
- **T1.F1.4**: CPU idle benchmark: measure CPU usage over 60 seconds of `Armed` state, verify average is < 2%.
- **T1.F1.5**: Privacy guardrail check: verify zero audio buffers are written to the temp directory or database before the wake-word is triggered.

#### F2: Global Hotkey Summons
- **T1.F2.1**: Summons UI in `Listening` state on global key combination event.
- **T1.F2.2**: Caret coordinate placement: verify Tauri window coordinates align with the active application cursor bounds.
- **T1.F2.3**: Rapid double-trigger: verify hotkey toggles start/stop without crashing the state machine.
- **T1.F2.4**: Capture active window HWND handle on summon.
- **T1.F2.5**: Confirm focus shifts back and restores original window focus post-dismissal.

#### F3: Audio Capture & VAD
- **T1.F3.1**: CPAL default mic audio buffer extraction verification.
- **T1.F3.2**: RMS amplitude visual output stream matching current input dB.
- **T1.F3.3**: Auto-trim silence: verify leading/trailing silent PCM samples are truncated.
- **T1.F3.4**: Initial silence timeout: verify app returns to `Idle` and hides window after 8 seconds of silent input.
- **T1.F3.5**: Autostop pause: verify recording stops and transcription triggers after 2 seconds of silence post-speech.

#### F4: Speech-to-Text Transcription
- **T1.F4.1**: Whisper daemon initialization and READY state handshake check.
- **T1.F4.2**: Verify transcription matching with standard medical phrases.
- **T1.F4.3**: Confirm initial medical prompting bias towards clinical names (e.g. "Amoxyclav" instead of "amoxy claff").
- **T1.F4.4**: Transcription performance benchmark: processing time ≤ real-time x 3.
- **T1.F4.5**: Whisper daemon recovery: verify daemon auto-restarts if process terminates mid-execution.

#### F5: Medical Terminology & Dosage Immutability
- **T1.F5.1**: CDSCO Database Match: verify "paracetamal" corrects to "Paracetamol" with high/medium confidence.
- **T1.F5.2**: Custom Doctor Vocabulary: verify injected custom list drug name corrects with high confidence.
- **T1.F5.3**: Confidence Classification: verify similarity < 0.65 flags as `ConfidenceLevel::Low`.
- **T1.F5.4**: Dosage Immutability (Safety Critical): verify "Dolo 65 mg" is never corrected to "Dolo 650 mg" despite Levenshtein proximity.
- **T1.F5.5**: Alternatives display: verify low-confidence terms list alternatives in the UI chips container.

#### F6: Text Injection
- **T1.F6.1**: `SendInput` keystroke injection into standard active notepad text area.
- **T1.F6.2**: `ClipboardPaste` text injection and clipboard recovery: original clipboard text must match post-test.
- **T1.F6.3**: Clipboard paste timing: verify clipboard restoration executes exactly 100ms after simulated `Ctrl+V`.
- **T1.F6.4**: Caret preservation: verify cursor index is at the end of injected text.
- **T1.F6.5**: Undo integrity: verify hitting `Ctrl+Z` removes the injected block as a single unit in Notepad.

#### F7: Local Encrypted Storage & Compliance
- **T1.F7.1**: SQLCipher database encryption validation (confirm database header is encrypted).
- **T1.F7.2**: DB key derivation validation via DPAPI fallback testing.
- **T1.F7.3**: Write audit log entry post-injection and verify fields (timestamp, EMR, text, confidence).
- **T1.F7.4**: Verify zero audio files or raw audio bytes remain in temp directories or SQLite databases post-transcription.
- **T1.F7.5**: Fetch recent audit logs with size limit parameter.

---

### Tier 2: Boundary & Corner Cases (>=5 per feature)

#### F1: Wake-Word Detection (R6)
- **T2.F1.1**: Ward noise tolerance: test wake detection accuracy with background clinic noise at 80dB (must not false trigger).
- **T2.F1.2**: Mask muffled speech: verify wake-word triggers correctly when speaker wears a surgical mask.
- **T2.F1.3**: Bluetooth profile transition: verify wake-word triggers during switch between A2DP (music) and HFP (hands-free) mic modes.
- **T2.F1.4**: Near-miss wake phrases (e.g. "Hey Scribe", "Hey ScribeText"): must remain in `Armed` state.
- **T2.F1.5**: Extremely rapid succession wake words: verify state machine does not lock up.

#### F2: Global Hotkey Summons
- **T2.F2.1**: Host application crash: active window closes mid-recording (verify ScribeRx degrades gracefully and discards session).
- **T2.F2.2**: Focus stolen during listening: another background application pops up (verify ScribeRx stays on top).
- **T2.F2.3**: Hotkey pressed while Tauri settings UI is focused: check event bubbling.
- **T2.F2.4**: Triggering hotkey when default input device is disconnected mid-action.
- **T2.F2.5**: Changing hotkey mappings to a key combination already registered by the OS (e.g., `Ctrl+Alt+Del`).

#### F3: Audio Capture & VAD
- **T3.F3.1**: Input clipping: audio input levels exceed maximum headroom (gain saturation); verify VAD does not crash.
- **T3.F3.2**: Rapid alternations of speech/silence (stuttering): verify VAD state stability.
- **T3.F3.3**: High gain static hum (background AC noise): check VAD noise gate calibration.
- **T3.F3.4**: Dynamic audio device detachment: user unplugging USB headset during recording.
- **T3.F3.5**: Extremely short utterance (<100ms click/pop): verify ignored as noise and session remains active.

#### F4: Speech-to-Text Transcription
- **T2.F4.1**: Input audio file with zero bytes/empty PCM buffer: verify STT returns empty string instead of hanging.
- **T2.F4.2**: Overly long dictation (>60 seconds of speech): verify model handles segment chunking without memory leak.
- **T2.F4.3**: Medical prompt buffer overflow (injecting thousands of custom words): verify Whisper initial prompt truncates safely.
- **T2.F4.4**: Mixed languages: dictation containing Tamil medical slang (verify phonetic transcription behaves stably).
- **T2.F4.5**: Unicode encoding mismatch in Python stdout pipe.

#### F5: Medical Terminology & Dosage Immutability
- **T2.F5.1**: Homophones / phonetic ambiguities (e.g., "Amlokind" vs "Amloxin"): verify confidence levels are set to Low, offering both alternatives.
- **T2.F5.2**: Decimal dosages ("0.25 mg", ".5 ml"): verify no deletion of decimal points (which would multiply dosage by 10-100x).
- **T2.F5.3**: Multi-word drug name matches (e.g., "Salicylic Acid Coal Tar Ointment"): verify matched as a single cohesive token.
- **T2.F5.4**: Numerical dosages with no space ("500mg"): verify parser splits unit safely and preserves the quantity "500".
- **T2.F5.5**: Drug names resembling numbers (e.g. "3-Monox"): verify no fuzzy matching clobbers the numerical digit.

#### F6: Text Injection
- **T2.F6.1**: Injection into non-editable host UI container (e.g. PDF viewer, desktop icon background): verify system handles failure gracefully.
- **T2.F6.2**: Extremely large clipboard payload prior to dictation (e.g. 50 MB image): verify clipboard backup/restore succeeds without memory overflow or exceeding the 100ms timing window.
- **T2.F6.3**: Clipboard lock: another process locks clipboard during paste window (verify fallback to `SendInput` keystrokes).
- **T2.F6.4**: Target field max-length constraint (e.g. input field capped at 20 chars): verify keystrokes truncate without app crash.
- **T2.F6.5**: Virtual desktop focus: cursor is on a different virtual desktop.

#### F7: Local Encrypted Storage & Compliance
- **T2.F7.1**: SQLCipher encryption write failure due to disk space exhaustion.
- **T2.F7.2**: Concurrent writes: logging audit logs during database backup operation.
- **T2.F7.3**: Special/Malicious character injection in audit log (SQL injection attempt, e.g. `' OR '1'='1`).
- **T2.F7.4**: User revokes Windows Hello credentials: key derivation check must prompt fallback password entry.
- **T2.F7.5**: Database corruption: corrupt header bytes (verify fallback rebuild or fail-safe deletion).

---

### Tier 3: Cross-Feature Combinations (Pairwise Coverage)

Pairwise coverage ensures that combinations of features function seamlessly together.

| Wake-Word / Hotkey | Audio & VAD | STT & Matcher | Injection Strategy | Storage & Logging | Expected Outcome |
|---|---|---|---|---|---|
| **Hotkey Summon** | 2s Pause Autostop | Low Confidence Match | Clipboard Paste | Audit Logging On | UI expands, user selects alternative, clipboard restored, logged. |
| **Wake-Word Summon** | 8s Silence Timeout | Empty Transcript | None (Cancelled) | Audit Logging On | UI shows timeout, hides, clipboard unmodified, no audit entry logged. |
| **Hotkey Summon** | Device Detached | Process Error | SendInput (Fallback) | Audit Logging On | UI displays error state, dismisses, clipboard unaffected, error logged. |
| **Wake-Word Summon** | 2s Pause Autostop | High Confidence Match | Clipboard Paste | Audit Logging Off | UI flashes green, text injected instantly, clipboard restored, no log written. |
| **Hotkey Summon** | 2s Pause Autostop | Dosage Discrepancy (Low) | Clipboard Paste | Audit Logging On | UI displays alternatives, user confirms raw text, dosage injected verbatim, audit logged. |

#### Selected Cross-Feature Test Designs:
- **T3.X1 (Hotkey + VAD + Matcher + Storage)**: Summon via hotkey, speak a prescription containing a medium-confidence drug, allow VAD to autostop, verify review window displays correct highlighting, confirm injection, and assert SQLCipher database logs the correct entry.
- **T3.X2 (Wake-Word + VAD + Clipboard Preservation)**: Trigger via "Hey ScribeRx" wake-word, copy text "Patient Data" to system clipboard, speak dictation, select low-confidence alternative, inject via ClipboardPaste, and verify that the clipboard contents match "Patient Data" after 120ms.
- **T3.X3 (Wake-Word + Audio Timeout + Focus)**: Trigger wake-word, remain silent, check that 8s initial timeout triggers UI dismissal, focus returns to EMR window, and CPU load stays below 2%.

---

### Tier 4: Real-World Application Scenarios

#### T4.Scenario 1: Doctor Anita's High-Noise Clinic Workflow (Chrome EMR)
- **Actor**: Dr. Anita (wearing a mask)
- **Environment**: Chrome Browser focused on Practo EHR notes text area. Noise level: 75dB (air conditioning + patient chatter). Clipboard contains a patient medical ID.
- **Steps**:
  1. Dr. Anita says "Hey ScribeRx".
  2. Audio chime alerts, capsule window pops up below the text field.
  3. Dr. Anita dictates: *"Tab Glycomet 500 mg twice daily, Tab Amlokind 5 mg once daily."*
  4. VAD detects 2-second pause after she stops talking.
  5. Capsule changes to "Analyzing..." state.
  6. The STT engine transcribes. Drug Matcher identifies "Glycomet" (High confidence, 500 mg dosage) and "Amlokind" (High confidence, 5 mg dosage).
  7. Since all confidence levels are High, the window automatically flashes green and injects the text via `ClipboardPaste` after 600ms.
  8. ScribeRx restores the clipboard contents back to the patient medical ID.
- **Assertions**:
  - Text injected into Chrome: `"Tab Glycomet 500 mg twice daily, Tab Amlokind 5 mg once daily."`
  - Clipboard contents: Matches original patient medical ID.
  - Active EMR window remains focused.
  - Audit log entries: Written to SQLCipher with correct data.

#### T4.Scenario 2: Notepad Prescription with Manual Alternative Selection
- **Actor**: Doctor using local Notepad for offline prescription writing.
- **Steps**:
  1. Doctor presses `Ctrl+Alt+Space`.
  2. Dictates: *"Tab Amoxiclav 625 mg at bedtime."* (whisper outputs `"amoxy claff 625 mg"`).
  3. Press hotkey again to stop capture.
  4. Matcher detects `"amoxy claff"` and fuzzy matches to `"Amoxyclav"` (Medium confidence) and `"Amoxicillin"` (Low confidence).
  5. The UI transitions to `.state-review-low` and displays selection chips.
  6. Doctor presses `Tab` and `Enter` (keyboard-only navigation) to select "Amoxyclav".
  7. Injected text inserts into Notepad.
  8. Doctor presses `Ctrl+Z` in Notepad.
- **Assertions**:
  - Word "Amoxyclav" is inserted, and dosage "625 mg" is preserved exactly.
  - Notepad undo (`Ctrl+Z`) reverts the entire block at once.

#### T4.Scenario 3: Armed Mode Background Surveillance (Battery & Power Constraint)
- **Actor**: Background daemon execution.
- **Steps**:
  1. System starts and enters `Armed` mode (wake-word activation active).
  2. ScribeRx runs in background for 2 hours with ongoing environment noise (patient-doctor talk, clinic sounds not matching the wake phrase).
- **Assertions**:
  - Average CPU load is verified to be < 2%.
  - Memory heap size remains stable (< 30 MB RAM) with zero leaks.
  - No temporary audio WAV files are created on disk.

---

## 6. Verification Method

To verify these test plans and execute build validations on a machine where cargo/rustup is installed:

1. **Verify Crate Gating & Compilation (Windows/macOS fallback)**:
   ```bash
   # Build the workspace to check platform-gated targets
   cargo check --workspace --all-targets
   ```
2. **Execute Crate Unit Tests**:
   ```bash
   # Run terminology, DB matching, and resampler logic tests
   cargo test --workspace
   ```
3. **Execute Tauri E2E Webdriver Suite**:
   ```bash
   # Install frontend E2E dependencies
   npm install
   # Run the mock E2E runner via Playwright
   npm run test:e2e
   ```
4. **Invalidation Conditions**:
   - If the EMR focus window handle is lost post-transcription, check `target_window` HWND tracking in `app-shell/src/main.rs`.
   - If dosage values are altered during Levenshtein matching, check the numeric token exclusion rules in `drug-match/src/lib.rs`.
