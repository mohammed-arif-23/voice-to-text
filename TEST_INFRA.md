# Universal Dictation E2E Test Infrastructure Strategy and Plan

This document outlines the strategy, architecture, feature inventory, test cases, and verification procedures for the Universal Dictation end-to-end (E2E) testing framework.

---

## 1. Test Strategy and Methodology

The E2E testing framework is designed as a requirement-driven, opaque-box validator for the Universal Dictation system. It treats the application as a black box and interacts strictly through its public boundaries or designed testing interfaces (simulated audio inputs, global hotkey invocations, and virtual UI automation target windows).

Our testing approach adopts a strict 4-Tier classification:
- **Tier 1: Feature Coverage:** Asserts standard execution paths for each of the 8 features. Minimum 5 test cases per feature.
- **Tier 2: Boundary & Corner Cases:** Targets error handling, limits, validation exceptions, timeout paths, and edge conditions for each of the 8 features. Minimum 5 test cases per feature.
- **Tier 3: Cross-Feature Combinations:** Validates pairwise interactions and state transitions between major features. Minimum 8 test cases.
- **Tier 4: Real-World Application Scenarios:** Simulates full, end-to-end user workflows inside complex target desktop environments. Minimum 5 test cases.

---

## 2. Comprehensive Feature Inventory and Test Cases

The test suite validates 8 main features across 93 planned test cases. Below is the comprehensive index and detail for every test case.

### Feature 1: Session State Machine (R2)

Enforces thread-safe dictation session state transitions across all 16 states: `SignedOut`, `Idle`, `Arming`, `Capturing`, `Streaming`, `Finalizing`, `ReviewRequired`, `ReadyToInsert`, `ValidatingTarget`, `Inserting`, `Verifying`, `Completed`, `Cancelled`, `RecoverableFailure`, `FatalFailure`, and `Offline`.

#### Tier 1: Feature Coverage (5 Test Cases)
1. **`TC-SSM-T1-01`: Initial State Validation**
   - **Description:** Assert that when the state machine is initialized (assuming signed-in state), the starting state is exactly `Idle`.
   - **Expected Result:** Current state reports `Idle`.
2. **`TC-SSM-T1-02`: Standard Dictation Flow Transitions**
   - **Description:** Trigger normal dictation start. Transition through `Idle` -> `Arming` -> `Capturing` -> `Streaming`.
   - **Expected Result:** State transitions successfully and records timestamped transition records.
3. **`TC-SSM-T1-03`: Normal Finalization Transitions**
   - **Description:** Trigger dictation finalization from `Streaming`. Transition through `Streaming` -> `Finalizing` -> `ReadyToInsert`.
   - **Expected Result:** State transitions smoothly without errors.
4. **`TC-SSM-T1-04`: Normal Safe Insertion Transitions**
   - **Description:** Trigger insertion from `ReadyToInsert`. Transition through `ReadyToInsert` -> `ValidatingTarget` -> `Inserting` -> `Verifying` -> `Completed` -> `Idle`.
   - **Expected Result:** All states transition sequentially, confirming text is verified and written.
5. **`TC-SSM-T1-05`: Early User Cancellation Flow**
   - **Description:** Press hotkey to start, then immediately cancel. Transition through `Idle` -> `Arming` -> `Cancelled` -> `Idle`.
   - **Expected Result:** State resets to `Idle` with no audio data retained.

#### Tier 2: Boundary & Corner Cases (5 Test Cases)
6. **`TC-SSM-T2-01`: Direct Invalid State Transition Attempt**
   - **Description:** Attempt to force a transition directly from `Idle` to `Inserting`.
   - **Expected Result:** Throws `InvalidSessionTransitionException` and leaves state unchanged.
7. **`TC-SSM-T2-02`: Out-of-Order Finalization Transition**
   - **Description:** Attempt to transition from `Streaming` directly to `Completed` without intermediate validating and inserting phases.
   - **Expected Result:** Throws `InvalidSessionTransitionException`.
8. **`TC-SSM-T2-03`: Thread-Safety Race Condition Validation**
   - **Description:** Trigger 100 concurrent state transition requests from `Idle` to different legal and illegal next states.
   - **Expected Result:** State machine handles all requests deterministically; only one legal transition succeeds, others fail cleanly, state is not corrupted.
9. **`TC-SSM-T2-04`: Recoverable Network Glitch Transition**
   - **Description:** Simulate a network drop during `Streaming` state.
   - **Expected Result:** State transitions to `RecoverableFailure` and returns to `Idle` on manual reset.
