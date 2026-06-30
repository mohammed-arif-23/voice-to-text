using System;
using System.Threading;
using System.Threading.Tasks;

namespace Desktop.Core;

public interface IClipboardService
{
    Task CopyTextAsync(string text, CancellationToken cancellationToken = default);
    Task<string?> GetTextAsync(CancellationToken cancellationToken = default);
    Task<IDisposable> PreserveClipboardAsync(CancellationToken cancellationToken = default);
}
