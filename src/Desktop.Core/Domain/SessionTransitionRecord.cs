using System;

namespace Desktop.Core;

public record SessionTransitionRecord(
    DictationState FromState,
    DictationState ToState,
    DateTime Timestamp
);
