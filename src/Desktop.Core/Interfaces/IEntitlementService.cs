using System.Threading;
using System.Threading.Tasks;

namespace Desktop.Core;

public interface IEntitlementService
{
    Task<bool> IsFeatureEnabledAsync(string featureName, CancellationToken cancellationToken = default);
}
