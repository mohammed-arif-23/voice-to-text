using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Desktop.Core;

namespace Desktop.Insertion;

public class BrowserExtensionAdapter : ITextInsertionAdapter
{
    public AdapterKind Kind => AdapterKind.BrowserExtension;
    public bool ExtensionConnected { get; set; } = true;

    public bool IsAvailable(TargetContext context)
    {
        return ExtensionConnected && context.ExecutableName.Contains("chrome", StringComparison.OrdinalIgnoreCase);
    }

    public Task<bool> InsertAsync(string text, TargetContext context)
    {
        return Task.FromResult(IsAvailable(context));
    }
}

public class UiaValuePatternAdapter : ITextInsertionAdapter
{
    public AdapterKind Kind => AdapterKind.UiaValuePattern;
    public bool UiaSupported { get; set; } = true;
    public string? InsertedValue { get; private set; }

    public bool IsAvailable(TargetContext context)
    {
        return UiaSupported &&
               !context.ExecutableName.Contains("rdp", StringComparison.OrdinalIgnoreCase) &&
               !context.ExecutableName.Contains("mstsc", StringComparison.OrdinalIgnoreCase);
    }

    public Task<bool> InsertAsync(string text, TargetContext context)
    {
        if (IsAvailable(context))
        {
            InsertedValue = text;
            return Task.FromResult(true);
        }
        return Task.FromResult(false);
    }
}

public class SendInputAdapter : ITextInsertionAdapter
{
    public AdapterKind Kind => AdapterKind.SendInput;
    public bool SendInputBlocked { get; set; }
    public string? SentKeys { get; private set; }

    public bool IsAvailable(TargetContext context)
    {
        return !SendInputBlocked;
    }

    public Task<bool> InsertAsync(string text, TargetContext context)
    {
        if (IsAvailable(context))
        {
            SentKeys = text;
            return Task.FromResult(true);
        }
        return Task.FromResult(false);
    }
}

public class ClipboardFallbackAdapter : ITextInsertionAdapter
{
    public AdapterKind Kind => AdapterKind.ClipboardFallback;
    public bool ClipboardLocked { get; set; }
    public int ClipboardOpenRetries { get; private set; }
    public object? BackupData { get; private set; }
    public bool RestorationSkipped { get; private set; }
    public string? PastedText { get; private set; }
    public object? ExternalClipboardData { get; set; }

    public bool IsAvailable(TargetContext context) => true;

    public async Task<bool> InsertAsync(string text, TargetContext context)
    {
        ClipboardOpenRetries = 0;
        while (ClipboardLocked && ClipboardOpenRetries < 3)
        {
            ClipboardOpenRetries++;
            await Task.Delay(10);
        }

        if (ClipboardLocked)
        {
            return false;
        }

        BackupData = "RichTextData";
        PastedText = text;

        if (ExternalClipboardData != null)
        {
            RestorationSkipped = true;
        }
        else
        {
            RestorationSkipped = false;
        }

        return true;
    }
}

public class InsertionAdapterChain
{
    private readonly List<ITextInsertionAdapter> _adapters;

    public InsertionAdapterChain(List<ITextInsertionAdapter> adapters)
    {
        _adapters = adapters;
    }

    public async Task<InsertionResult> ExecuteAsync(string text, TargetContext context, List<AdapterKind> enabledAdapters)
    {
        foreach (var adapter in _adapters)
        {
            if (enabledAdapters.Contains(adapter.Kind) && adapter.IsAvailable(context))
            {
                bool success = await adapter.InsertAsync(text, context);
                if (success)
                {
                    return new InsertionResult(true, text, adapter.Kind);
                }
            }
        }
        return new InsertionResult(false, "", AdapterKind.SendInput, "No adapter in the chain succeeded.");
    }
}