10. **`TC-SSM-T2-05`: Fatal Target Mismatch Failure Transition**
    - **Description:** Simulate target context changing invalidly during `ValidatingTarget` state.
    - **Expected Result:** State transitions to `FatalFailure` and triggers immediate logging.

---

### Feature 2: Logging Redaction Pipeline (R4)

Configures Serilog with custom policies to redact properties tagged with `[Redacted]` (replacing with `***`), redacting transcripts, clipboard content, API tokens, file paths containing user names, and writing JSON output to file.

#### Tier 1: Feature Coverage (5 Test Cases)
11. **`TC-LRP-T1-01`: Pipeline Initialization and File Sink Creation**
    - **Description:** Initialize Serilog and assert that a rolling log file is created in `logs/dictation-.log`.
    - **Expected Result:** Log file is successfully created and writable.
12. **`TC-LRP-T1-02`: Basic Property Redaction**
    - **Description:** Log a structured event with a property explicitly tagged with `[Redacted]`.
    - **Expected Result:** The serialized log entry contains `"***"` instead of the property value.
13. **`TC-LRP-T1-03`: Transcript Text Redaction**
    - **Description:** Log a session model containing a dictated transcript text.
    - **Expected Result:** Transcript text is entirely replaced by `"***"` in the file log output.
14. **`TC-LRP-T1-04`: API Token Redaction**
    - **Description:** Log a configuration model containing an API token.
    - **Expected Result:** The API token is redacted.
15. **`TC-LRP-T1-05`: Clipboard Content Redaction**
    - **Description:** Log a clipboard fallback event containing user clipboard data.
    - **Expected Result:** Clipboard content is replaced with `"***"`.

#### Tier 2: Boundary & Corner Cases (5 Test Cases)
16. **`TC-LRP-T2-01`: Window Title Hash Source Redaction**
    - **Description:** Log a target window capture context containing a raw Window Title.
    - **Expected Result:** Raw Window Title text is redacted; only the SHA-256 hash is written.
17. **`TC-LRP-T2-02`: Sensitive File Path Redaction**
    - **Description:** Log an exception stack trace containing file paths like `C:\Users\john_doe\AppData`.
    - **Expected Result:** User names in directory paths are redacted (e.g. replaced with `C:\Users\***\AppData`).
18. **`TC-LRP-T2-03`: Deeply Nested Redaction Object Traversal**
    - **Description:** Log an object containing nested arrays and dictionary maps that contain `[Redacted]` values.
    - **Expected Result:** Nested properties are successfully traversed and redacted.
19. **`TC-LRP-T2-04`: Release Log Level Gate**
    - **Description:** In Release build configuration, emit `Debug` and `Information` logs.
    - **Expected Result:** Only `Information` level logs are written to the sink.
20. **`TC-LRP-T2-05`: Debug Log Level Gate**
    - **Description:** In Debug build configuration, emit `Debug` and `Information` logs.
    - **Expected Result:** Both `Debug` and `Information` logs are written to the sink.

---

### Feature 3: WASAPI Audio Capture (R5)

Exposes microphone enumeration, hot-plug notifications, Mono PCM 16kHz resampling, bounded ring buffering, and handles permission and conflict errors.

#### Tier 1: Feature Coverage (5 Test Cases)
21. **`TC-WAC-T1-01`: Microphone Enumeration Verification**
    - **Description:** Enumerate active devices via the audio interface.
    - **Expected Result:** Returns a list of active system audio capture endpoints.
22. **`TC-WAC-T1-02`: Device Arrival Event Trigger**
    - **Description:** Simulate a new audio input device arrival.
    - **Expected Result:** Capturer detects changes and raises `AudioDeviceChangedEvent`.
23. **`TC-WAC-T1-03`: Device Removal Event Trigger**
    - **Description:** Simulate removal of the active input device.
    - **Expected Result:** Capturer raises `AudioDeviceChangedEvent` and stops capture safely.
24. **`TC-WAC-T1-04`: Continuous Streaming Enumerator**
    - **Description:** Verify that `IAudioCaptureService.StreamFramesAsync()` yields frames continuously.
    - **Expected Result:** `IAsyncEnumerable<AudioFrame>` yields valid PCM frames.
25. **`TC-WAC-T1-05`: MediaFoundationResampler Resampling Verification**
    - **Description:** Capture audio at 44.1 kHz stereo and process it through the pipeline.
    - **Expected Result:** Emitted `AudioFrame` contains 16 kHz, 16-bit, Mono PCM format bytes.

