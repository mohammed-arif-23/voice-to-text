## 2026-06-30T11:10:09Z
You are the remediation worker subagent for Milestone 1: Foundations.
Your working directory is `/Users/mohammedarif/voice-to-text/.agents/worker_m1_remediation`.

Objectives:
1. Move the types, exceptions, and interfaces implemented in `tests/E2E/Stubs.cs` to their proper production projects. Specifically:
   - Move all interfaces (`IStreamingTranscriptionProvider`, `IOfflineTranscriptionProvider`, `IAudioCaptureService`, `IAudioProcessingPipeline`, `ITargetContextProvider`, `ITextInsertionAdapter`, `IBrowserBridge`, `IClipboardService`, `IHotkeyService`, `IEntitlementService`, `IUsageMeter`, `IUpdateService`, `ITelemetrySink`) to `src/Desktop.Core/Interfaces/`.
   - Move all exceptions (`DictationException`, `InvalidSessionTransitionException`, `TargetValidationException`, `SensitiveFieldBlockedException`, `ProviderAuthException`, `AudioCaptureException`, `InsertionFailedException`) to `src/Desktop.Core/Exceptions/`.
   - Move all domain types (`TargetContext`, `TranscriptSegment`, `SessionId`, `AudioFrame`, `InsertionResult`, `DictationSessionOptions`, `SensitivityClassification`, `AdapterKind`, `SegmentKind`) to `src/Desktop.Core/Domain/`.
   Ensure that these types use the namespace `Desktop.Core`.
2. Inspect `tests/E2E/Stubs.cs`. Clean it up by removing all the stubs you moved.
3. Make sure the E2E test project (`tests/E2E/UniversalDictation.E2E.csproj`) references the production project `src/Desktop.Core/Desktop.Core.csproj` so it compiles successfully.
4. Update `Directory.Packages.props` to upgrade `OpenTelemetry` and `OpenTelemetry.Api` package versions to `1.11.2` or later to resolve the `NU1902` NuGet vulnerability warnings.
5. Resolve all formatting violations across the C#, XAML, XAML.cs, and JSON files by running `dotnet format` on the solution, so that `dotnet format --verify-no-changes UniversalDictation.sln` passes cleanly.
6. Verify your work by running:
   - `dotnet build --configuration Release UniversalDictation.sln`
   - `dotnet format --verify-no-changes UniversalDictation.sln`
   Ensure there are zero warnings and zero errors in the build and formatting checks.
7. Create a handoff report at `/Users/mohammedarif/voice-to-text/.agents/worker_m1_remediation/handoff.md` detailing the actions taken, build results, and layout verification.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Send a message back to the parent once done.
