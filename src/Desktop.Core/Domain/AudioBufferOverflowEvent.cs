using System;

namespace Desktop.Core;

public record AudioBufferOverflowEvent(DateTime Timestamp, int BufferSize, int OverflowBytes);
