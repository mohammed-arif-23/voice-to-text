using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Desktop.Core;

namespace Desktop.Audio;

public class WasapiAudioCaptureService : IAudioCaptureService
{
    public event Action<AudioDeviceChangedEvent>? DeviceChanged;
    public event Action<AudioBufferOverflowEvent>? BufferOverflow;

    public bool SimulateMissingDevice { get; set; }
    public bool SimulateAccessDenied { get; set; }
    public bool SimulateExclusiveModeConflict { get; set; }
    public bool SimulateOverflow { get; set; }
    public int SampleRateOverride { get; set; } = 44100;

    private bool _isCapturing;
    private readonly List<string> _devices = new() { "Default Mic", "USB Audio Device" };

    public IEnumerable<string> EnumerateDevices()
    {
        return _devices;
    }

    public void AddDevice(string deviceName)
    {
        _devices.Add(deviceName);
        DeviceChanged?.Invoke(new AudioDeviceChangedEvent(deviceName, "Added"));
    }

    public void RemoveDevice(string deviceName)
    {
        if (_devices.Remove(deviceName))
        {
            DeviceChanged?.Invoke(new AudioDeviceChangedEvent(deviceName, "Removed"));
            if (_isCapturing)
            {
                StopCapture();
            }
        }
    }

    public void StartCapture(string deviceName)
    {
        if (SimulateMissingDevice || !_devices.Contains(deviceName))
        {
            throw new AudioCaptureException("Input device not found.", unchecked((int)0x80070490));
        }

        if (SimulateAccessDenied)
        {
            throw new AudioCaptureException("Access to microphone denied by system privacy settings.", unchecked((int)0x80070005));
        }

        if (SimulateExclusiveModeConflict)
        {
            throw new AudioCaptureException("Device is locked by another process in exclusive mode.", unchecked((int)0x8889000A));
        }

        _isCapturing = true;
    }

    public void StopCapture()
    {
        _isCapturing = false;
    }

    public async IAsyncEnumerable<AudioFrame> StreamFramesAsync([System.Runtime.CompilerServices.EnumeratorCancellation] CancellationToken cancellationToken)
    {
        if (!_isCapturing)
        {
            throw new InvalidOperationException("Capture not started.");
        }

        while (_isCapturing && !cancellationToken.IsCancellationRequested)
        {
            if (SimulateOverflow)
            {
                BufferOverflow?.Invoke(new AudioBufferOverflowEvent(DateTime.UtcNow, 80000, 16000));
            }

            byte[] dummyPcm = new byte[3200];
            yield return new AudioFrame(dummyPcm, DateTime.UtcNow, 16000, 16, 1);

            try
            {
                await Task.Delay(10, cancellationToken);
            }
            catch (TaskCanceledException)
            {
                break;
            }
        }
    }
}
