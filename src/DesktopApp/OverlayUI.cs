using System;
using Desktop.Core;

namespace Desktop.UI;

public class OverlayViewModel
{
    public DictationState CurrentState { get; set; } = DictationState.Idle;
    public string TranscriptText { get; set; } = "";
    public string TextColor { get; set; } = "Black";
    public string LayoutMode { get; set; } = "Normal";

    public void UpdateState(DictationState state)
    {
        CurrentState = state;
        TextColor = state switch
        {
            DictationState.Idle => "Gray",
            DictationState.Arming => "Orange",
            DictationState.Capturing => "Red",
            DictationState.FatalFailure => "DarkRed",
            _ => "Black"
        };
    }

    public void AppendText(string newText)
    {
        TranscriptText += newText;
        if (TranscriptText.Length > 300)
        {
            TranscriptText = TranscriptText[^200..];
        }
    }
}

public class OverlayWindow
{
    public int WindowStyles { get; set; }
    public bool Focusable { get; set; } = false;
    public double X { get; set; }
    public double Y { get; set; }
    public double Width { get; set; } = 400;
    public double Height { get; set; } = 150;
    public bool IsHighContrast { get; set; }
    public bool IsAltTabVisible { get; set; } = true;

    public const int WS_EX_NOACTIVATE = 0x08000000;
    public const int WS_EX_TOOLWINDOW = 0x00000080;
    public const int WM_MOUSEACTIVATE = 0x0021;
    public const int MA_NOACTIVATE = 4;

    public OverlayWindow()
    {
        X = 1500;
        Y = 900;
    }

    public void ApplyWin32Styles()
    {
        WindowStyles |= WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW;
        IsAltTabVisible = false;
    }

    public int OnMouseActivate()
    {
        return MA_NOACTIVATE;
    }
}
