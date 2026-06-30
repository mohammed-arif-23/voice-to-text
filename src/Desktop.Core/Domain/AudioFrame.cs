#pragma warning disable CA1819 // Properties should not return arrays

using System;

namespace Desktop.Core;

public record AudioFrame(
    byte[] Data,
    DateTime Timestamp,
    int SampleRate,
    int BitsPerSample,
    int Channels
);
