using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;

namespace Desktop.Core;

public interface IOfflineTranscriptionProvider
{
    Task<List<TranscriptSegment>> TranscribeAsync(byte[] audioData, string modelPath, CancellationToken cancellationToken);
}
