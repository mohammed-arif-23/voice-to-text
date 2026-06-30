using System.Threading;
using System.Threading.Tasks;

namespace Desktop.Core;

public interface IBrowserBridge
{
    Task<bool> IsBrowserFocusedAsync(TargetContext context, CancellationToken cancellationToken = default);
    Task<InsertionResult> InsertTextViaBridgeAsync(TargetContext context, string text, CancellationToken cancellationToken = default);
}
