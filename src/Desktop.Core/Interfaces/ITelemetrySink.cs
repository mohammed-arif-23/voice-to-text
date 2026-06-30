using System;
using System.Collections.Generic;

namespace Desktop.Core;

public interface ITelemetrySink
{
    void TrackEvent(string eventName, IReadOnlyDictionary<string, string>? properties = null);
    void TrackMetric(string metricName, double value);
    void TrackException(Exception exception, IReadOnlyDictionary<string, string>? properties = null);
}
