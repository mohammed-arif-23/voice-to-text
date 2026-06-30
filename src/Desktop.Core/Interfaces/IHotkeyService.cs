using System;

namespace Desktop.Core;

public interface IHotkeyService
{
    void Register(int vkCode, int modifiers, Action onPress, Action onRelease, bool isToggle);
    void UnregisterAll();
}
