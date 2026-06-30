# Original User Request

## Initial Request — 2026-06-30T16:35:17Z

You are the Milestone 3 Sub-Orchestrator. Your working directory is `/Users/mohammedarif/voice-to-text/.agents/sub_orch_m3_audio`.
Your role is to implement Milestone 3: Audio & Hotkeys.

Objectives:
1. Implement `WasapiAudioCaptureService` in `Desktop.Audio` (R5) using NAudio, resampling to 16kHz Mono PCM16, ring buffering, and device arrival/removal notifications.
2. Implement `GlobalHotkeyService` in `Desktop.Targeting` (R8) using RegisterHotKey/UnregisterHotKey P/Invoke, supporting push-to-talk and toggle modes.
3. Implement `TargetContextService` in `Desktop.Targeting` (R10) capturing focused control metadata, ProcessIntegrityLevel, and performing revalidation before insertion.
4. Verify your work by writing unit/integration tests in `tests/Unit` and `tests/Integration` covering ring buffer overflow, hot-plug notifications, hotkey conflict exception, and target context revalidation.
5. Compile and test Release configuration, ensuring zero warnings.
6. Track progress in `progress.md` and write a final `handoff.md`.

You are a pure orchestrator: do not write code directly. Spawn explorers, workers, and reviewers, and use the iteration loop to implement and verify this milestone.
Your parent conversation ID is: 7d2ee2cb-6f12-44f0-862d-b6c2f020f27f. Send updates and reports back to your parent using send_message.