#### Tier 2: Boundary & Corner Cases (5 Test Cases)
26. **`TC-WAC-T2-01`: Ring Buffer Overflow Event**
    - **Description:** Fill the 5-second ring buffer by simulating delayed consumption of frames.
    - **Expected Result:** Bounded buffer raises `AudioBufferOverflowEvent` when full.
27. **`TC-WAC-T2-02`: Ring Buffer Frame Drop (Non-Blocking)**
    - **Description:** When the ring buffer is overflowing, continue to input frames.
    - **Expected Result:** Older frames are dropped, capture thread does not block, no memory leak occurs.
28. **`TC-WAC-T2-03`: Missing Input Device Exception**
    - **Description:** Attempt to start capture when no microphone device is plugged in.
    - **Expected Result:** Throws `AudioCaptureException` with a specific Diagnostic Code.
29. **`TC-WAC-T2-04`: Access Denied Permission Exception**
    - **Description:** Simulate OS-level privacy settings denying microphone permission to the app.
    - **Expected Result:** Capturer throws `AudioCaptureException` representing Access Denied.
30. **`TC-WAC-T2-05`: Exclusive-Mode Conflict Handling**
    - **Description:** Attempt capture on a device locked by another process in exclusive mode.
    - **Expected Result:** Pipeline catches error, falls back or throws `AudioCaptureException`.

---

### Feature 4: Transcription Providers & Reconciler (R6)

Implements Deepgram Nova-3 (WSS), Azure (Push-stream), Whisper.net (Offline), and transcript segment reconciliation.

#### Tier 1: Feature Coverage (5 Test Cases)
31. **`TC-TPR-T1-01`: Deepgram WebSocket Query Construction**
    - **Description:** Verify that the WebSocket connection request contains all mandatory query parameters.
    - **Expected Result:** URL includes `model=nova-3`, `encoding=linear16`, `sample_rate=16000`, `interim_results=true`, `endpointing=200`, and `keyterm` values.
32. **`TC-TPR-T1-02`: Deepgram Interim Segment Parsing**
    - **Description:** Feed a recorded Deepgram interim JSON payload to the adapter.
    - **Expected Result:** Emits `TranscriptSegment` with `SegmentKind.Interim`.
33. **`TC-TPR-T1-03`: Deepgram Final Segment Parsing**
    - **Description:** Feed a recorded Deepgram final JSON payload to the adapter.
    - **Expected Result:** Emits `TranscriptSegment` with `SegmentKind.Final`.
34. **`TC-TPR-T1-04`: Azure Speech Adapter Mapping**
    - **Description:** Hook up Mock Azure SDK and trigger `Recognizing` and `Recognized` events.
    - **Expected Result:** Maps to `Interim` and `Final` `TranscriptSegment` values.
35. **`TC-TPR-T1-05`: Whisper Offline Buffer Processing**
    - **Description:** Pass a 3-second audio buffer of speech into `WhisperOfflineTranscriptionProvider`.
    - **Expected Result:** Transcribes correctly returning timestamps and text segments.

#### Tier 2: Boundary & Corner Cases (5 Test Cases)
36. **`TC-TPR-T2-01`: Whisper Missing Model File Exception**
    - **Description:** Attempt offline Whisper transcription when the configured GGUF file is missing.
    - **Expected Result:** Throws `OfflineModelNotFoundException` referencing the expected model path.
37. **`TC-TPR-T2-02`: Deepgram Token Authentication Failure**
    - **Description:** Connect to Deepgram using an invalid/expired token.
    - **Expected Result:** Adapter throws `ProviderAuthException`.
38. **`TC-TPR-T2-03`: Deepgram Network Outage & Reconnect**
    - **Description:** Simulate connection closure during active streaming (e.g. HTTP 429/503/socket close).
    - **Expected Result:** Reconnects with exponential backoff (up to 3 attempts), then fails with `RecoverableFailure`.
39. **`TC-TPR-T2-04`: Transcript Reconciler Interim Deduplication**
    - **Description:** Pass overlapping interim transcript revisions into `TranscriptReconciler`.
    - **Expected Result:** The reconciler stable-segment tracker removes duplication and yields clean output.
40. **`TC-TPR-T2-05`: Azure Speech SDK Cancellation Handling**
    - **Description:** Trigger Azure cancellation with various reasons (e.g., error, auth).
    - **Expected Result:** SDK cancellation is parsed, mapping details to a typed exception.

---

### Feature 5: Deterministic Voice Command Parser (R7)

