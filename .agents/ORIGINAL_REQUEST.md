# Original User Request

## Initial Request — 2026-06-30T10:37:31Z

Build a production-grade, commercially shippable universal voice-to-text desktop
application for Windows — a complete dictation layer that captures the user's
focused field, streams speech to a cloud STT provider in real time, and inserts
verified text back into the original Windows or browser application safely and
reliably. This is **not** a prototype or MVP: all implemented code must be
correct, tested, and wired into the runtime.

Working directory: `/Users/mohammedarif/voice-to-text`
Integrity mode: development

---

## Context

Three files have already been created in the working directory:

- `global.json` — pins .NET 10 LTS, rollForward latestFeature
- `Directory.Build.props` — shared build settings (nullable, TreatWarningsAsErrors,
  net10.0-windows, Deterministic, SourceLink)
- `Directory.Packages.props` — central package version management (all NuGet
  versions declared centrally; individual projects must NOT specify Version on
  PackageReference items)

Do not delete or overwrite these three files. Read them before creating any
project files so that your `.csproj` files are consistent.

A git repository is already initialised. Commit work in logical checkpoints
with descriptive messages. Do not force-push.

---

## Technology Decisions (fixed — do not deviate)

- **Runtime:** .NET 10 LTS (`net10.0-windows`)
- **Desktop shell:** WPF, MVVM pattern
- **Audio:** NAudio 3.x `WasapiCapture`; resample to 16 kHz PCM16 via
  `MediaFoundationResampler` before sending to providers
- **Primary STT:** Deepgram Nova-3 streaming over WSS
  (`wss://api.deepgram.com/v1/listen`); parameter name is `keyterm` (singular,
  repeated), NOT `keyterms` or `keywords`
- **Secondary STT:** Azure Cognitive Services Speech SDK
  (`Microsoft.CognitiveServices.Speech` 1.50.0)
- **Offline STT:** Whisper.net 1.9.0 + `Whisper.net.Runtime` (CPU-only;
  target `ggml-small` GGUF model)
- **Logging:** Serilog with file + console sinks
- **DI / config / logging abstractions:** Microsoft.Extensions.*
- **Telemetry:** OpenTelemetry SDK (privacy-filtered, opt-in)
- **Packaging (future):** WiX v6 MSI — do not add MSIX; NativeMessaging host
  registration requires full registry write access

---

## Requirements

### R1. Solution structure and build foundations

Create the full Visual Studio solution (`UniversalDictation.sln`) containing
these projects under `src/`:

| Project | Type |
|---|---|
| `DesktopApp` | WPF Application (net10.0-windows) |
| `Desktop.Core` | Class Library — domain, state machine, interfaces |
| `Desktop.Audio` | Class Library — audio capture and processing |
| `Desktop.Transcription` | Class Library — provider adapters |
| `Desktop.Targeting` | Class Library — hotkeys, target context |
| `Desktop.Insertion` | Class Library — adapter chain, verification |
| `Desktop.NativeInterop` | Class Library — Win32 P/Invoke, DPAPI |
| `NativeMessagingHost` | Console Application |
| `ControlPlane.Api` | ASP.NET Core Web API (net10.0, not windows-specific) |
| `ControlPlane.Application` | Class Library |
| `ControlPlane.Domain` | Class Library |
| `ControlPlane.Infrastructure` | Class Library |
| `AdminPortal` | ASP.NET Core Web App (Razor Pages) |

Test projects under `tests/`:

| Project | Type |
|---|---|
| `Unit` | xUnit test project |
| `Integration` | xUnit test project |
| `Contract` | xUnit test project |

All projects must: use central package management (no `Version=` on
`PackageReference`), target the appropriate framework from `Directory.Build.props`,
compile in Release mode without warnings or analyzer errors.

Add `.editorconfig` enforcing: 4-space indent for C#, 2-space for JSON/YAML,
`csharp_style_var_for_built_in_types = false`, `dotnet_sort_system_directives_first = true`.

Add `.gitignore` appropriate for a .NET / WPF / Node.js / GitHub Actions repository.

### R2. Session state machine

