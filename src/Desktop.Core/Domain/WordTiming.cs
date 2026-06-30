using System;

namespace Desktop.Core;

public record WordTiming(string Word, TimeSpan Start, TimeSpan End, double Confidence);
