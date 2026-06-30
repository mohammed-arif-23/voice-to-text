using System.Threading.Tasks;

namespace Desktop.Core;

public interface ITextInsertionAdapter
{
    AdapterKind Kind { get; }
    bool IsAvailable(TargetContext context);
    Task<bool> InsertAsync(string text, TargetContext context);
}