Parses 16 voice command phrases (e.g. "new line", "delete last word", "scratch that") deterministically before insertion.

#### Tier 1: Feature Coverage (5 Test Cases)
41. **`TC-VCP-T1-01`: New Line Command Expansion**
    - **Description:** Input "new line" phrase into `VoiceCommandParser`.
    - **Expected Result:** Outputs action representing `\n` insertion.
42. **`TC-VCP-T1-02`: New Paragraph Command Expansion**
    - **Description:** Input "new paragraph" phrase into `VoiceCommandParser`.
    - **Expected Result:** Outputs action representing `\n\n` insertion.
43. **`TC-VCP-T1-03`: Punctuation Command Conversion**
    - **Description:** Input punctuation phrases: "comma", "period", "question mark", "colon", "semicolon".
    - **Expected Result:** Converts to `, `, `. `, `? `, `: `, `; ` respectively.
44. **`TC-VCP-T1-04`: Text Manipulation Commands**
    - **Description:** Input manipulation commands: "delete last word", "delete last sentence", "scratch that".
    - **Expected Result:** Command parser updates/clears active segment buffers.
45. **`TC-VCP-T1-05`: Dictation control Commands**
    - **Description:** Input control phrases: "stop dictation", "cancel dictation".
    - **Expected Result:** Emits control signals to state machine.

#### Tier 2: Boundary & Corner Cases (5 Test Cases)
46. **`TC-VCP-T2-01`: Non-Command Text Passthrough**
    - **Description:** Input normal dictated text (e.g., "The patient has a mild cough").
    - **Expected Result:** Passes through completely unmodified.
47. **`TC-VCP-T2-02`: Punctuation Word Adjacency Separation**
    - **Description:** Input phrase "hello comma world".
    - **Expected Result:** Parses command, outputting "hello, world".
48. **`TC-VCP-T2-03`: Rapid Command Combination**
    - **Description:** Input phrase "delete last word comma new line".
    - **Expected Result:** Executes word deletion first, then inserts a comma and a new line.
49. **`TC-VCP-T2-04`: Locale-Aware Matching (en-US)**
    - **Description:** Test command phrase "period" and "full stop" under en-US locale setting.
    - **Expected Result:** Both command phrases map to `. `.
50. **`TC-VCP-T2-05`: Stripping Commands Prior to Insertion**
    - **Description:** Process command "scratch that" at the end of a transcript buffer.
    - **Expected Result:** Clears transcript; no command strings or deleted words reach the text insertion adapter.

---

### Feature 6: Global Hotkeys (R8)

Implements global hotkeys via Win32 `RegisterHotKey`, supporting push-to-talk, toggle, conflict detection, and non-blocking asynchronous events.

#### Tier 1: Feature Coverage (5 Test Cases)
51. **`TC-GHS-T1-01`: Push-To-Talk Registration**
    - **Description:** Register VK_CAPITAL (CapsLock) as a push-to-talk hotkey.
    - **Expected Result:** Hotkey is successfully registered in the OS.
52. **`TC-GHS-T1-02`: Toggle-Mode Registration**
    - **Description:** Register Ctrl+Alt+D as a toggle hotkey.
    - **Expected Result:** Hotkey registered successfully.
53. **`TC-GHS-T1-03`: Hotkey Event Triggering**
    - **Description:** Simulate hotkey press event from the OS.
    - **Expected Result:** Dispatches trigger event, starting state machine arming.
54. **`TC-GHS-T1-04`: PTT Release Termination**
    - **Description:** Simulate hotkey release event in push-to-talk mode.
    - **Expected Result:** Dispatches finalization, stopping WASAPI capture.
55. **`TC-GHS-T1-05`: Cleanup OS Registration**
    - **Description:** Unregister hotkeys during service shutdown.
    - **Expected Result:** Calls Win32 `UnregisterHotKey` for all allocated hotkeys.

#### Tier 2: Boundary & Corner Cases (5 Test Cases)
56. **`TC-GHS-T2-01`: Conflict Detection and Exception Throwing**
    - **Description:** Attempt to register a hotkey that is already registered by the system or another app.
    - **Expected Result:** Throws `HotkeyConflictException` detailing the VK code.
57. **`TC-GHS-T2-02`: Safe Dispose Verification**
    - **Description:** Dispose the `GlobalHotkeyService` without explicitly unregistering hotkeys.
    - **Expected Result:** Destructor/Dispose automatically cleans up Win32 handles to prevent OS leaks.