Implement the dictation session state machine in `Desktop.Core`. The machine
must have exactly these states:

```
SignedOut | Idle | Arming | Capturing | Streaming | Finalizing
| ReviewRequired | ReadyToInsert | ValidatingTarget | Inserting
| Verifying | Completed | Cancelled | RecoverableFailure | FatalFailure
| Offline
```

Define all legal transitions explicitly. Any attempt to transition to an
unlisted next-state must throw `InvalidSessionTransitionException` (a typed
domain exception, not a generic one). The machine must record a
`SessionTransitionRecord` (timestamp, from-state, to-state, trigger, no content)
without recording any dictated text. The machine must be thread-safe.

### R3. Core port interfaces and domain types

In `Desktop.Core`, implement all of the following — these are interfaces and
value-object types only; no production implementations in this project:

**Interfaces:**
`IStreamingTranscriptionProvider`, `IOfflineTranscriptionProvider`,
`IAudioCaptureService`, `IAudioProcessingPipeline`, `ITargetContextProvider`,
`ITextInsertionAdapter`, `IBrowserBridge`, `IClipboardService`,
`IHotkeyService`, `IEntitlementService`, `IUsageMeter`, `IUpdateService`,
`ITelemetrySink`

**Domain types:**
`TargetContext` (all fields from the master spec §6), `TranscriptSegment`
(with `SegmentKind`: Interim / Stable / Final / Corrected), `SessionId`
(strongly-typed wrapper over Guid), `AudioFrame` (timestamp + PCM buffer span),
`InsertionResult` (adapter used, verified, failure code if any),
`DictationSessionOptions`, `SensitivityClassification` (Normal / Password /
Secure / Unknown), `AdapterKind` enum.

**Error types:**
`InvalidSessionTransitionException`, `TargetValidationException`,
`SensitiveFieldBlockedException`, `ProviderAuthException`,
`AudioCaptureException`, `InsertionFailedException` — all extending a base
`DictationException` with a `DiagnosticCode` (never containing user content).

### R4. Logging redaction pipeline

Configure Serilog in `Desktop.Core` (and referenced from `DesktopApp`) with:

- A `IDestructuringPolicy` or `LogEventEnricher` that redacts any property
  tagged `[Redacted]` by replacing its value with `"***"`.
- Redacted properties must include: transcript text, window title hash source,
  clipboard content, API tokens, file paths containing user names.
- Rolling file sink: `logs/dictation-.log` (1-day rolls, 7-day retention).
- Minimum level: `Information` in Release, `Debug` in Debug.
- Structured JSON output to file; human-readable to console.

### R5. WASAPI audio capture (`Desktop.Audio`)

Implement `WasapiAudioCaptureService : IAudioCaptureService` using NAudio 3.x
`WasapiCapture`. Requirements:

- Enumerate and expose available microphone devices.
- Support hot-plug: detect device arrival/removal via `MMDeviceEnumerator`
  notifications; emit `AudioDeviceChangedEvent`.
- Capture raw audio and resample to **16 kHz, 16-bit, mono PCM** using
  `MediaFoundationResampler` before publishing frames.
- Buffer audio in a bounded ring buffer (configurable capacity, default 5 s).
  When the ring buffer is full, emit `AudioBufferOverflowEvent` and drop
  incoming frames (never block the capture thread).
- Never write raw audio to disk during normal operation.
- Publish audio via `IAsyncEnumerable<AudioFrame>` with cooperative cancellation.
- Handle: no microphone present, permission denied (access is denied error from
  WASAPI), microphone disconnected mid-capture, exclusive-mode conflict. Each
  failure must emit a typed `AudioCaptureException` with a `DiagnosticCode`.

### R6. Streaming transcription provider adapters (`Desktop.Transcription`)

Implement three adapters behind `IStreamingTranscriptionProvider`:

**a) Deepgram Nova-3 adapter**
- Connect to `wss://api.deepgram.com/v1/listen` with query parameters:
  `model=nova-3`, `encoding=linear16`, `sample_rate=16000`,
  `interim_results=true`, `endpointing=200`, and any `keyterm` entries from
  `DictationSessionOptions` (one query parameter per term).
