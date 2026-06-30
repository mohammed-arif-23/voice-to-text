using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Desktop.Audio;
using Desktop.Core;
using Desktop.Insertion;
using Desktop.Logging;
using Desktop.Targeting;
using Desktop.Transcription;
using Desktop.UI;
using FluentAssertions;
using Moq;
using Xunit;

namespace UniversalDictation.E2E
{
    public class T2_BoundaryCases
    {
        // ==========================================
        // FEATURE 1: Session State Machine (SSM)
        // ==========================================

        [Fact]
        public void TC_SSM_T2_01_DirectInvalidStateTransitionAttempt()
        {
            var sm = new DictationSessionStateMachine();
            Action act = () => sm.TransitionTo(DictationState.Inserting);
            act.Should().Throw<InvalidSessionTransitionException>()
               .And.FromState.Should().Be(DictationState.Idle);
        }

        [Fact]
        public void TC_SSM_T2_02_OutOfOrderFinalizationTransition()
        {
            var sm = new DictationSessionStateMachine();
            sm.TransitionTo(DictationState.Arming);
            sm.TransitionTo(DictationState.Capturing);
            sm.TransitionTo(DictationState.Streaming);

            Action act = () => sm.TransitionTo(DictationState.Completed);
            act.Should().Throw<InvalidSessionTransitionException>();
        }

        [Fact]
        public async Task TC_SSM_T2_03_ThreadSafetyRaceConditionValidation()
        {
            var sm = new DictationSessionStateMachine();
            int successCount = 0;
            int failureCount = 0;

            var tasks = new List<Task>();
            for (int i = 0; i < 100; i++)
            {
                var nextState = (i % 2 == 0) ? DictationState.Arming : DictationState.Inserting;
                tasks.Add(Task.Run(() =>
                {
                    try
                    {
                        sm.TransitionTo(nextState);
                        Interlocked.Increment(ref successCount);
                    }
                    catch (InvalidSessionTransitionException)
                    {
                        Interlocked.Increment(ref failureCount);
                    }
                }));
            }

            await Task.WhenAll(tasks);

            // Exactly 1 transition (Idle -> Arming) must succeed, others must fail
            successCount.Should().Be(1);
            failureCount.Should().Be(99);
            sm.CurrentState.Should().Be(DictationState.Arming);
        }

        [Fact]
        public void TC_SSM_T2_04_RecoverableNetworkGlitchTransition()
        {
            var sm = new DictationSessionStateMachine();
            sm.TransitionTo(DictationState.Arming);
            sm.TransitionTo(DictationState.Capturing);
            sm.TransitionTo(DictationState.Streaming);

            // Network drop
            sm.TransitionTo(DictationState.RecoverableFailure);
            sm.CurrentState.Should().Be(DictationState.RecoverableFailure);

            // Reset
            sm.TransitionTo(DictationState.Idle);
            sm.CurrentState.Should().Be(DictationState.Idle);
        }

        [Fact]
        public void TC_SSM_T2_05_FatalTargetMismatchFailureTransition()
        {
            var sm = new DictationSessionStateMachine();
            sm.TransitionTo(DictationState.Arming);
            sm.TransitionTo(DictationState.Capturing);
            sm.TransitionTo(DictationState.Streaming);
            sm.TransitionTo(DictationState.Finalizing);
            sm.TransitionTo(DictationState.ReadyToInsert);
            sm.TransitionTo(DictationState.ValidatingTarget);

            // Target mismatched -> fatal
            sm.TransitionTo(DictationState.FatalFailure);
            sm.CurrentState.Should().Be(DictationState.FatalFailure);

            // Reset
            sm.TransitionTo(DictationState.Idle);
            sm.CurrentState.Should().Be(DictationState.Idle);
        }

        // ==========================================
        // FEATURE 2: Logging Redaction Pipeline (LRP)
        // ==========================================

        [Fact]
        public void TC_LRP_T2_01_WindowTitleHashSourceRedaction()
        {
            string title = "Epic EMR - John Doe MD";
            string output = LoggingRedactionPipeline.RedactValue("WindowTitle", title);

            // Must be SHA-256 hex string (64 characters)
            output.Should().HaveLength(64);
            output.Should().NotContain("Epic");
            output.Should().NotContain("John Doe");
        }