58. **`TC-GHS-T2-03`: Asynchronous Non-Blocking Execution**
    - **Description:** Trigger hotkey events while the WPF UI thread is performing layout/drawing.
    - **Expected Result:** Hotkey messages are processed on a background thread without UI stutter.
59. **`TC-GHS-T2-04`: Rapid Debounce Filtering**
    - **Description:** Simulate extremely rapid key presses (e.g. 5 times within 100ms) on a toggle hotkey.
    - **Expected Result:** Filters rapid taps, maintaining stable capturing state.
60. **`TC-GHS-T2-05`: Invalid Hotkey Argument Validation**
    - **Description:** Attempt to register hotkeys with null modifiers or unbound keys.
    - **Expected Result:** Throws `ArgumentException`.

---

### Feature 7: No-Activate Overlay UI (R9)

WPF overlay window using `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW` and mouse hooks to prevent focus theft, showing status and scrolling transcript.

#### Tier 1: Feature Coverage (5 Test Cases)
61. **`TC-NAO-T1-01`: WPF Window Styles Application**
    - **Description:** Show the overlay window and query its Window Long styles via Win32.
    - **Expected Result:** Styles contain `WS_EX_NOACTIVATE` and `WS_EX_TOOLWINDOW` flags.
62. **`TC-NAO-T1-02`: Mouse Click Focus Retention**
    - **Description:** Focus a text editor window, then click inside the overlay window area.
    - **Expected Result:** `WM_MOUSEACTIVATE` returns `MA_NOACTIVATE`. Keyboard focus remains in the text editor.
63. **`TC-NAO-T1-03`: Visual State Indicator Representation**
    - **Description:** Programmatically cycle the overlay's view-model state through Idle, Arming, Capturing, and Error.
    - **Expected Result:** UI renders distinct color/icon configurations for each state.
64. **`TC-NAO-T1-04`: Transcript Text Scrolling**
    - **Description:** Feed a long transcription string (300 characters) into the overlay view-model.
    - **Expected Result:** Text is truncated to the last 200 characters and scrolls smoothly.
65. **`TC-NAO-T1-05`: Default Placement Location**
    - **Description:** Launch overlay UI on a standard monitor.
    - **Expected Result:** Window places itself near the system clock/tray area by default.

#### Tier 2: Boundary & Corner Cases (5 Test Cases)
66. **`TC-NAO-T2-01`: Window Coordinate Persistence**
    - **Description:** Drag the overlay window to custom coordinates, close, and reopen.
    - **Expected Result:** Coordinates persist in user settings and load on launch.
67. **`TC-NAO-T2-02`: Windows Display Scaling Legibility**
    - **Description:** Simulate high DPI display scales: 100%, 125%, 150%, 200%, 300%.
    - **Expected Result:** Text remains clear and within boundaries; overlay does not overflow screen.
68. **`TC-NAO-T2-03`: High Contrast Mode Toggle**
    - **Description:** Trigger Windows System High Contrast Mode.
    - **Expected Result:** Theme colors update automatically to high-contrast compliance.
69. **`TC-NAO-T2-04`: Drag Move Focus Check**
    - **Description:** Drag the overlay to reposition it while typing into Notepad.
    - **Expected Result:** Typing in Notepad is uninterrupted; focus is not stolen during the drag.
70. **`TC-NAO-T2-05`: Alt-Tab Hiding Validation**
    - **Description:** Open Alt-Tab task switcher dialog in Windows.
    - **Expected Result:** Overlay window is absent from the listed switchable applications.

---

### Feature 8: Target Context & Safe Insertion Chain (R10 & R11)

Captures focused control properties, revalidates process details before insertion, blocks password fields, and executes insertion chain with clipboard fallback.

#### Tier 1: Feature Coverage (5 Test Cases)
71. **`TC-TCS-T1-01`: Target Context Snapshot Capture**
    - **Description:** Trigger snapshot capture when an editor is active.
    - **Expected Result:** Captures ProcessId, ExecutableName, IntegrityLevel, and WindowHandle.
72. **`TC-TCS-T1-02`: Password Field Identification**
    - **Description:** Capture context when focused on a field with UI Automation `IsPassword == true`.
    - **Expected Result:** `SensitivityClassification` is classified as `Password`.
73. **`TC-TCS-T1-03`: Context Snapshot Matching Validation**
    - **Description:** Revalidate context with matching active window parameters.
    - **Expected Result:** Verification passes; insertion is allowed.
74. **`TC-TCS-T1-04`: Priority Chain Execution Order**
    - **Description:** Trigger text insertion. Disable Browser extension and UiaValuePattern.
    - **Expected Result:** Chain executes SendInput adapter; skips higher-priority ones cleanly.
