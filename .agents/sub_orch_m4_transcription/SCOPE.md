# Scope: Milestone 4: Transcription

## Architecture
- Module/package boundaries, data flow, shared interfaces
  - Deepgram, Azure, Whisper.net adapters in `Desktop.Transcription` project.
  - Core types and reconciler in `Desktop.Core` project.
  - Tests in `tests/Contract` and `tests/Unit`.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Explore | Codebase exploration and planning | None | DONE |
| 2 | Deepgram | Nova-3 adapter (R6a) | M1 | IN_PROGRESS |
| 3 | Azure | Azure Speech adapter (R6b) | M1 | PLANNED |
| 4 | Whisper | Whisper.net offline adapter (R6c) | M1 | PLANNED |
| 5 | Reconciler | TranscriptReconciler implementation (R6) | M2, M3, M4 | PLANNED |
| 6 | Verification | Unit/contract tests, Release build check | M5 | PLANNED |

## Interface Contracts
- `IStreamingTranscriptionProvider`: ConnectAsync, SendAudioAsync, DisconnectAsync, SegmentReceived, ErrorOccurred.
- `IOfflineTranscriptionProvider`: TranscribeAsync.
