# Scope: E2E Testing Track

## Architecture
The E2E test suite acts as an opaque-box validator for the Universal Dictation application. It targets the public boundaries:
- Hotkey interface (Simulated global hotkeys and Win32 key events)
- Audio capture (Simulated WASAPI capture loopback)
- Transcription API (Simulated WebSockets for Deepgram and Mock Azure SDK push-stream, and Whisper offline runner)
- Target application window state (Simulated UI Automation controls, Browser Bridge, Clipboard)
- WPF Overlay window focus behaviour (Verification of WS_EX_NOACTIVATE focus-preservation)

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Test Infra Setup | Create tests/UniversalDictation.E2E/ project, reference core assemblies, design mock/simulation harnesses | None | PLANNED |
| 2 | Tier 1: Feature Coverage | Implement >=40 tests for standard execution paths across all 8 features | M1 | PLANNED |
| 3 | Tier 2: Boundary & Corner Cases | Implement >=40 tests for error handling, limits, validation exceptions, and timeout paths | M2 | PLANNED |
| 4 | Tier 3: Combinations | Implement >=8 tests for pairwise combinations of major features | M3 | PLANNED |
| 5 | Tier 4: Real-World Scenarios | Implement >=5 full end-to-end user workflow simulations | M4 | PLANNED |
| 6 | Verification & Sign-off | Run tests to verify compilation and baseline execution, publish TEST_READY.md | M5 | PLANNED |

## Interface Contracts
### E2E Test Suite ↔ Universal Dictation
- The test suite interacts with the application via its public interfaces, DI container configuration, or custom testing hooks designed to allow headless simulation (e.g. injecting virtual WASAPI audio streams or virtual UI automation targets).
- Test runner: `dotnet test` targeting the E2E project.
