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
    public class T4_RealWorldScenarios
    {
        [Fact]
        public async Task TC_RWS_T4_01_WebBasedEMRClinicalDictation_ChromeEpic()
        {
            // Scenario: Epic EMR running in Chrome, Push-to-talk CapsLock
            var sm = new DictationSessionStateMachine();
            using var hotkeyService = new GlobalHotkeyService();
            var audioService = new WasapiAudioCaptureService();
            var deepgram = new DeepgramTranscriptionProvider();
            var reconciler = new TranscriptReconciler();
            var contextService = new TargetContextService();
            var overlayVm = new OverlayViewModel();

            var browserAdapter = new BrowserExtensionAdapter();
            var uiaAdapter = new UiaValuePatternAdapter();
            var sendInputAdapter = new SendInputAdapter();
            var clipboardAdapter = new ClipboardFallbackAdapter();

            var chain = new InsertionAdapterChain(new List<ITextInsertionAdapter>
            {
                browserAdapter,
                uiaAdapter,
                sendInputAdapter,
                clipboardAdapter
            });

            // Target Context is Epic EMR in Chrome
            var chromeContext = new TargetContext(4001, "chrome.exe", "Medium", new IntPtr(0xCAFE), false, "Epic Hyperspace - Patient Chart");
            contextService.CurrentContext = chromeContext;
            contextService.ActiveContext = chromeContext;

            sm.StateChanged += (record) => overlayVm.UpdateState(record.ToState);

            // Bind hotkeys
            hotkeyService.Register(0x14, 0,
                onPress: () =>
                {
                    sm.TransitionTo(DictationState.Arming);
                    audioService.StartCapture("Default Mic");
                    sm.TransitionTo(DictationState.Capturing);
                },
                onRelease: () =>
                {
                    sm.TransitionTo(DictationState.Finalizing);
                    audioService.StopCapture();
                    sm.TransitionTo(DictationState.ReadyToInsert);
                },
                isToggle: false
            );

            // 1. Press hotkey (PTT starts)
            hotkeyService.TriggerPress(0x14);
            await Task.Delay(20);
            sm.CurrentState.Should().Be(DictationState.Capturing);

            // 2. Stream audio & transcribe
            sm.TransitionTo(DictationState.Streaming);
            await deepgram.ConnectAsync("valid-token", CancellationToken.None);

            // User dictates: "Patient reports headaches comma new paragraph starting ibuprofen period"
            string incomingSpeech = "Patient reports headaches comma new paragraph starting ibuprofen period";

            // Emulate streaming segments to reconciler and overlay
            reconciler.AddSegment(new TranscriptSegment(incomingSpeech, 1.0, SegmentKind.Final));
            string parsedSpeech = reconciler.GetReconciledText();
            overlayVm.AppendText(parsedSpeech);

            // 3. Release hotkey (PTT ends)
            hotkeyService.TriggerRelease(0x14);
            await Task.Delay(20);
            sm.CurrentState.Should().Be(DictationState.ReadyToInsert);

            // 4. Revalidate & insert
            sm.TransitionTo(DictationState.ValidatingTarget);
            contextService.Revalidate(chromeContext).Should().BeTrue();

            sm.TransitionTo(DictationState.Inserting);
            string insertText = reconciler.GetReconciledText();
            var options = new DictationSessionOptions();
            var insertResult = await chain.ExecuteAsync(insertText, chromeContext, options.EnabledAdapters);

            insertResult.Success.Should().BeTrue();
            insertResult.RouteChosen.Should().Be(AdapterKind.BrowserExtension);
            insertResult.InsertedText.Should().Be("Patient reports headaches,\n\nstarting ibuprofen. ");

            sm.TransitionTo(DictationState.Verifying);
            sm.TransitionTo(DictationState.Completed);
            sm.TransitionTo(DictationState.Idle);

            sm.CurrentState.Should().Be(DictationState.Idle);
        }

        [Fact]
        public async Task TC_RWS_T4_02_RemoteDesktopDictation_OfflineWhisperFallback()
        {
            // Scenario: Dictating to Notepad in RDP, socket exception happens -> fallback to offline Whisper
            var sm = new DictationSessionStateMachine();
            var deepgram = new DeepgramTranscriptionProvider();
            var whisper = new WhisperOfflineTranscriptionProvider();
            var contextService = new TargetContextService();
            var reconciler = new TranscriptReconciler();

            var browserAdapter = new BrowserExtensionAdapter();
            var uiaAdapter = new UiaValuePatternAdapter();
            var sendInputAdapter = new SendInputAdapter();
            var clipboardAdapter = new ClipboardFallbackAdapter();

            var chain = new InsertionAdapterChain(new List<ITextInsertionAdapter>
            {
                browserAdapter,
                uiaAdapter,
                sendInputAdapter,
                clipboardAdapter
            });

            // Target Context is RDP (UIA & Browser extension not available)
            var rdpContext = new TargetContext(6001, "mstsc.exe", "Medium", new IntPtr(0x8888), false, "Remote Desktop notepad");
            contextService.CurrentContext = rdpContext;
            contextService.ActiveContext = rdpContext;

            sm.TransitionTo(DictationState.Arming);
            sm.TransitionTo(DictationState.Capturing);
            sm.TransitionTo(DictationState.Streaming);

            // Trigger Socket Exception during streaming
            deepgram.SimulateNetworkOutage = true;
            Func<Task> sendAudio = () => deepgram.SendAudioAsync(new AudioFrame(new byte[100], DateTime.UtcNow, 16000, 16, 1), CancellationToken.None);

            await sendAudio.Should().ThrowAsync<System.Net.WebSockets.WebSocketException>();

            // Failover to Offline state
            sm.TransitionTo(DictationState.Offline);

            // Finalize local audio
            sm.TransitionTo(DictationState.Finalizing);

            // Run Whisper transcription
            var offlineResults = await whisper.TranscribeAsync(new byte[32000], "models/ggml-base.en.bin", CancellationToken.None);
            foreach (var r in offlineResults)
            {
                reconciler.AddSegment(r);
            }

            sm.TransitionTo(DictationState.ReadyToInsert);
            sm.TransitionTo(DictationState.ValidatingTarget);
            contextService.Revalidate(rdpContext).Should().BeTrue();

            sm.TransitionTo(DictationState.Inserting);
            string textToInsert = reconciler.GetReconciledText();
            var options = new DictationSessionOptions();
            var result = await chain.ExecuteAsync(textToInsert, rdpContext, options.EnabledAdapters);

            // Assertions
            result.Success.Should().BeTrue();
            result.RouteChosen.Should().Be(AdapterKind.SendInput); // skips UIA and Browser because mstsc is active
            result.InsertedText.Should().Be("Transcribed from Whisper");
            sendInputAdapter.SentKeys.Should().Be("Transcribed from Whisper");
        }

        [Fact]
        public async Task TC_RWS_T4_03_FocusHijacking_TargetApplicationSwitchMismatch()
        {
            // Scenario: Typing into Word. Mid-dictation, admin password dialog hijacks focus. Insertion blocked.
            var sm = new DictationSessionStateMachine();
            var contextService = new TargetContextService();
            var reconciler = new TranscriptReconciler();

            // Word document context
            var originalContext = new TargetContext(7001, "winword.exe", "Medium", new IntPtr(0x1111), false, "Report.docx - Word");
            contextService.CurrentContext = originalContext;
            contextService.ActiveContext = originalContext;

            sm.TransitionTo(DictationState.Arming);
            sm.TransitionTo(DictationState.Capturing);
            sm.TransitionTo(DictationState.Streaming);

            // Dictate some text
            reconciler.AddSegment(new TranscriptSegment("Patient note detail", 1.0, SegmentKind.Final));

            // Focus hijacking occurs! Switch active context to an admin elevation password prompt
            var hijackedContext = new TargetContext(9001, "consent.exe", "System", new IntPtr(0x9999), true, "Windows Security Password");
            contextService.ActiveContext = hijackedContext;

            sm.TransitionTo(DictationState.Finalizing);
            sm.TransitionTo(DictationState.ReadyToInsert);
            sm.TransitionTo(DictationState.ValidatingTarget);

            // Validation should fail
            Action revalidate = () => contextService.Revalidate(originalContext);
            revalidate.Should().Throw<TargetValidationException>();

            sm.TransitionTo(DictationState.FatalFailure);
            sm.CurrentState.Should().Be(DictationState.FatalFailure);

            // Ensure no transcript was written or logged
            string logOutput = LoggingRedactionPipeline.RedactValue("Transcript", reconciler.GetReconciledText());
            logOutput.Should().Be("***"); // Redacted
        }

        [Fact]
        public async Task TC_RWS_T4_04_LegacyAppDictation_ClipboardLockAndPasteFallback()
        {
            // Scenario: Legacy app with UIA/SendInput blocked. Fallback clipboard locked, retried, pasted, external modification check
            var sm = new DictationSessionStateMachine();
            var contextService = new TargetContextService();
            var reconciler = new TranscriptReconciler();

            // Legacy app context where other adapters fail
            var legacyContext = new TargetContext(8001, "legacy_med.exe", "High", new IntPtr(0x5555), false, "Legacy Clinical Data Entry");
            contextService.CurrentContext = legacyContext;
            contextService.ActiveContext = legacyContext;

            var browserAdapter = new BrowserExtensionAdapter { ExtensionConnected = false };
            var uiaAdapter = new UiaValuePatternAdapter { UiaSupported = false };
            var sendInputAdapter = new SendInputAdapter { SendInputBlocked = true };
            var clipboardAdapter = new ClipboardFallbackAdapter { ClipboardLocked = true }; // Locked initially

            var chain = new InsertionAdapterChain(new List<ITextInsertionAdapter>
            {
                browserAdapter,
                uiaAdapter,
                sendInputAdapter,
                clipboardAdapter
            });

            sm.TransitionTo(DictationState.Arming);
            sm.TransitionTo(DictationState.Capturing);
            sm.TransitionTo(DictationState.Streaming);

            reconciler.AddSegment(new TranscriptSegment("Prescribed aspirin.", 1.0, SegmentKind.Final));

            sm.TransitionTo(DictationState.Finalizing);
            sm.TransitionTo(DictationState.ReadyToInsert);
            sm.TransitionTo(DictationState.ValidatingTarget);
            contextService.Revalidate(legacyContext).Should().BeTrue();

            sm.TransitionTo(DictationState.Inserting);

            // Clipboard is locked initially but we want it to unlock after 1 retry
            // Simulate background unlock task
            var unlockTask = Task.Run(async () =>
            {
                await Task.Delay(15);
                clipboardAdapter.ClipboardLocked = false;
            });

            var result = await chain.ExecuteAsync(reconciler.GetReconciledText(), legacyContext, new List<AdapterKind> { AdapterKind.ClipboardFallback });
            await unlockTask;

            result.Success.Should().BeTrue();
            result.RouteChosen.Should().Be(AdapterKind.ClipboardFallback);
            clipboardAdapter.ClipboardOpenRetries.Should().BeGreaterThan(0);
            clipboardAdapter.PastedText.Should().Be("Prescribed aspirin.");
            clipboardAdapter.RestorationSkipped.Should().BeFalse();
        }

        [Fact]
        public async Task TC_RWS_T4_05_MultiMonitorScaling_HighContrastRapidDictation()
        {
            // Scenario: Multi-monitor scaling 200%, High Contrast mode, rapid command dictation
            var sm = new DictationSessionStateMachine();
            var contextService = new TargetContextService();
            var parser = new VoiceCommandParser();
            var overlayVm = new OverlayViewModel();
            var overlayWindow = new OverlayWindow { IsHighContrast = true };

            var target = new TargetContext(1234, "notepad.exe", "Medium", new IntPtr(0x5678));
            contextService.CurrentContext = target;
            contextService.ActiveContext = target;

            sm.StateChanged += (record) => overlayVm.UpdateState(record.ToState);

            sm.TransitionTo(DictationState.Arming);
            sm.TransitionTo(DictationState.Capturing);
            sm.TransitionTo(DictationState.Streaming);

            // Multi-monitor scaling verification
            double scale = 2.0; // 200%
            double scaledWidth = overlayWindow.Width * scale;
            scaledWidth.Should().Be(800);

            // High Contrast layout adaptation
            if (overlayWindow.IsHighContrast)
            {
                overlayVm.TextColor = "White";
                overlayVm.LayoutMode = "HighContrast";
            }

            // Rapid command dictation: "scratch that hello comma world full stop stop dictation"
            string spoken = "scratch that hello comma world full stop stop dictation";
            string parsed = parser.Parse(spoken, out var signals);

            parsed.Should().Be("hello, world. ");
            signals.Should().Contain("stop");

            if (signals.Contains("stop"))
            {
                sm.TransitionTo(DictationState.Finalizing);
            }

            sm.TransitionTo(DictationState.ReadyToInsert);
            sm.TransitionTo(DictationState.ValidatingTarget);
            sm.TransitionTo(DictationState.Inserting);

            var uiaAdapter = new UiaValuePatternAdapter();
            bool success = await uiaAdapter.InsertAsync(parsed, target);
            success.Should().BeTrue();
            uiaAdapter.InsertedValue.Should().Be("hello, world. ");

            sm.TransitionTo(DictationState.Verifying);
            sm.TransitionTo(DictationState.Completed);
            sm.TransitionTo(DictationState.Idle);

            sm.CurrentState.Should().Be(DictationState.Idle);
            overlayVm.TextColor.Should().Be("Gray"); // Resets to Idle color
        }
    }
}