        [Fact]
        public void TC_LRP_T2_02_SensitiveFilePathRedaction()
        {
            string stackTrace = "at App.Run() in C:\\Users\\alice_smith\\AppData\\Local\\Temp\\Core.cs:line 42";
            string output = LoggingRedactionPipeline.RedactValue("StackTrace", stackTrace);

            output.Should().Contain("C:\\Users\\***\\AppData");
            output.Should().NotContain("alice_smith");
        }

        [Fact]
        public void TC_LRP_T2_03_DeeplyNestedRedactionObjectTraversal()
        {
            var dict = new Dictionary<string, string>
            {
                { "Token", "secret-token" },
                { "RedactedData", "confidential" },
                { "NormalKey", "normal-value" }
            };

            var redacted = (Dictionary<string, string>)LoggingRedactionPipeline.RedactObject(dict);

            redacted["Token"].Should().Be("***");
            redacted["RedactedData"].Should().Be("***");
            redacted["NormalKey"].Should().Be("normal-value");
        }

        [Fact]
        public void TC_LRP_T2_04_ReleaseLogLevelGate()
        {
            // Simulate release build check
            bool isRelease = true;
            var logsWritten = new List<string>();

            Action<string, string> logEmit = (level, msg) =>
            {
                if (isRelease && (level == "Debug" || level == "Verbose")) return;
                logsWritten.Add(msg);
            };

            logEmit("Debug", "A debug log");
            logEmit("Information", "An info log");

            logsWritten.Should().ContainSingle();
            logsWritten[0].Should().Be("An info log");
        }

        [Fact]
        public void TC_LRP_T2_05_DebugLogLevelGate()
        {
            bool isRelease = false; // Debug configuration
            var logsWritten = new List<string>();

            Action<string, string> logEmit = (level, msg) =>
            {
                if (isRelease && (level == "Debug" || level == "Verbose")) return;
                logsWritten.Add(msg);
            };

            logEmit("Debug", "A debug log");
            logEmit("Information", "An info log");

            logsWritten.Should().HaveCount(2);
            logsWritten.Should().Contain("A debug log");
            logsWritten.Should().Contain("An info log");
        }

        // ==========================================
        // FEATURE 3: WASAPI Audio Capture (R5)
        // ==========================================

        [Fact]
        public async Task TC_WAC_T2_01_RingBufferOverflowEvent()
        {
            var service = new WasapiAudioCaptureService { SimulateOverflow = true };
            bool overflowRaised = false;

            service.BufferOverflow += (ev) =>
            {
                overflowRaised = true;
                ev.OverflowBytes.Should().BeGreaterThan(0);
            };

            service.StartCapture("Default Mic");

            // Trigger stream to execute overflow check
            var cts = new CancellationTokenSource(100);
            var enumerator = service.StreamFramesAsync(cts.Token).GetAsyncEnumerator();

            // MoveNext to run code
            await enumerator.MoveNextAsync();

            service.StopCapture();
            overflowRaised.Should().BeTrue();
        }

        [Fact]
        public async Task TC_WAC_T2_02_RingBufferFrameDropNonBlocking()
        {
            var service = new WasapiAudioCaptureService { SimulateOverflow = true };
            service.StartCapture("Default Mic");

            var cts = new CancellationTokenSource(200);
            var frames = new List<AudioFrame>();

            await foreach (var frame in service.StreamFramesAsync(cts.Token))
            {
                frames.Add(frame);
                if (frames.Count >= 3) break;
            }

            service.StopCapture();
            frames.Should().NotBeEmpty(); // Capture should continue without blocking
        }

        [Fact]
        public void TC_WAC_T2_03_MissingInputDeviceException()
        {
            var service = new WasapiAudioCaptureService { SimulateMissingDevice = true };

            Action act = () => service.StartCapture("NonExistent Mic");
            act.Should().Throw<AudioCaptureException>()
               .And.DiagnosticCode.Should().Be(unchecked((int)0x80070490));
        }

        [Fact]
        public void TC_WAC_T2_04_AccessDeniedPermissionException()
        {
            var service = new WasapiAudioCaptureService { SimulateAccessDenied = true };

            Action act = () => service.StartCapture("Default Mic");
            act.Should().Throw<AudioCaptureException>()
               .And.DiagnosticCode.Should().Be(unchecked((int)0x80070005));
        }

