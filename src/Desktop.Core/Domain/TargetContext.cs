using System;

namespace Desktop.Core;

public record TargetContext(
    int ProcessId,
    string ExecutableName,
    string IntegrityLevel,
    IntPtr WindowHandle,
    bool IsPassword = false,
    string? WindowTitle = null
);
