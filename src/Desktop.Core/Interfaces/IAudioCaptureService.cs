using System;
using System.Collections.Generic;
using System.Threading;

namespace Desktop.Core;

public interface IAudioCaptureService
{
    event Action<AudioDeviceChangedEvent>? DeviceChanged;
    event Action<AudioBufferOverflowEvent>? BufferOverflow;
    IEnumerable<string> EnumerateDevices();
    IAsyncEnumerable<AudioFrame> StreamFramesAsync(CancellationToken cancellationToken);
    void StartCapture(string deviceName);
    void StopCapture();
}