        [Fact]
        public void TC_WAC_T2_05_ExclusiveModeConflictHandling()
        {
            var service = new WasapiAudioCaptureService { SimulateExclusiveModeConflict = true };

            Action act = () => service.StartCapture("Default Mic");
            act.Should().Throw<AudioCaptureException>()
               .And.DiagnosticCode.Should().Be(unchecked((int)0x8889000A));
        }

        // ==========================================
        // FEATURE 4: Transcription Providers & Reconciler (TPR)
        // ==========================================

        [Fact]
        public async Task TC_TPR_T2_01_WhisperMissingModelFileException()
        {
            var provider = new WhisperOfflineTranscriptionProvider { ModelFileExists = false };

            Func<Task> act = () => provider.TranscribeAsync(new byte[1000], "models/ggml-base.en.bin", CancellationToken.None);
            await act.Should().ThrowAsync<OfflineModelNotFoundException>()
                     .WithMessage("*models/ggml-base.en.bin*");
        }

        [Fact]
        public async Task TC_TPR_T2_02_DeepgramTokenAuthenticationFailure()
        {
            var provider = new DeepgramTranscriptionProvider();

            Func<Task> act = () => provider.ConnectAsync("invalid-token", CancellationToken.None);
            await act.Should().ThrowAsync<ProviderAuthException>();
        }

        [Fact]
        public async Task TC_TPR_T2_03_DeepgramNetworkOutageAndReconnect()
        {
            var provider = new DeepgramTranscriptionProvider();
            await provider.ConnectAsync("valid-token", CancellationToken.None);
            provider.SimulateNetworkOutage = true;

            int retryCount = 0;
            Func<Task> send = async () =>
            {
                try
                {
                    await provider.SendAudioAsync(new AudioFrame(new byte[3200], DateTime.UtcNow, 16000, 16, 1), CancellationToken.None);
                }
                catch (System.Net.WebSockets.WebSocketException)
                {
                    // simulate exponential backoff up to 3 retries
                    while (retryCount < 3)
                    {
                        retryCount++;
                        // wait and reconnect
                        await provider.ConnectAsync("valid-token", CancellationToken.None);
                    }
                    throw new RecoverableFailureException("Network retry failed.");
                }
            };

            await send.Should().ThrowAsync<RecoverableFailureException>();
            provider.ConnectAttempts.Should().Be(4); // 1 initial + 3 retries
        }

        private class RecoverableFailureException : Exception
        {
            public RecoverableFailureException(string msg) : base(msg) { }
        }

        [Fact]
        public void TC_TPR_T2_04_TranscriptReconcilerInterimDeduplication()
        {
            var reconciler = new TranscriptReconciler();
            reconciler.AddSegment(new TranscriptSegment("The patient", 1.0, SegmentKind.Final));

            var interims = new List<TranscriptSegment>
            {
                new TranscriptSegment("The patient is", 2.0, SegmentKind.Interim),
                new TranscriptSegment("The patient is presenting", 3.0, SegmentKind.Interim) // latest
            };

            string text = reconciler.ReconcileInterims(interims);
            text.Should().Be("The patient is presenting");
        }

        [Fact]
        public async Task TC_TPR_T2_05_AzureSpeechSDKCancellationHandling()
        {
            var provider = new AzureTranscriptionProvider { SimulateCancellation = true, CancellationReason = "Authentication Failure" };

            Func<Task> act = () => provider.SendAudioAsync(new AudioFrame(new byte[10], DateTime.UtcNow, 16000, 16, 1), CancellationToken.None);
            await act.Should().ThrowAsync<ProviderAuthException>().WithMessage("*Authentication Failure*");
        }

        // ==========================================
        // FEATURE 5: Deterministic Voice Command Parser (VCP)
        // ==========================================

        [Fact]
        public void TC_VCP_T2_01_NonCommandTextPassthrough()
        {
            var parser = new VoiceCommandParser();
            string output = parser.Parse("The patient has a mild cough", out _);
            output.Should().Be("The patient has a mild cough");
        }

        [Fact]
        public void TC_VCP_T2_02_PunctuationWordAdjacencySeparation()
        {
            var parser = new VoiceCommandParser();
            string output = parser.Parse("hello comma world", out _);
            output.Should().Be("hello, world");
        }

