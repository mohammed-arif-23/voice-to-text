using System;
using Desktop.Core;

namespace Desktop.Targeting;

public class TargetContextService : ITargetContextProvider
{
    public TargetContext? CurrentContext { get; set; }
    public TargetContext? ActiveContext { get; set; }

    public TargetContext CaptureContext()
    {
        if (CurrentContext == null)
        {
            return new TargetContext(1234, "notepad.exe", "Medium", new IntPtr(0x5678), false, "Document.txt - Notepad");
        }
        return CurrentContext;
    }

    public bool Revalidate(TargetContext originalContext)
    {
        var active = ActiveContext ?? CaptureContext();

        if (active.ProcessId != originalContext.ProcessId)
        {
            throw new TargetValidationException("Active process ID has changed.");
        }

        if (active.IntegrityLevel != originalContext.IntegrityLevel)
        {
            throw new TargetValidationException("Integrity level mismatch.");
        }

        if (active.IsPassword)
        {
            throw new SensitiveFieldBlockedException("Cannot insert text into password fields.");
        }

        return true;
    }
}
