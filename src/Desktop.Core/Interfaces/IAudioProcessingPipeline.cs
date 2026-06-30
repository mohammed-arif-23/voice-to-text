using System;

namespace Desktop.Core;

public interface IAudioProcessingPipeline
{
    void ProcessFrame(AudioFrame frame);
    event EventHandler<AudioFrameEventArgs>? OnFrameProcessed;
}
