using System;

namespace Desktop.Core;

public class DictationException : Exception
{
    public string DiagnosticCode { get; } = "DICTATION_ERROR";

    public DictationException()
        : base("A dictation error occurred.")
    {
    }

    public DictationException(string message)
        : base(message)
    {
    }

    public DictationException(string message, Exception? innerException)
        : base(message, innerException)
    {
    }

    public DictationException(string message, string diagnosticCode, Exception? innerException = null)
        : base(message, innerException)
    {
        DiagnosticCode = diagnosticCode;
    }
}

public class InvalidSessionTransitionException : DictationException
{
    public DictationState FromState { get; }
    public DictationState ToState { get; }

    public InvalidSessionTransitionException(DictationState fromState, DictationState toState)
        : base($"Invalid session state transition from {fromState} to {toState}.", "INVALID_SESSION_TRANSITION")
    {
        FromState = fromState;
        ToState = toState;
    }

    public InvalidSessionTransitionException()
        : base("Invalid session transition occurred.", "INVALID_SESSION_TRANSITION")
    {
    }



    public InvalidSessionTransitionException(string message) 
        : base(message, "INVALID_SESSION_TRANSITION")
    {
    }

    public InvalidSessionTransitionException(string message, Exception? innerException) 
        : base(message, "INVALID_SESSION_TRANSITION", innerException)
    {
    }
}

public class TargetValidationException : DictationException
{
    public TargetValidationException()
        : base("Target window or automation validation failed.", "TARGET_VALIDATION_FAILED")
    {
    }

    public TargetValidationException(string message)
        : base(message, "TARGET_VALIDATION_FAILED")
    {
    }

    public TargetValidationException(string message, Exception? innerException)
        : base(message, "TARGET_VALIDATION_FAILED", innerException)
    {
    }

    protected TargetValidationException(string message, string diagnosticCode)
        : base(message, diagnosticCode)
    {
    }

    protected TargetValidationException(string message, string diagnosticCode, Exception? innerException)
        : base(message, diagnosticCode, innerException)
    {
    }
}

public class SensitiveFieldBlockedException : TargetValidationException
{
    public SensitiveFieldBlockedException()
        : base("Sensitive field is blocked from input.", "SENSITIVE_FIELD_BLOCKED")
    {
    }

    public SensitiveFieldBlockedException(string message)
        : base(message, "SENSITIVE_FIELD_BLOCKED")
    {
    }

    public SensitiveFieldBlockedException(string message, Exception? innerException)
        : base(message, "SENSITIVE_FIELD_BLOCKED", innerException)
    {
    }
}

public class ProviderAuthException : DictationException
{
    public ProviderAuthException()
        : base("Authentication with transcription provider failed.", "PROVIDER_AUTH_FAILED")
    {
    }

    public ProviderAuthException(string message)
        : base(message, "PROVIDER_AUTH_FAILED")
    {
    }

    public ProviderAuthException(string message, Exception? innerException)
        : base(message, "PROVIDER_AUTH_FAILED", innerException)
    {
    }
}

public class AudioCaptureException : DictationException
{
    public new int DiagnosticCode { get; }

    public AudioCaptureException()
        : base("Audio capture device failed or was disconnected.", "AUDIO_CAPTURE_FAILED")
    {
    }

    public AudioCaptureException(string message)
        : base(message, "AUDIO_CAPTURE_FAILED")
    {
    }

    public AudioCaptureException(string message, Exception? innerException)
        : base(message, "AUDIO_CAPTURE_FAILED", innerException)
    {
    }

    public AudioCaptureException(string message, int diagnosticCode)
        : base(message)
    {
        DiagnosticCode = diagnosticCode;
    }
}

public class InsertionFailedException : DictationException
{
    public InsertionFailedException()
        : base("Failed to insert text into the target context.", "INSERTION_FAILED")
    {
    }

    public InsertionFailedException(string message)
        : base(message, "INSERTION_FAILED")
    {
    }

    public InsertionFailedException(string message, Exception? innerException)
        : base(message, "INSERTION_FAILED", innerException)
    {
    }
}

public class OfflineModelNotFoundException : DictationException
{
    public string ModelPath { get; } = "";

    public OfflineModelNotFoundException(string path)
        : base($"Offline Whisper model not found at: {path}", "OFFLINE_MODEL_NOT_FOUND")
    {
        ModelPath = path;
    }

    public OfflineModelNotFoundException()
        : base("Offline Whisper model not found.", "OFFLINE_MODEL_NOT_FOUND")
    {
    }


    public OfflineModelNotFoundException(string message, Exception? innerException)
        : base(message, "OFFLINE_MODEL_NOT_FOUND", innerException)
    {
    }
}

public class HotkeyConflictException : DictationException
{
    public int VirtualKeyCode { get; }
    public int Modifiers { get; }

    public HotkeyConflictException(int vkCode, int modifiers)
        : base($"Hotkey registration conflict for VK code {vkCode} with modifiers {modifiers}.", "HOTKEY_CONFLICT")
    {
        VirtualKeyCode = vkCode;
        Modifiers = modifiers;
    }

    public HotkeyConflictException()
        : base("Hotkey registration conflict.", "HOTKEY_CONFLICT")
    {
    }

    public HotkeyConflictException(string message)
        : base(message, "HOTKEY_CONFLICT")
    {
    }

    public HotkeyConflictException(string message, Exception? innerException)
        : base(message, "HOTKEY_CONFLICT", innerException)
    {
    }
}
