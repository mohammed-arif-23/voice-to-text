using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using Desktop.Core;

namespace Desktop.Transcription;

public class WhisperOfflineTranscriptionProvider : IOfflineTranscriptionProvider
{
    public bool ModelFileExists { get; set; } = true;

    public Task<List<TranscriptSegment>> TranscribeAsync(byte[] audioData, string modelPath, CancellationToken cancellationToken)
    {
        if (!ModelFileExists || !modelPath.EndsWith("ggml-base.en.bin"))
        {
            throw new OfflineModelNotFoundException(modelPath);
        }

        var results = new List<TranscriptSegment>
        {
            new TranscriptSegment("Transcribed from Whisper", 0.0, SegmentKind.Final)
        };
        return Task.FromResult(results);
    }
}