- Authenticate via short-lived token (retrieved from Control Plane, not
  embedded in the app). Accept the token via constructor/options — the adapter
  must not fetch credentials itself.
- Send 20 ms PCM frames. Send `KeepAlive` JSON during silence.
  Send `CloseStream` JSON on finalize.
- Parse interim and final result JSON; emit `TranscriptSegment` with correct
  `SegmentKind`, confidence, and word timings where provided.
- Do not duplicate words when providers revise interim hypotheses (implement
  stable-segment tracking).
- Handle: WebSocket close, provider error codes (402, 429, 503),
  malformed JSON, auth expiry. Reconnect with exponential backoff (max 3
  attempts, then surface `RecoverableFailure`).

**b) Azure Speech adapter**
- Use `Microsoft.CognitiveServices.Speech` push-stream pattern.
- Accept region + auth token via options (no embedded keys).
- Map `Recognizing` → `Interim`, `Recognized` → `Final` `TranscriptSegment`.
- Handle `Canceled` with `CancellationReason` mapped to typed `DiagnosticCode`.

**c) Whisper.net offline adapter (`IOfflineTranscriptionProvider`)**
- Use `Whisper.net` 1.9.0 with `Whisper.net.Runtime` (CPU only).
- Accept a path to a local GGUF model file via options.
- Process a completed audio buffer (not streaming — offline mode collects the
  full recording then transcribes).
- Return segments with timestamps.
- If the model file does not exist, throw `OfflineModelNotFoundException`
  (typed, with the expected path in the message — no user content).

**Transcript reconciliation:**
Implement `TranscriptReconciler` that merges an ordered stream of
`TranscriptSegment` values into a clean final string, applying the voice
command parser (R7) and deduplicating revised interim segments.

### R7. Voice command parser

Implement `VoiceCommandParser` in `Desktop.Core` with a locale-aware phrase
table (at minimum `en-US`). Recognised commands and their actions:

| Spoken phrase | Action |
|---|---|
| "new line" | Insert `\n` |
| "new paragraph" | Insert `\n\n` |
| "comma" | Insert `, ` |
| "full stop" / "period" | Insert `. ` |
| "question mark" | Insert `? ` |
| "colon" | Insert `: ` |
| "semicolon" | Insert `; ` |
| "open quote" | Insert `"` |
| "close quote" | Insert `"` |
| "bullet point" | Insert `• ` |
| "delete last word" | Remove last word from buffer |
| "delete last sentence" | Remove last sentence from buffer |
| "scratch that" | Clear current segment buffer |
| "undo" | Signal undo event |
| "stop dictation" | Finalize cleanly |
| "cancel dictation" | Cancel without insertion |

The parser must be deterministic. Do not pass commands to any LLM. Commands
must be extracted before the text reaches the insertion path.

### R8. Global hotkeys (`Desktop.Targeting`)

Implement `GlobalHotkeyService : IHotkeyService` using `RegisterHotKey` /
`UnregisterHotKey` Win32 APIs via P/Invoke. Requirements:

- Support configurable push-to-talk (hold) and toggle-mode (press) shortcuts.
- Detect conflicts with existing system hotkeys and registered application
  hotkeys; surface `HotkeyConflictException` with the conflicting VK code.
- Clean up all registered hotkeys on dispose.
- Never block the UI thread waiting for hotkey events.

### R9. No-activate overlay (`DesktopApp`)

Implement a WPF floating overlay window with these constraints:

- Must NOT steal keyboard focus under any circumstances during normal
  dictation.
- Apply `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW` via P/Invoke in
  `OnSourceInitialized`. Handle `WM_MOUSEACTIVATE` returning `MA_NOACTIVATE`
  via an `HwndSource` hook.
- Show the current session state: Idle / Arming / Capturing / Streaming /
  Processing / Error — with a distinct visual indicator for each.
- Show interim transcript text as it arrives (scrolling, truncated to last
  200 characters).
- Position itself near the system clock by default; support user-draggable
  repositioning that persists across sessions.
- Support Windows display scaling: 100%, 125%, 150%, 200%, 300% — all text
  must remain legible and the window must not overflow the screen.
