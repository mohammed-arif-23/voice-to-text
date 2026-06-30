# Implementation Plan - Milestone 1: Foundations

## Phase 1: Solution & Project Layout Creation
1. **Create Solution**: Run `dotnet new sln -n UniversalDictation` at `/Users/mohammedarif/voice-to-text`.
2. **Create Projects**:
   - `src/DesktopApp` (WPF app)
   - `src/Desktop.Core` (Class Library)
   - `src/Desktop.Audio` (Class Library)
   - `src/Desktop.Transcription` (Class Library)
   - `src/Desktop.Targeting` (Class Library)
   - `src/Desktop.Insertion` (Class Library)
   - `src/Desktop.NativeInterop` (Class Library)
   - `src/NativeMessagingHost` (Console App)
   - `src/ControlPlane.Domain` (Class Library, net10.0)
   - `src/ControlPlane.Application` (Class Library, net10.0)
   - `src/ControlPlane.Infrastructure` (Class Library, net10.0)
   - `src/ControlPlane.Api` (Web API, net10.0)
   - `src/AdminPortal` (Web / Razor Pages App, net10.0)
   - `tests/Unit` (xUnit test project)
   - `tests/Integration` (xUnit test project)
   - `tests/Contract` (xUnit test project)
3. **Add Projects to Solution**: Run `dotnet sln UniversalDictation.sln add <project-path>` for all 16 projects.

## Phase 2: Project Dependency & Package Alignment
1. **Add Project References**:
   - Add reference relationships as defined in Section 5 of the explorer report.
2. **Add Package References**:
   - Add the specified packages to each project. Ensure CPM is respected (no `Version` attribute inside the project files).
3. **Validate Initial Build**:
   - Run a quick test build of the empty projects to ensure structure is correct.

## Phase 3: Configuration Files Setup
1. **Create .editorconfig**: Place at the root. Set C# styles (indent = 4, disable var for built-in, sort system directives).
2. **Create .gitignore**: Place at the root. Include all Visual Studio, Rider, OS, and build artifacts.

## Phase 4: Core Domain & Interface Implementations in `Desktop.Core`
1. **Domain Types**:
   - `TargetContext` class.
   - `TranscriptSegment` class.
   - `SegmentKind` enum.
   - `SessionId` readonly struct.
   - `AudioFrame` class.
   - `InsertionResult` class.
   - `DictationSessionOptions` class.
   - `SensitivityClassification` enum.
   - `AdapterKind` enum.
2. **Exception Types**:
   - `DictationException` (base exception)
   - `InvalidSessionTransitionException`
   - `TargetValidationException`
   - `SensitiveFieldBlockedException`
   - `ProviderAuthException`
   - `AudioCaptureException`
   - `InsertionFailedException`
   - Ensure none of the exceptions include user content in their messages.
3. **Interfaces**:
   - `IStreamingTranscriptionProvider`
   - `IOfflineTranscriptionProvider`
   - `IAudioCaptureService`
   - `IAudioProcessingPipeline`
   - `ITargetContextProvider`
   - `ITextInsertionAdapter`
   - `IBrowserBridge`
   - `IClipboardService`
   - `IHotkeyService`
   - `IEntitlementService`
   - `IUsageMeter`
   - `IUpdateService`
   - `ITelemetrySink`

## Phase 5: Verification & Handoff
1. **Verify**: Run `dotnet build --configuration Release UniversalDictation.sln`.
2. **Create Handoff Report**: Write findings, logic chain, and verification details in `handoff.md`.
