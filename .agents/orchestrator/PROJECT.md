# Project: Universal Dictation

## Architecture
The Universal Dictation system is a high-reliability Windows dictation utility written in .NET 10 (WPF) with central package management. It consists of:
- **DesktopApp**: WPF UI, overlays, and composition root.
- **Desktop.Core**: Central domain models, state machine, ports, logging redaction, and voice command parsing.
- **Desktop.Audio**: WASAPI audio capture, resampler, and ring buffering.
- **Desktop.Transcription**: Deepgram Nova-3, Azure, and Whisper.net adapters and transcript reconciliation.
- **Desktop.Targeting**: Win32 Hotkeys and UI Automation context capture/revalidation.
- **Desktop.Insertion**: Safe insertion chain with post-insertion validation and clipboard fallback.
- **Desktop.NativeInterop**: P/Invokes, DPAPI, and native helpers.
- **NativeMessagingHost**: Browser extension bridge.
- **ControlPlane.* & AdminPortal**: Management API and user portal.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Foundations | Solution, project structure, core interfaces & types (R1, R3) | None | IN_PROGRESS (1377cf59-c689-4d72-87a4-9bd99775b9a4) |
| 2 | Core Logic | State machine, Serilog redaction, voice command parser (R2, R4, R7) | M1 | PLANNED (429 Rate Limited) |
| 3 | Audio & Hotkeys | WASAPI capture, global hotkeys, target context capture (R5, R8, R10) | M1, M2 | PLANNED (429 Rate Limited) |
| 4 | Transcription | Deepgram, Azure, Whisper.net adapters & reconciler (R6) | M1, M2 | PLANNED (429 Rate Limited) |
| 5 | Safe Insertion | Insertion adapter chain, validation, clipboard fallback (R11) | M1, M2, M3 | PLANNED |
| 6 | WPF Desktop UI | Floating overlay (WS_EX_NOACTIVATE), positioning, composition (R9) | All previous | PLANNED |
| 7 | Integration & Hardening | Final E2E test verification, adversarial hardening (Tier 1-5) | All previous, TEST_READY.md | PLANNED |

## Interface Contracts
### IAudioCaptureService ↔ Desktop.Audio
- Exposes microphone enumeration, hot-plug notifications, and frame streaming via `IAsyncEnumerable<AudioFrame>`.
### IStreamingTranscriptionProvider ↔ Desktop.Transcription
- Handles streaming connection to Deepgram / Azure, processing audio frames, emitting `TranscriptSegment`s.
### IOfflineTranscriptionProvider ↔ Desktop.Transcription
- Processes full audio buffer offline via Whisper.net, returns timestamps and segments.
### ITargetContextProvider ↔ Desktop.Targeting
- Captures active focus window context snapshot and performs revalidation before insertion.
### ITextInsertionAdapter ↔ Desktop.Insertion
- Orchestrates insertion attempts using UIA, SendInput, BrowserBridge, or Clipboard fallback with verification.

## Code Layout
- `src/DesktopApp/` - WPF UI
- `src/Desktop.Core/` - Core logic, state machine
- `src/Desktop.Audio/` - Audio capture and resampling
- `src/Desktop.Transcription/` - Provider adapters
- `src/Desktop.Targeting/` - Hotkeys, TargetContext
- `src/Desktop.Insertion/` - Insertion chain
- `src/Desktop.NativeInterop/` - Win32 interop
- `src/NativeMessagingHost/` - Console bridge
- `src/ControlPlane.Api/`, `src/ControlPlane.Application/`, `src/ControlPlane.Domain/`, `src/ControlPlane.Infrastructure/` - Web API and application logic
- `src/AdminPortal/` - Razor Pages portal
- `tests/Unit/` - Unit tests
- `tests/Integration/` - Integration tests
- `tests/Contract/` - Contract tests
