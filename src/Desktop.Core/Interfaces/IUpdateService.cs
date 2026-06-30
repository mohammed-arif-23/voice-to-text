using System.Threading;
using System.Threading.Tasks;

namespace Desktop.Core;

public interface IUpdateService
{
    Task<bool> CheckForUpdatesAsync(CancellationToken cancellationToken = default);
    Task DownloadUpdateAsync(CancellationToken cancellationToken = default);
}
