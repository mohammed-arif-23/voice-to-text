using System;

namespace Desktop.Core;

public class AudioFrameEventArgs : EventArgs
{
    public AudioFrame Frame { get; }

    public AudioFrameEventArgs(AudioFrame frame)
    {
        Frame = frame;
    }
}