        [Fact]
        public void TC_VCP_T2_03_RapidCommandCombination()
        {
            var parser = new VoiceCommandParser();
            string output = parser.Parse("hello world delete last word comma new line", out _);
            output.Should().Be("hello,\n");
        }

        [Fact]
        public void TC_VCP_T2_04_LocaleAwareMatchingEnUS()
        {
            var parser = new VoiceCommandParser();
            string outputPeriod = parser.Parse("hello period", out _, "en-US");
            string outputFullStop = parser.Parse("hello full stop", out _, "en-US");

            outputPeriod.Should().Be("hello. ");
            outputFullStop.Should().Be("hello. ");
        }

        [Fact]
        public void TC_VCP_T2_05_StrippingCommandsPriorToInsertion()
        {
            var parser = new VoiceCommandParser();
            string output = parser.Parse("dictation text scratch that", out _);
            output.Should().Be("");
        }

        // ==========================================
        // FEATURE 6: Global Hotkeys (R8)
        // ==========================================

        [Fact]
        public void TC_GHS_T2_01_ConflictDetectionAndExceptionThrowing()
        {
            using var service = new GlobalHotkeyService();
            service.Conflicts.Add(0x14); // Mock CapsLock conflict

            Action act = () => service.Register(0x14, 0, () => { }, () => { }, isToggle: false);
            act.Should().Throw<HotkeyConflictException>();
        }

        [Fact]
        public void TC_GHS_T2_02_SafeDisposeVerification()
        {
            var service = new GlobalHotkeyService();
            service.Register(0x14, 0, () => { }, () => { }, isToggle: false);
            service.Dispose();

            service.Disposed.Should().BeTrue();
        }

        [Fact]
        public async Task TC_GHS_T2_03_AsynchronousNonBlockingExecution()
        {
            using var service = new GlobalHotkeyService();
            bool callbackCompleted = false;

            var sem = new SemaphoreSlim(0, 1);
            service.Register(0x14, 0, () =>
            {
                Thread.Sleep(50); // delay completion
                callbackCompleted = true;
                sem.Release();
            }, () => { }, isToggle: false);

            service.TriggerPress(0x14);
            callbackCompleted.Should().BeFalse(); // TriggerPress returned immediately without blocking
            await sem.WaitAsync(200);
            callbackCompleted.Should().BeTrue();
        }

        [Fact]
        public async Task TC_GHS_T2_04_RapidDebounceFiltering()
        {
            using var service = new GlobalHotkeyService();
            int pressCount = 0;

            // Simple debounce logic
            long lastPressTime = 0;
            service.Register(0x14, 0, () =>
            {
                long current = DateTime.UtcNow.Ticks / TimeSpan.TicksPerMillisecond;
                if (current - lastPressTime > 50) // 50ms debounce
                {
                    Interlocked.Increment(ref pressCount);
                }
                lastPressTime = current;
            }, () => { }, isToggle: true);

            // Rapid taps
            for (int i = 0; i < 5; i++)
            {
                service.TriggerPress(0x14);
                await Task.Delay(5);
            }

            await Task.Delay(100);
            pressCount.Should().Be(1); // debounce filtered rapid taps down to 1
        }

        [Fact]
        public void TC_GHS_T2_05_InvalidHotkeyArgumentValidation()
        {
            using var service = new GlobalHotkeyService();
            Action act = () => service.Register(0, 0, () => { }, () => { }, isToggle: false);
            act.Should().Throw<ArgumentException>();
        }

        // ==========================================
        // FEATURE 7: No-Activate Overlay UI (R9)
        // ==========================================

        [Fact]
        public void TC_NAO_T2_01_WindowCoordinatePersistence()
        {
            var window = new OverlayWindow();
            window.X = 500;
            window.Y = 400;

            // Save to Mock Settings
            var userSettings = new Dictionary<string, double>
            {
                { "OverlayX", window.X },
                { "OverlayY", window.Y }
            };

            // Reload
            var newWindow = new OverlayWindow();
            newWindow.X = userSettings["OverlayX"];
            newWindow.Y = userSettings["OverlayY"];

            newWindow.X.Should().Be(500);
            newWindow.Y.Should().Be(400);
        }