- Support high-contrast mode.

### R10. Target context capture and revalidation (`Desktop.Targeting`)

Implement `TargetContextService : ITargetContextProvider`:

- Capture a `TargetContext` snapshot when the hotkey is pressed, using
  `System.Windows.Automation` (UI Automation) for the focused element.
- Populate all fields defined in `TargetContext`: ProcessId, ProcessExecutable,
  ProcessIntegrityLevel (via `GetTokenInformation` P/Invoke),
  TopLevelWindowHandle, FocusedWindowHandle, AutomationElementRuntimeId,
  AutomationControlType, AutomationId, ClassName, WindowTitleHash (SHA-256 of
  raw title — do not store the raw title), EditableState, ReadOnlyState,
  PasswordOrSensitiveState, CaptureTimestamp.
- Before insertion, re-acquire the active process and focused element and
  compare against the captured snapshot. Reject if ProcessId, RuntimeId,
  SensitivityClassification, or IntegrityLevel changed.
- Classify `SensitivityClassification` as `Password` when
  `AutomationControlType == Edit` and `IsPassword == true`, or when the
  class name matches known password field patterns.
- Emit `TargetValidationException` with a `DiagnosticCode` (no user content)
  when validation fails.

### R11. Text insertion adapter chain (`Desktop.Insertion`)

Implement `InsertionAdapterChain : ITextInsertionAdapter` that tries adapters
in priority order:

1. `BrowserExtensionAdapter` — delegates to `IBrowserBridge`; skip if bridge
   not connected.
2. `UiaValuePatternAdapter` — uses UI Automation `ValuePattern.SetValue`;
   skip if pattern unavailable or control is read-only.
3. `SendInputAdapter` — uses `SendInput` Win32 API with `KEYEVENTF_UNICODE`;
   inserts text character by character; skip if target integrity level is
   higher than current process.
4. `ClipboardFallbackAdapter` — saves complete `IDataObject`, pastes via
   `Ctrl+V` `SendInput`, waits for clipboard change confirmation, then
   restores; skip only if clipboard is permanently locked.

Each adapter must:
- Report which adapter was selected and why higher-priority adapters were skipped.
- Implement post-insertion verification: read back the field value (or a
  reliable signal from the browser adapter) and compare normalised text.
- Emit `InsertionFailedException` if verification fails.
- Never insert into a field where `SensitivityClassification != Normal`.

Clipboard fallback constraints (when used):
- Preserve the full `IDataObject`, not only Unicode text.
- Retry up to 3 times if `OpenClipboard` fails (another process holds it).
- Wait for paste confirmation before restoration.
- Restore previous clipboard only if it was not changed by another process
  during the operation.
- Never log clipboard contents.

---

## Documentation Requirements

### D1. Legacy assessment
Create `docs/legacy-assessment.md` referencing the Rust/Tauri/ScribeRx
prototype available in git history. Include: reusable concepts, and at minimum
these explicitly rejected patterns (with rationale): blind clipboard-only
insertion, mock EMR context, broad fuzzy word replacement, hardcoded paths,
raw transcript logging, predictable temporary WAV files, direct insertion
without target validation.

### D2. Architecture Decision Records
Create `docs/adr/` with one file per ADR for all 14 decisions from the
approved plan. Use the standard ADR format: Status / Context / Decision /
Consequences.

### D3. STATUS.md
Create `docs/STATUS.md` tracking every R1–R11 + D1–D3 component using the
allowed states: `Not started | Designed | Implemented but unverified | Verified`.
Never mark a component Verified unless unit tests pass against it.

---

## Acceptance Criteria

### Build quality
- [ ] `dotnet build --configuration Release UniversalDictation.sln` exits 0
- [ ] `dotnet format --verify-no-changes UniversalDictation.sln` exits 0
- [ ] Zero compiler warnings or CA analyzer warnings in Release output
- [ ] All projects reference packages without explicit `Version=` attributes
      (central management enforced)

### State machine (R2)
- [ ] Unit tests cover all 16 states
- [ ] Unit tests assert every defined legal transition succeeds
- [ ] Unit tests assert every explicitly illegal transition throws
      `InvalidSessionTransitionException`
