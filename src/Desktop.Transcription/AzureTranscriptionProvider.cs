using System;
using System.Threading;
using System.Threading.Tasks;
using Desktop.Core;

namespace Desktop.Transcription;

public class AzureTranscriptionProvider : IStreamingTranscriptionProvider
{
    public event Action<TranscriptSegment>? SegmentReceived;
    public event Action<Exception>? ErrorOccurred;

    public bool SimulateCancellation { get; set; }
    public string? CancellationReason { get; set; }

    public Task ConnectAsync(string token, CancellationToken cancellationToken)
    {
        return Task.CompletedTask;
    }

    public Task SendAudioAsync(AudioFrame frame, CancellationToken cancellationToken)
    {
        if (SimulateCancellation)
        {
            var ex = new ProviderAuthException($"Azure cancellation: {CancellationReason ?? "Error"}");
            ErrorOccurred?.Invoke(ex);
            throw ex;
        }
        return Task.CompletedTask;
    }

    public Task DisconnectAsync()
    {
        return Task.CompletedTask;
    }

    public void TriggerRecognizing(string text)
    {
        SegmentReceived?.Invoke(new TranscriptSegment(text, 0.0, SegmentKind.Interim));
    }

    public void TriggerRecognized(string text)
    {
        SegmentReceived?.Invoke(new TranscriptSegment(text, 0.0, SegmentKind.Final));
    }
}