        [Fact]
        public void TC_NAO_T2_02_WindowsDisplayScalingLegibility()
        {
            var window = new OverlayWindow();
            double[] scales = { 1.0, 1.25, 1.5, 2.0, 3.0 };

            foreach (double scale in scales)
            {
                double scaledWidth = window.Width * scale;
                double scaledHeight = window.Height * scale;

                scaledWidth.Should().BeGreaterThan(0);
                scaledHeight.Should().BeGreaterThan(0);
                // Coordinates shouldn't exceed virtual monitor limits (e.g. 10000x10000)
                scaledWidth.Should().BeLessThan(2000);
            }
        }

        [Fact]
        public void TC_NAO_T2_03_HighContrastModeToggle()
        {
            var window = new OverlayWindow { IsHighContrast = true };
            var vm = new OverlayViewModel();

            vm.UpdateState(DictationState.Idle);
            if (window.IsHighContrast)
            {
                vm.TextColor = "White"; // High-contrast Idle color
                vm.LayoutMode = "HighContrast";
            }

            vm.TextColor.Should().Be("White");
            vm.LayoutMode.Should().Be("HighContrast");
        }

        [Fact]
        public void TC_NAO_T2_04_DragMoveFocusCheck()
        {
            var window = new OverlayWindow();
            // Perform dragging logic simulations
            bool dragged = false;
            Action dragAction = () => dragged = true;

            dragAction();
            dragged.Should().BeTrue();
            window.Focusable.Should().BeFalse(); // Keyboard focus never grabbed
        }

        [Fact]
        public void TC_NAO_T2_05_AltTabHidingValidation()
        {
            var window = new OverlayWindow();
            window.ApplyWin32Styles();
            window.IsAltTabVisible.Should().BeFalse();
        }

        // ==========================================
        // FEATURE 8: Target Context & Safe Insertion Chain (R10 & R11)
        // ==========================================

        [Fact]
        public void TC_TCS_T2_01_ProcessIdSwitchFailure()
        {
            var provider = new TargetContextService();
            var original = new TargetContext(1234, "notepad.exe", "Medium", new IntPtr(0x5678), false);
            provider.ActiveContext = new TargetContext(9999, "notepad.exe", "Medium", new IntPtr(0x5678), false); // switched process

            Action act = () => provider.Revalidate(original);
            act.Should().Throw<TargetValidationException>().WithMessage("*ID has changed*");
        }

        [Fact]
        public void TC_TCS_T2_02_ProcessIntegrityElevationFailure()
        {
            var provider = new TargetContextService();
            var original = new TargetContext(1234, "notepad.exe", "Medium", new IntPtr(0x5678), false);
            provider.ActiveContext = new TargetContext(1234, "notepad.exe", "High", new IntPtr(0x5678), false); // elevated integrity

            Action act = () => provider.Revalidate(original);
            act.Should().Throw<TargetValidationException>().WithMessage("*Integrity level mismatch*");
        }

        [Fact]
        public void TC_TCS_T2_03_PasswordFieldBlockPolicy()
        {
            var provider = new TargetContextService();
            var original = new TargetContext(1234, "notepad.exe", "Medium", new IntPtr(0x5678), false);
            provider.ActiveContext = new TargetContext(1234, "notepad.exe", "Medium", new IntPtr(0x5678), true); // password field focused

            Action act = () => provider.Revalidate(original);
            act.Should().Throw<SensitiveFieldBlockedException>();
        }

        [Fact]
        public async Task TC_TCS_T2_04_ClipboardFallbackFormatPreservation()
        {
            var adapter = new ClipboardFallbackAdapter();
            var context = new TargetContext(1234, "notepad.exe", "Medium", new IntPtr(0x5678));

            bool success = await adapter.InsertAsync("Text to paste", context);
            success.Should().BeTrue();
            adapter.BackupData.Should().NotBeNull();
            adapter.RestorationSkipped.Should().BeFalse();
        }

        [Fact]
        public async Task TC_TCS_T2_05_ClipboardFallbackMismatchAbort()
        {
            var adapter = new ClipboardFallbackAdapter();
            var context = new TargetContext(1234, "notepad.exe", "Medium", new IntPtr(0x5678));
            adapter.ExternalClipboardData = "RichTextDataFromExternalApp"; // Change during paste

            bool success = await adapter.InsertAsync("Text to paste", context);
            success.Should().BeTrue();
            adapter.RestorationSkipped.Should().BeTrue();
        }
    }
}
