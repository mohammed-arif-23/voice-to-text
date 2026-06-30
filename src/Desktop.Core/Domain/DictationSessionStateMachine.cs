using System;
using System.Collections.Generic;
using System.Linq;

namespace Desktop.Core;

public class DictationSessionStateMachine
{
    private readonly object _lock = new();
    private DictationState _currentState = DictationState.Idle;
    private readonly List<SessionTransitionRecord> _history = new();

    public DictationState CurrentState
    {
        get { lock (_lock) return _currentState; }
    }

    public IReadOnlyList<SessionTransitionRecord> TransitionHistory
    {
        get { lock (_lock) return _history.ToList(); }
    }

    public event Action<SessionTransitionRecord>? StateChanged;

    public void TransitionTo(DictationState nextState)
    {
        lock (_lock)
        {
            if (!IsValidTransition(_currentState, nextState))
            {
                throw new InvalidSessionTransitionException(_currentState, nextState);
            }

            var record = new SessionTransitionRecord(_currentState, nextState, DateTime.UtcNow);
            _currentState = nextState;
            _history.Add(record);
            StateChanged?.Invoke(record);
        }
    }

    public static bool IsValidTransition(DictationState from, DictationState to)
    {
        if (from == to) return false;

        return from switch
        {
            DictationState.SignedOut => to == DictationState.Idle,
            DictationState.Idle => to is DictationState.Arming or DictationState.Offline or DictationState.SignedOut,
            DictationState.Arming => to is DictationState.Capturing or DictationState.Cancelled or DictationState.FatalFailure,
            DictationState.Capturing => to is DictationState.Streaming or DictationState.Finalizing or DictationState.Cancelled or DictationState.RecoverableFailure,
            DictationState.Streaming => to is DictationState.Finalizing or DictationState.Cancelled or DictationState.RecoverableFailure or DictationState.Offline,
            DictationState.Finalizing => to is DictationState.ReadyToInsert or DictationState.ReviewRequired or DictationState.RecoverableFailure,
            DictationState.ReviewRequired => to is DictationState.ReadyToInsert or DictationState.Cancelled,
            DictationState.ReadyToInsert => to is DictationState.ValidatingTarget or DictationState.Cancelled,
            DictationState.ValidatingTarget => to is DictationState.Inserting or DictationState.FatalFailure or DictationState.Cancelled,
            DictationState.Inserting => to is DictationState.Verifying or DictationState.FatalFailure,
            DictationState.Verifying => to is DictationState.Completed or DictationState.RecoverableFailure,
            DictationState.Completed => to == DictationState.Idle,
            DictationState.Cancelled => to == DictationState.Idle,
            DictationState.RecoverableFailure => to == DictationState.Idle,
            DictationState.FatalFailure => to is DictationState.Idle or DictationState.SignedOut,
            DictationState.Offline => to is DictationState.Finalizing or DictationState.Idle,
            _ => false
        };
    }
}
