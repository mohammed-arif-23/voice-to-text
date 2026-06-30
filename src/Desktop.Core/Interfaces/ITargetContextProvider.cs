namespace Desktop.Core;

public interface ITargetContextProvider
{
    TargetContext CaptureContext();
    bool Revalidate(TargetContext originalContext);
}
