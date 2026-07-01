using System;
using System.Speech.Recognition;
using System.Threading;
using System.Threading.Tasks;
using Desktop.Core;

namespace Desktop.Transcription;

/// <summary>
/// 100% free, offline transcription using the Windows built-in
/// Speech Recognition API (SAPI / System.Speech).
/// No API key, no internet, no model download required.
/// Automatically stops after 2 seconds of silence.
/// </summary>
public class WindowsSpeechTranscriptionProvider : IDisposable
{
    /// <summary>Fired when a partial or final transcript segment is ready.</summary>
    public event Action<TranscriptSegment>? SegmentReceived;

    /// <summary>Fired when recognition ends (silence timeout or explicit stop).</summary>
    public event Action? RecognitionCompleted;

    /// <summary>Fired when an unrecoverable error occurs.</summary>
    public event Action<Exception>? ErrorOccurred;

    private SpeechRecognitionEngine? _engine;
    private readonly object _lock = new();
    private bool _active;

    /// <summary>
    /// Start listening. Fires <see cref="SegmentReceived"/> for each word group,
    /// fires <see cref="RecognitionCompleted"/> after <paramref name="silenceSeconds"/>
    /// of silence, then stops automatically.
    /// </summary>
    public void StartListening(int silenceSeconds = 2)
    {
        lock (_lock)
        {
            if (_active) return;
            _active = true;
        }

        Task.Run(() =>
        {
            try
            {
                lock (_lock)
                {
                    _engine = new SpeechRecognitionEngine();

                    // Load free-form dictation grammar — accepts any speech
                    _engine.LoadGrammar(new DictationGrammar());

                    // Use default microphone
                    _engine.SetInputToDefaultAudioDevice();

                    // How long to wait for initial speech before giving up
                    _engine.InitialSilenceTimeout = TimeSpan.FromSeconds(8);

                    // How long of silence after speech before finalizing
                    _engine.EndSilenceTimeout = TimeSpan.FromSeconds(silenceSeconds);
                    _engine.EndSilenceTimeoutAmbiguous = TimeSpan.FromSeconds(silenceSeconds);

                    _engine.SpeechHypothesized += OnHypothesized;
                    _engine.SpeechRecognized += OnRecognized;
                    _engine.RecognizeCompleted += OnCompleted;
                }

                // Async recognition — returns when silence timeout fires
                _engine.RecognizeAsync(RecognizeMode.Multiple);
            }
            catch (Exception ex)
            {
                ErrorOccurred?.Invoke(ex);
                CleanupEngine();
            }
        });
    }

    /// <summary>Force-stop recognition before the silence timeout.</summary>
    public void StopListening()
    {
        lock (_lock)
        {
            if (!_active) return;
            _active = false;
            _engine?.RecognizeAsyncStop();
        }
    }

    // Interim partial result — show live in UI
    private void OnHypothesized(object? sender, SpeechHypothesizedEventArgs e)
    {
        if (string.IsNullOrWhiteSpace(e.Result.Text)) return;
        var segment = new TranscriptSegment(e.Result.Text, e.Result.Confidence, SegmentKind.Interim);
        SegmentReceived?.Invoke(segment);
    }

    // Final committed result for this phrase
    private void OnRecognized(object? sender, SpeechRecognizedEventArgs e)
    {
        if (string.IsNullOrWhiteSpace(e.Result.Text)) return;
        // Only accept results with at least 30% confidence
        if (e.Result.Confidence < 0.30f) return;
        var segment = new TranscriptSegment(e.Result.Text, e.Result.Confidence, SegmentKind.Final);
        SegmentReceived?.Invoke(segment);
    }

    // Called when RecognizeAsync finishes (silence timeout or explicit stop)
    private void OnCompleted(object? sender, RecognizeCompletedEventArgs e)
    {
        CleanupEngine();
        RecognitionCompleted?.Invoke();
    }

    private void CleanupEngine()
    {
        lock (_lock)
        {
            _active = false;
            if (_engine != null)
            {
                _engine.SpeechHypothesized -= OnHypothesized;
                _engine.SpeechRecognized -= OnRecognized;
                _engine.RecognizeCompleted -= OnCompleted;
                try { _engine.Dispose(); } catch { /* ignored */ }
                _engine = null;
            }
        }
    }

    public void Dispose()
    {
        StopListening();
        CleanupEngine();
        GC.SuppressFinalize(this);
    }
}