75. **`TC-TCS-T1-05`: Post-Insertion Readback Verification**
    - **Description:** Insert text into a mock control and verify the inserted content.
    - **Expected Result:** Normalised text values match; adapter reports success.

#### Tier 2: Boundary & Corner Cases (5 Test Cases)
76. **`TC-TCS-T2-01`: ProcessId Switch Failure**
    - **Description:** Capture context. Before insertion, simulate focus switching to another process.
    - **Expected Result:** Revalidation fails; throws `TargetValidationException`.
77. **`TC-TCS-T2-02`: Process Integrity Elevation Failure**
    - **Description:** Capture context. Target process escalates integrity level.
    - **Expected Result:** Revalidation rejects mismatch; throws `TargetValidationException`.
78. **`TC-TCS-T2-03`: Password Field Block Policy**
    - **Description:** Attempt text insertion where `SensitivityClassification` is `Password`.
    - **Expected Result:** Insertion chain blocks execution; throws `SensitiveFieldBlockedException`.
79. **`TC-TCS-T2-04`: Clipboard Fallback Format Preservation**
    - **Description:** Execute clipboard fallback insertion while clipboard contains complex `IDataObject` structures.
    - **Expected Result:** Clipboard data is backed up, text is pasted, and the original rich objects are restored.
80. **`TC-TCS-T2-05`: Clipboard Fallback Mismatch Abort**
    - **Description:** Simulate external clipboard change during the backup and restore sequence.
    - **Expected Result:** Clipboard restoration is skipped to prevent overwriting new external clipboard data.

---

### Tier 3: Cross-Feature Combinations (8 Test Cases)

Validates complex interactions and states across different features of the application.

81. **`TC-COM-T3-01`: State Machine & Hotkey Integration**
    - **Description:** Verify that global hotkey triggers successfully drive state transitions in the state machine under rapid fire scenarios.
    - **Expected Result:** State machine accurately follows VK press/release events without throwing state errors.
82. **`TC-COM-T3-02`: State Machine & WASAPI Capture Event Integration**
    - **Description:** Simulate a WASAPI capture buffer overflow while the state machine is in `Streaming` state.
    - **Expected Result:** State machine transitions immediately to `RecoverableFailure` and stops capture.
83. **`TC-COM-T3-03`: Transcription & State Machine Command Stop Integration**
    - **Description:** Stream audio containing the "stop dictation" command to the reconciler during transcription.
    - **Expected Result:** Reconciler strips command and signals state machine to transition to `Finalizing`.
84. **`TC-COM-T3-04`: Overlay UI & State Machine Visualization**
    - **Description:** Cycle the state machine quickly through rapid transitions.
    - **Expected Result:** Overlay UI reflects state changes with correct colors and layouts without lag.
85. **`TC-COM-T3-05`: Target Context Mismatch & Error Overlay Integration**
    - **Description:** Cause context revalidation to fail while state machine is in `ValidatingTarget` state.
    - **Expected Result:** State machine transitions to `FatalFailure`, and the overlay UI renders the error message.
86. **`TC-COM-T3-06`: WASAPI Silence & Deepgram Connection Integration**
    - **Description:** Stream 30 seconds of pure silence from WASAPI to the Deepgram provider adapter.
    - **Expected Result:** Adapter sends periodic WebSocket KeepAlive messages; connection remains active.
87. **`TC-COM-T3-07`: Safe Insertion Fallback & Redaction Logging Integration**
    - **Description:** Perform clipboard fallback insertion. Ensure details are logged.
    - **Expected Result:** Serilog redaction catches clipboard contents and replaces them with `"***"`.
88. **`TC-COM-T3-08`: Voice Command Parser & Safe Insertion Integration**
    - **Description:** Voice-dictate a paragraph containing "new paragraph" commands.
    - **Expected Result:** Reconciler strips command, formatting the text with `\n\n` prior to insertion.

---

### Tier 4: Real-World Application Scenarios (5 Test Cases)

Verifies full end-to-end user workflows in typical real-world target desktop environments.

89. **`TC-RWS-T4-01`: Web-Based EMR Clinical Dictation (Chrome/Epic)**
    - **Description:** User focuses a text input field in Epic EMR running in Google Chrome, holds CapsLock (PTT), dictates notes, speaks "comma" and "new paragraph" commands, then releases CapsLock.
    - **Expected Result:** Capture is active, text streams onto the overlay. On release, revalidation succeeds, the `BrowserExtensionAdapter` inserts formatted text, and verification succeeds.
