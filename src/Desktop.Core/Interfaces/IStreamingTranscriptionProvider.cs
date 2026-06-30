using System;
using System.Threading;
using System.Threading.Tasks;

namespace Desktop.Core;

public interface IStreamingTranscriptionProvider
{
    event Action<TranscriptSegment>? SegmentReceived;
    event Action<Exception>? ErrorOccurred;
    Task ConnectAsync(string token, CancellationToken cancellationToken);
    Task SendAudioAsync(AudioFrame frame, CancellationToken cancellationToken);
    Task DisconnectAsync();
}
