# Universal Dictation Development Plan

This plan outlines the architecture, milestones, and tasks for building the production-grade, commercially shippable Universal Voice-to-Text desktop application.

## Milestones and Tasks

### Milestone 1: Solution Foundations & Core Interfaces
- **Goal:** Establish the Visual Studio solution structure, project files, central package management, build settings, and core types/interfaces.
- **Tasks:**
  - **Task 1.1:** Initialize the Visual Studio Solution `UniversalDictation.sln` containing the 12 projects under `src/` and `tests/` as specified in R1.
  - **Task 1.2:** Configure `.editorconfig` (4-space indent for C#, sorting system directives) and `.gitignore` appropriate for .NET/WPF.
  - **Task 1.3:** Setup central package management ensuring individual projects do not specify package versions.
  - **Task 1.4:** Define core domain interfaces (`IAudioCaptureService`, `ITranscriptionProvider`, `ITextInsertionAdapter`, etc.) and domain types (`TargetContext`, `TranscriptSegment`, `SessionId`, error types) in `Desktop.Core` (R3).
  - **Task 1.5:** Verify clean compile of all skeleton projects in Release mode.

### Milestone 2: Core Logic & Logging Redaction
- **Goal:** Implement the dictation session state machine, logging redaction, and voice command parser.
- **Tasks:**
  - **Task 2.1:** Implement the thread-safe `DictationSessionStateMachine` in `Desktop.Core` enforcing all 16 states and legal transitions (R2).
  - **Task 2.2:** Configure Serilog logging pipeline with custom enrichment/destructuring policies to redact transcript, API keys, and clipboard contents (R4).
  - **Task 2.3:** Implement a locale-aware deterministic `VoiceCommandParser` for the specified voice command phrases (R7).
  - **Task 2.4:** Write unit tests demonstrating 100% state coverage, illegal transitions throwing correct exception, and log redaction of sensitive properties.

### Milestone 3: Audio Capture, Hotkeys & Targeting
- **Goal:** Develop audio recording, global key bindings, and active focus context detection.
- **Tasks:**
  - **Task 3.1:** Implement `WasapiAudioCaptureService` in `Desktop.Audio` using NAudio, resampling to 16kHz PCM16, ring buffering, and device hot-plug detection (R5).
  - **Task 3.2:** Implement `GlobalHotkeyService` in `Desktop.Targeting` using Win32 API (`RegisterHotKey`) supporting push-to-talk and toggle modes (R8).
  - **Task 3.3:** Implement `TargetContextService` in `Desktop.Targeting` using Windows UI Automation to capture focused window/control metadata and ProcessIntegrityLevel (R10).
  - **Task 3.4:** Implement context revalidation checking for changes in ProcessId, RuntimeId, Sensitivity, or Integrity level before insertion (R10).
  - **Task 3.5:** Write unit/integration tests covering ring buffer overflow, hotkey conflict exceptions, password field classification, and revalidation failures.

### Milestone 4: Transcription Adapters & Reconciler
- **Goal:** Connect to streaming and offline STT engines.
- **Tasks:**
  - **Task 4.1:** Implement `DeepgramTranscriptionProvider` using WebSocket API (`wss://api.deepgram.com/v1/listen`) with interim/final segment handling (R6a).
  - **Task 4.2:** Implement `AzureTranscriptionProvider` using Microsoft Cognitive Services Speech SDK push-stream (R6b).
  - **Task 4.3:** Implement `WhisperOfflineTranscriptionProvider` using Whisper.net CPU-only with GGUF model path configuration (R6c).
  - **Task 4.4:** Implement `TranscriptReconciler` to merge interim/stable segments and parse voice commands on the fly.
  - **Task 4.5:** Write contract tests verifying correct JSON parsing, authentication failure mapping, and reconciler deduplication.

### Milestone 5: Safe Text Insertion Adapter Chain
- **Goal:** Implement secure insertion of transcribed text back to target applications.
- **Tasks:**
  - **Task 5.1:** Implement `InsertionAdapterChain` containing `BrowserExtensionAdapter`, `UiaValuePatternAdapter`, `SendInputAdapter`, and `ClipboardFallbackAdapter` (R11).
  - **Task 5.2:** Implement clipboard fallback preserving all data formats, with retries on lock and conditional restoration (R11).
  - **Task 5.3:** Implement post-insertion verification reading back the text and comparing.
  - **Task 5.4:** Write unit tests verifying fallback traversal, password/sensitive field blocking, and clipboard preservation.

### Milestone 6: WPF Desktop UI Overlay & Application Composition
- **Goal:** Create the no-focus overlay UI and wire together the application's composition root.
- **Tasks:**
  - **Task 6.1:** Implement the WPF overlay window applying `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW` and handling `WM_MOUSEACTIVATE` (R9).
  - **Task 6.2:** Design the overlay view showing state, scrolling transcript, handling scaling and high contrast (R9).
  - **Task 6.3:** Wire the DI container (Microsoft.Extensions.DependencyInjection) in `DesktopApp` and implement the MVVM controllers.
  - **Task 6.4:** Document test procedures for focus preservation and scaling.

### Milestone 7: Integration & Hardening
- **Goal:** Run E2E test suites and harden coverage with adversarial testing.
- **Tasks:**
  - **Task 7.1:** Integrate E2E test suite and pass all Tier 1-4 tests.
  - **Task 7.2:** Conduct adversarial coverage hardening (Tier 5) and resolve all identified bugs.