90. **`TC-RWS-T4-02`: Remote Desktop Dictation with Offline Whisper Fallback**
    - **Description:** User dictating into Notepad inside an RDP (Remote Desktop Connection) session. During dictation, the network drops.
    - **Expected Result:** Deepgram stream fails; state machine falls back to `Offline` and collects local audio. Whisper.net transcribes the audio, and the `SendInputAdapter` inserts text into the RDP session.
91. **`TC-RWS-T4-03`: Focus Hijacking & Target Application Switch Mismatch**
    - **Description:** User begins dictating into an active Word document. During dictation, an administrator password window pops up, stealing keyboard focus.
    - **Expected Result:** User finishes dictating. Target revalidation runs and detects the active window has switched to a high-integrity password window. Validation fails, blocking insertion and transitioning to `FatalFailure`.
92. **`TC-RWS-T4-04`: Legacy App Dictation with Clipboard Lock & Paste Fallback**
    - **Description:** User dictates into a legacy medical software field where UIA and SendInput are blocked due to integrity levels. The clipboard is locked by a clipboard utility.
    - **Expected Result:** Insertion chain uses `ClipboardFallbackAdapter`, retries opening the clipboard, backs up existing rich formats, pastes text, waits for validation, and skips restoration if clipboard was modified externally.
93. **`TC-RWS-T4-05`: Multi-Monitor Scaling & High-Contrast Rapid Dictation**
    - **Description:** User runs a multi-monitor layout (100% and 200% scaling) in High Contrast mode. User triggers dictation in toggle-mode and speaks rapidly using several commands.
    - **Expected Result:** Overlay adapts layout and colors instantly, remains fully legible, processes formatting, and inserts text via UIA value pattern.

---

## 3. Test Architecture and Directory Layout

The Universal Dictation testing suite is structured using the standard .NET 10 xUnit and Coverlet setup.

### Directory Layout

The tests are organized co-located under the `tests/` directory at the project root:

```
/
├── Directory.Build.props
├── Directory.Packages.props
├── global.json
├── UniversalDictation.sln
├── src/
│   └── [Production Class Libraries & Apps]
└── tests/
    ├── Unit/
    │   ├── UniversalDictation.Unit.csproj
    │   ├── Core/
    │   │   ├── StateMachineTests.cs
    │   │   ├── VoiceCommandParserTests.cs
    │   │   └── LoggingRedactionTests.cs
    │   ├── Audio/
    │   │   └── WasapiCaptureTests.cs
    │   └── Targeting/
    │       ├── HotkeyServiceTests.cs
    │       └── TargetContextTests.cs
    ├── Integration/
    │   ├── UniversalDictation.Integration.csproj
    │   ├── Audio/
    │   │   └── AudioPipelineTests.cs
    │   ├── Transcription/
    │   │   ├── DeepgramAdapterTests.cs
    │   │   └── AzureAdapterTests.cs
    │   └── Insertion/
    │       └── InsertionAdapterChainTests.cs
    ├── Contract/
    │   ├── UniversalDictation.Contract.csproj
    │   └── Fixtures/
    │       ├── deepgram_response.json
    │       └── azure_response.json
    └── E2E/
        ├── UniversalDictation.E2E.csproj
        ├── Framework/
        │   ├── SimulatedAudioCapture.cs
        │   ├── SimulatedHotkeyService.cs
        │   └── MockUiaWindow.cs
        ├── T1_FeatureCoverage/
        ├── T2_BoundaryCases/
        ├── T3_Combinations/
        └── T4_RealWorldScenarios/
```

### Test Execution Commands

To execute tests via the command line, use the following `dotnet test` commands from the project root:

- **Run all tests in the solution:**
  ```bash
  dotnet test UniversalDictation.sln --configuration Release
  ```

- **Run E2E test project only:**
  ```bash
  dotnet test tests/E2E/UniversalDictation.E2E.csproj --configuration Release
  ```

- **Run tests with coverage tracking (co-located XML formats):**
  ```bash
  dotnet test UniversalDictation.sln --configuration Release /p:CollectCoverage=true /p:CoverletOutputFormat=cobertura
  ```

---

## 4. Real-World Application Scenarios (Detailed Workflow)

This section provides the simulation and assertion logic details for the 5 Tier 4 E2E scenarios.

### Scenario 1: Web-based EMR Clinical Dictation
- **Simulation Setup:** Spin up a simulated Chrome browser process containing a text area element. The extension port is mocked.
- **Action Sequence:**
  1. Dispatch hotkey press for push-to-talk.
  2. Stream audio samples representing "Patient reports headaches comma new line starting ibuprofen period".
  3. Dispatch hotkey release.
