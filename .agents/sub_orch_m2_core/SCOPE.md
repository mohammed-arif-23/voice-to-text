# Scope: Milestone 2 — Core Logic

## Architecture
- **Desktop.Core**: Contains the Core business logic for the Voice-to-Text client.
  - State Machine (`DictationSessionStateMachine`): Implements 16 states, controls legal transitions thread-safely, records state transitions history, and raises change events.
  - Exceptions: Core exception types including `InvalidSessionTransitionException`, `DictationException`.
  - Voice Command Parser (`VoiceCommandParser` and `TranscriptReconciler`): Locale-aware, deterministic text replacement and control command detection.
  - Logging Redaction Pipeline (`LoggingRedactionPipeline`, enricher, and sink): Redacts sensitive attributes before writing to the rolling log files.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M2.1 | Dictation Session State Machine | Define `DictationState` enum, implement thread-safe `DictationSessionStateMachine` and its exceptions in `Desktop.Core` | none | PLANNED |
| M2.2 | Logging Redaction Pipeline | Implement `LoggingRedactionPipeline`, Serilog properties enricher/pipeline, and rolling file sink configurations | none | PLANNED |
| M2.3 | Voice Command Parser | Implement deterministic locale-aware `VoiceCommandParser` for 16 commands and `TranscriptReconciler` | none | PLANNED |
| M2.4 | Unit Tests Verification | Write xUnit tests under `tests/Unit` verifying all state transitions, redactions, and parsing rules | M2.1, M2.2, M2.3 | PLANNED |
| M2.5 | Compile & Test Release | Verify project builds with zero warnings/errors in Release configuration and all unit/E2E tests pass | M2.4 | PLANNED |

## Interface Contracts
### State Machine ↔ Application Core
- `DictationSessionStateMachine` class:
  - `DictationState CurrentState { get; }`
  - `IReadOnlyList<SessionTransitionRecord> TransitionHistory { get; }`
  - `event Action<SessionTransitionRecord>? StateChanged`
  - `void TransitionTo(DictationState nextState)`
- `SessionTransitionRecord` record:
  - `DictationState FromState`
  - `DictationState ToState`
  - `DateTime Timestamp`

### Logging Redaction ↔ Serilog Pipeline
- `LoggingRedactionPipeline` class:
  - `static string RedactValue(string propertyName, string value)`
  - `static object RedactObject(object obj)`
- Serilog Enricher/Destructuring Policy for automatic redaction of properties like `Transcript`, `ClipboardContent`, `ApiToken`, `WindowTitle`, `FilePath`, `StackTrace`.

### Voice Command Parser ↔ Transcription Reconciler
- `VoiceCommandParser` class:
  - `string Parse(string input, out List<string> controlSignals, string locale = "en-US")`
- `TranscriptReconciler` class:
  - `IReadOnlyList<TranscriptSegment> StableSegments { get; }`
  - `string AddSegment(TranscriptSegment segment, string locale = "en-US")`
  - `string ReconcileInterims(List<TranscriptSegment> interims)`
  - `string GetReconciledText()`
  - `void Clear()`