- [ ] ≥ 95% line coverage on `Desktop.Core` state machine files

### Logging redaction (R4)
- [ ] Unit test: log a message containing a simulated transcript string → assert
      the string is absent from the captured Serilog sink output
- [ ] Unit test: log a message containing a simulated API token → assert absent
- [ ] Unit test: log a message containing a simulated window title → assert absent
- [ ] Tests pass in Release configuration

### Audio capture (R5)
- [ ] Unit tests cover: buffer overflow behaviour (ring buffer drops frames,
      emits event); device-not-found path; permission-denied path; mid-capture
      disconnect
- [ ] Integration test (using a loopback or silence device where available)
      demonstrates `IAsyncEnumerable<AudioFrame>` produces frames

### Provider adapters (R6)
- [ ] Contract tests using recorded Deepgram fixture JSON: interim segments
      parsed correctly; final segments parsed; duplicate interim revisions
      deduplicated; auth error mapped to `ProviderAuthException`
- [ ] Contract tests using recorded Azure fixture: `Recognizing` → Interim,
      `Recognized` → Final, `Canceled` → typed exception
- [ ] Whisper.net unit test: missing model file → `OfflineModelNotFoundException`

### Voice commands (R7)
- [ ] Unit tests for all 16 command phrases produce the correct action
- [ ] Unit test: non-command text passes through unmodified
- [ ] Unit test: command adjacent to dictated text is correctly separated

### Hotkeys (R8)
- [ ] Unit test: registering the same hotkey twice throws `HotkeyConflictException`
- [ ] Unit test: dispose unregisters all hotkeys (verify via Win32 mock)

### Overlay (R9)
- [ ] `WS_EX_NOACTIVATE` is applied — verified by automated focus test:
      overlay is shown → a text editor retains focus → typing continues into
      editor (documented test procedure in `tests/WindowsE2E/`)
- [ ] Overlay renders correctly at 100% and 200% DPI (screenshots in
      `docs/release/overlay-scaling.md`)

### Target context and validation (R10)
- [ ] Unit test: password-field classification returns `SensitivityClassification.Password`
- [ ] Unit test: process changed between capture and revalidation →
      `TargetValidationException`
- [ ] Unit test: integrity level mismatch → `TargetValidationException`

### Insertion adapter chain (R11)
- [ ] Unit test: UIA adapter unavailable → chain falls through to SendInput
- [ ] Unit test: field `SensitivityClassification != Normal` → all adapters
      refuse and `SensitiveFieldBlockedException` is thrown
- [ ] Unit test: clipboard fallback preserves full `IDataObject` (not just text)
- [ ] Unit test: clipboard restoration skipped when clipboard was modified
      during operation

### Documentation
- [ ] `docs/legacy-assessment.md` exists and lists ≥ 5 rejected patterns with
      rationale
- [ ] All 14 ADR files present in `docs/adr/` with Status / Context / Decision
      / Consequences sections
- [ ] `docs/STATUS.md` present; all R1–R11 + D1–D3 entries have honest states

---

## Absolute prohibitions

Do not ship any of the following in any file:
- Hardcoded API keys, tokens, credentials, passwords, or machine-specific paths
- Production composition roots that select mock or stub implementations
- Blind clipboard-only insertion as the primary insertion method
- Audio written to disk during dictation
- Dictated text, window titles, or clipboard contents in log output
- `TreatWarningsAsErrors` disabled or `#pragma warning disable` without a
  documented reason
- Any `TODO`, `FIXME`, `HACK`, or placeholder comments that represent
  unimplemented behaviour — if something is not implemented, it must throw
  `NotImplementedException` with a `DiagnosticCode`, not silently no-op

## Follow-up — 2026-06-30T11:04:22Z

Command update: The user has requested to expedite the implementation of all remaining milestones (Milestone 1 through Milestone 7) immediately. Please invoke your subagents in parallel where possible, resolve task dependencies efficiently, and push the full implementation (core interfaces, state machine, audio, transcription, hotkeys, UI overlay, text insertion, and tests) through to completion as quickly as possible.
