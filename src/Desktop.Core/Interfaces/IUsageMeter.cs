using System.Threading;
using System.Threading.Tasks;

namespace Desktop.Core;

public interface IUsageMeter
{
    Task ReportUsageAsync(string meterName, double quantity, CancellationToken cancellationToken = default);
}
