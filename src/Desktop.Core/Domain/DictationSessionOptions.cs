#pragma warning disable CA2227, CA1002

using System;
using System.Collections.Generic;

namespace Desktop.Core;

public class DictationSessionOptions
{
    public bool UsePushToTalk { get; set; } = true;
    public TimeSpan Timeout { get; set; } = TimeSpan.FromSeconds(30);
    public SensitivityClassification Sensitivity { get; set; } = SensitivityClassification.Normal;
    public List<AdapterKind> EnabledAdapters { get; set; } = new()
    {
        AdapterKind.BrowserExtension,
        AdapterKind.UiaValuePattern,
        AdapterKind.SendInput,
        AdapterKind.ClipboardFallback
    };
    public string Locale { get; set; } = "en-US";
    public string? DeepgramToken { get; set; } = "valid-token";
    public string? WhisperModelPath { get; set; } = "models/ggml-base.en.bin";
}