- **Verification Assertions:**
  - State machine correctly flows: `Idle` -> `Arming` -> `Capturing` -> `Streaming` -> `Finalizing` -> `ReadyToInsert` -> `ValidatingTarget` -> `Inserting` -> `Verifying` -> `Completed` -> `Idle`.
  - The text inserted into Chrome matches: `Patient reports headaches, \nstarting ibuprofen. `
  - The Browser Extension adapter is documented as the chosen injection route.

### Scenario 2: Remote Desktop (RDP) Dictation with Network Failover
- **Simulation Setup:** Launch a mock target application representing a remote desktop terminal. Simulate WebSocket connection drop immediately after starting stream.
- **Action Sequence:**
  1. Trigger hotkey toggle to start.
  2. Feed raw audio stream representing patient data.
  3. Terminate internet connection in mock socket.
  4. Trigger hotkey toggle to stop.
- **Verification Assertions:**
  - Deepgram adapter throws socket exception; system falls back to `Offline` state.
  - Whisper offline transcription processes the local buffer.
  - Injection routing skips UIA and Browser, selecting the `SendInputAdapter`.
  - Normalised text is output character-by-character into the mock RDP terminal.

### Scenario 3: Focus Hijacking and Context Switch
- **Simulation Setup:** Open target Text Editor window. Focus a simulated Password Text Box (Sensitivity: Password) mid-dictation.
- **Action Sequence:**
  1. Hold hotkey. Dictation begins into Text Editor.
  2. Change system active focus to the Password Box.
  3. Release hotkey.
- **Verification Assertions:**
  - Revalidation stage detects active control class/type changed to password field.
  - Validation fails, throwing `TargetValidationException` / `SensitiveFieldBlockedException`.
  - Insertion into the password box is completely blocked.
  - State machine transitions to `FatalFailure`.
  - Serilog file log output contains no trace of the spoken text.

### Scenario 4: Clipboard Fallback with Lock Mismatch
- **Simulation Setup:** Target application blocks standard UIA and API injection. Clipboard contains active formatted RTF data. Mock system clipboard locking mechanism to fail.
- **Action Sequence:**
  1. Dictate a line of text.
  2. Trigger text insertion.
  3. Simulate Clipboard Lock conflict.
- **Verification Assertions:**
  - Insertion chain attempts Browser, UIA, and SendInput; all report unavailable.
  - Fallback is selected. Adapter retries up to 3 times to open clipboard.
  - Clipboard data is backed up. Text is pasted using `Ctrl+V`.
  - Simulate external clipboard modification.
  - Adapter aborts clipboard restoration and logs the skip to protect new external clipboard content.

### Scenario 5: High display scaling multi-monitor and rapid commands
- **Simulation Setup:** Initialize WPF Overlay UI. Simulate system scaling configuration at 200%.
- **Action Sequence:**
  1. Toggle dictation hotkey.
  2. Input rapid commands: "scratch that", "hello", "comma", "world", "full stop", "stop dictation".
- **Verification Assertions:**
  - Overlay UI rendering measurements do not exceed boundary sizes.
  - Reconciler parses commands, producing "hello, world. "
  - State machine finalizes cleanly, and text is successfully injected.

---

## 5. Coverage Thresholds and Quality Gates

To maintain high reliability, the project enforces strict coverage targets and quality gates before any commit can be merged.

### Coverage Thresholds

| Scope / Project | Line Coverage Target | Branch Coverage Target |
|---|---|---|
| `Desktop.Core` (State Machine) | 95% | 90% |
| `Desktop.Core` (All code) | 90% | 85% |
| `Desktop.Audio` | 80% | 75% |
| `Desktop.Transcription` | 85% | 80% |
| `Desktop.Targeting` | 80% | 75% |
| `Desktop.Insertion` | 85% | 80% |
| `DesktopApp` (WPF UI logic) | 70% | 60% |
| **Overall Solution** | **85%** | **80%** |

### Quality Gates

1. **Zero Warnings:** The solution must build in `Release` configuration with zero compiler warnings and zero C# code analyzer (CA) warnings.
2. **Format Verification:** All source code files must adhere to the style defined in `.editorconfig`. Running `dotnet format --verify-no-changes` must exit with code 0.
3. **Execution Success:** All 93 test cases defined within this infrastructure must be registered. No tests may be skipped without explicit, documented justification.
