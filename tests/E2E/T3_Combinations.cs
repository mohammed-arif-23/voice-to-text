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
    public class T3_Combinations
    {
        [Fact]
        public async Task TC_COM_T3_01_StateMachineAndHotkeyIntegration()
        {
            var sm = new DictationSessionStateMachine();
            using var hotkeyService = new GlobalHotkeyService();

            // Setup Hotkey triggers to drive State Machine
            hotkeyService.Register(0x14, 0,
                onPress: () => sm.TransitionTo(DictationState.Arming),
                onRelease: () => sm.TransitionTo(DictationState.Finalizing),
                isToggle: false
            );

            // Press
            hotkeyService.TriggerPress(0x14);
            await Task.Delay(20);
            sm.CurrentState.Should().Be(DictationState.Arming);

            // Move state machine forward to Streaming
            sm.TransitionTo(DictationState.Capturing);
            sm.TransitionTo(DictationState.Streaming);

            // Release
            hotkeyService.TriggerRelease(0x14);
            await Task.Delay(20);
            sm.CurrentState.Should().Be(DictationState.Finalizing);
        }

        [Fact]
        public async Task TC_COM_T3_02_StateMachineAndWasapiCaptureEventIntegration()
        {
            var sm = new DictationSessionStateMachine();
            var audioService = new WasapiAudioCaptureService();

            // Set up state machine to follow audio service overflow event
            audioService.BufferOverflow += (ev) =>
            {
                if (sm.CurrentState == DictationState.Streaming)
                {
                    sm.TransitionTo(DictationState.RecoverableFailure);
                    audioService.StopCapture();
                }
            };

            sm.TransitionTo(DictationState.Arming);
            sm.TransitionTo(DictationState.Capturing);
            sm.TransitionTo(DictationState.Streaming);

            audioService.StartCapture("Default Mic");
            audioService.SimulateOverflow = true;

            // Trigger stream to trigger overflow check
            var cts = new CancellationTokenSource(100);
            var enumerator = audioService.StreamFramesAsync(cts.Token).GetAsyncEnumerator();
            await Task.Run(async () => await enumerator.MoveNextAsync());

            sm.CurrentState.Should().Be(DictationState.RecoverableFailure);
        }

        [Fact]
        public void TC_COM_T3_03_TranscriptionAndStateMachineCommandStopIntegration()
        {
            var sm = new DictationSessionStateMachine();
            var reconciler = new TranscriptReconciler();

            sm.TransitionTo(DictationState.Arming);
            sm.TransitionTo(DictationState.Capturing);
            sm.TransitionTo(DictationState.Streaming);

            var segment = new TranscriptSegment("This is dictation stop dictation", 1.0, SegmentKind.Final);
            string result = reconciler.AddSegment(segment);

            var controlSignals = new List<string>();
            var parser = new VoiceCommandParser();
            parser.Parse(segment.Text, out controlSignals);

            if (controlSignals.Contains("stop"))
            {
                sm.TransitionTo(DictationState.Finalizing);
            }

            result.Should().Be("This is dictation"); // stripped
            sm.CurrentState.Should().Be(DictationState.Finalizing);
        }

        [Fact]
        public void TC_COM_T3_04_OverlayUIAndStateMachineVisualization()
        {
            var sm = new DictationSessionStateMachine();
            var vm = new OverlayViewModel();

            sm.StateChanged += (record) =>
            {
                vm.UpdateState(record.ToState);
            };

            sm.TransitionTo(DictationState.Arming);
            vm.CurrentState.Should().Be(DictationState.Arming);
            vm.TextColor.Should().Be("Orange");

            sm.TransitionTo(DictationState.Capturing);
            vm.CurrentState.Should().Be(DictationState.Capturing);
            vm.TextColor.Should().Be("Red");
        }

        [Fact]
        public void TC_COM_T3_05_TargetContextMismatchAndErrorOverlayIntegration()
        {
            var sm = new DictationSessionStateMachine();
            var vm = new OverlayViewModel();
            var contextService = new TargetContextService();

            sm.StateChanged += (record) =>
            {
                vm.UpdateState(record.ToState);
            };

            sm.TransitionTo(DictationState.Arming);
            sm.TransitionTo(DictationState.Capturing);
            sm.TransitionTo(DictationState.Streaming);
            sm.TransitionTo(DictationState.Finalizing);
            sm.TransitionTo(DictationState.ReadyToInsert);
            sm.TransitionTo(DictationState.ValidatingTarget);

            // Revalidate fails
            var original = new TargetContext(1234, "notepad.exe", "Medium", new IntPtr(0x5678), false);
            contextService.ActiveContext = new TargetContext(9999, "notepad.exe", "Medium", new IntPtr(0x5678), false); // Process switched

            try
            {
                contextService.Revalidate(original);
            }
            catch (TargetValidationException)
            {
                sm.TransitionTo(DictationState.FatalFailure);
                vm.AppendText("Error: Focus switched during dictation.");
            }

            sm.CurrentState.Should().Be(DictationState.FatalFailure);
            vm.TextColor.Should().Be("DarkRed");
            vm.TranscriptText.Should().Contain("Error:");
        }

        [Fact]
        public async Task TC_COM_T3_06_WasapiSilenceAndDeepgramConnectionIntegration()
        {
            var audioService = new WasapiAudioCaptureService();
            var deepgram = new DeepgramTranscriptionProvider();

            audioService.StartCapture("Default Mic");
            await deepgram.ConnectAsync("valid-token", CancellationToken.None);

            int keepAlives = 0;
            var cts = new CancellationTokenSource(150);

            await foreach (var frame in audioService.StreamFramesAsync(cts.Token))
            {
                await deepgram.SendAudioAsync(frame, cts.Token);

                // Simulate frame analysis for silence -> sends KeepAlive
                bool isSilence = frame.Data.All(b => b == 0);
                if (isSilence)
                {
                    keepAlives++;
                }
            }

            audioService.StopCapture();
            deepgram.LastConnectionUri.Should().Contain("model=nova-3");
            keepAlives.Should().BeGreaterThan(0);
        }

        [Fact]
        public async Task TC_COM_T3_07_SafeInsertionFallbackAndRedactionLoggingIntegration()
        {
            var clipboard = new ClipboardFallbackAdapter();
            var context = new TargetContext(1234, "notepad.exe", "Medium", new IntPtr(0x5678));

            string textToInsert = "Sensitive clinical notes";
            bool success = await clipboard.InsertAsync(textToInsert, context);
            success.Should().BeTrue();

            // Log details
            string logText = $"Clipboard Fallback inserted text: {clipboard.PastedText}";
            string redactedLog = LoggingRedactionPipeline.RedactValue("Transcript", logText);

            redactedLog.Should().Be("***");
        }

        [Fact]
        public async Task TC_COM_T3_08_VoiceCommandParserAndSafeInsertionIntegration()
        {
            var parser = new VoiceCommandParser();
            var context = new TargetContext(1234, "notepad.exe", "Medium", new IntPtr(0x5678));
            var uiaAdapter = new UiaValuePatternAdapter();

            string dictatedText = "first line new paragraph second line";
            string formattedText = parser.Parse(dictatedText, out _);

            bool success = await uiaAdapter.InsertAsync(formattedText, context);
            success.Should().BeTrue();
            uiaAdapter.InsertedValue.Should().Be("first line\n\nsecond line");
        }
    }
}
