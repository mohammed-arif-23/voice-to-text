namespace Desktop.Core;

public enum DictationState
{
    SignedOut,
    Idle,
    Arming,
    Capturing,
    Streaming,
    Finalizing,
    ReviewRequired,
    ReadyToInsert,
    ValidatingTarget,
    Inserting,
    Verifying,
    Completed,
    Cancelled,
    RecoverableFailure,
    FatalFailure,
    Offline
}
