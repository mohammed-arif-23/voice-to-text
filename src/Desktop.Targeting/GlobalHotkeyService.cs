using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Desktop.Core;

namespace Desktop.Targeting;

public class GlobalHotkeyService : IHotkeyService, IDisposable
{
    private class HotkeyRegistration
    {
        public int VkCode { get; set; }
        public int Modifiers { get; set; }
        public Action OnPress { get; set; } = null!;
        public Action OnRelease { get; set; } = null!;
        public bool IsToggle { get; set; }
    }

    private readonly List<HotkeyRegistration> _registrations = new();
    public HashSet<int> Conflicts { get; } = new();
    public bool Disposed { get; private set; }

    public void Register(int vkCode, int modifiers, Action onPress, Action onRelease, bool isToggle)
    {
        if (vkCode == 0) throw new ArgumentException("VK Code cannot be zero.");

        if (Conflicts.Contains(vkCode))
        {
            throw new HotkeyConflictException(vkCode, modifiers);
        }

        _registrations.Add(new HotkeyRegistration
        {
            VkCode = vkCode,
            Modifiers = modifiers,
            OnPress = onPress,
            OnRelease = onRelease,
            IsToggle = isToggle
        });
    }

    public void UnregisterAll()
    {
        _registrations.Clear();
    }

    public void TriggerPress(int vkCode)
    {
        var reg = _registrations.FirstOrDefault(r => r.VkCode == vkCode);
        if (reg != null)
        {
            Task.Run(() => reg.OnPress());
        }
    }

    public void TriggerRelease(int vkCode)
    {
        var reg = _registrations.FirstOrDefault(r => r.VkCode == vkCode);
        if (reg != null)
        {
            Task.Run(() => reg.OnRelease());
        }
    }

    public void Dispose()
    {
        Disposed = true;
        UnregisterAll();
        GC.SuppressFinalize(this);
    }
}
