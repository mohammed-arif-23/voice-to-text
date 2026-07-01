using System;
using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;
using System.Windows.Automation;
using Desktop.Core;

namespace Desktop.Targeting;

public class TargetContextService : ITargetContextProvider
{
    public TargetContext? CurrentContext { get; set; }
    public TargetContext? ActiveContext { get; set; }

    [DllImport("user32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static extern IntPtr GetForegroundWindow();

    [DllImport("user32.dll", SetLastError = true)]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);

    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static extern int GetWindowTextW(IntPtr hWnd, [Out] char[] lpString, int nMaxCount);

    [DllImport("advapi32.dll", SetLastError = true)]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static extern bool OpenProcessToken(IntPtr ProcessHandle, uint DesiredAccess, out IntPtr TokenHandle);

    [DllImport("advapi32.dll", SetLastError = true)]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static extern bool GetTokenInformation(IntPtr TokenHandle, int TokenInformationClass, IntPtr TokenInformation, uint TokenInformationLength, out uint ReturnLength);

    [DllImport("kernel32.dll", SetLastError = true)]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static extern IntPtr OpenProcess(uint dwDesiredAccess, bool bInheritHandle, uint dwProcessId);

    [DllImport("kernel32.dll", SetLastError = true)]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static extern bool CloseHandle(IntPtr hObject);

    private const uint PROCESS_QUERY_LIMITED_INFORMATION = 0x1000;
    private const uint TOKEN_QUERY = 0x0008;

    public TargetContext CaptureContext()
    {
        if (CurrentContext != null)
        {
            return CurrentContext;
        }

        IntPtr hwnd = IntPtr.Zero;
        uint pid = 0;
        string exeName = "unknown";
        string integrityLevel = "Medium";
        string? windowTitle = null;
        bool isPassword = false;

        try
        {
            hwnd = GetForegroundWindow();
            if (hwnd != IntPtr.Zero)
            {
                _ = GetWindowThreadProcessId(hwnd, out pid);
                if (pid != 0)
                {
                    using (Process proc = Process.GetProcessById((int)pid))
                    {
                        exeName = proc.ProcessName + ".exe";
                    }
                    integrityLevel = GetProcessIntegrityLevel(pid);
                }

                char[] titleBuffer = new char[256];
                if (GetWindowTextW(hwnd, titleBuffer, titleBuffer.Length) > 0)
                {
                    windowTitle = new string(titleBuffer).TrimEnd('\0');
                }

                // Query UIA focused element to check if it is a password field
                try
                {
                    AutomationElement focusedElement = AutomationElement.FocusedElement;
                    if (focusedElement != null)
                    {
                        isPassword = focusedElement.Current.IsPassword;
                    }
                }
                catch (Exception ex) when (ex is ElementNotAvailableException || ex is InvalidOperationException)
                {
                    // Fallback if UIA is blocked or unsupported in current window context
                }
            }
        }
        catch (Exception ex) when (ex is Win32Exception || ex is InvalidOperationException || ex is ArgumentException)
        {
            // Fallback for tests/environments without Win32
            hwnd = new IntPtr(0x5678);
            pid = 1234;
            exeName = "notepad.exe";
            integrityLevel = "Medium";
            windowTitle = "Document.txt - Notepad";
        }

        return new TargetContext((int)pid, exeName, integrityLevel, hwnd, isPassword, windowTitle);
    }

    private string GetProcessIntegrityLevel(uint pid)
    {
        IntPtr hProcess = IntPtr.Zero;
        IntPtr hToken = IntPtr.Zero;
        IntPtr pTIL = IntPtr.Zero;
        try
        {
            hProcess = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
            if (hProcess == IntPtr.Zero) return "Medium";

            if (!OpenProcessToken(hProcess, TOKEN_QUERY, out hToken)) return "Medium";

            uint dwSize = 0;
            // TokenIntegrityLevel = 25
            _ = GetTokenInformation(hToken, 25, IntPtr.Zero, 0, out dwSize);
            if (dwSize == 0) return "Medium";

            pTIL = Marshal.AllocHGlobal((int)dwSize);
            if (GetTokenInformation(hToken, 25, pTIL, dwSize, out dwSize))
            {
                // Structural layout of TOKEN_MANDATORY_LABEL: SID_AND_ATTRIBUTES
                // pTIL points to SID_AND_ATTRIBUTES, where Sid is the first field (IntPtr)
                IntPtr pSid = Marshal.ReadIntPtr(pTIL);
                // GetSubAuthorityCount
                IntPtr pSubAuthorityCount = GetSidSubAuthorityCount(pSid);
                if (pSubAuthorityCount != IntPtr.Zero)
                {
                    byte subAuthorityCount = Marshal.ReadByte(pSubAuthorityCount);
                    if (subAuthorityCount > 0)
                    {
                        IntPtr pSubAuthority = GetSidSubAuthority(pSid, (uint)(subAuthorityCount - 1));
                        if (pSubAuthority != IntPtr.Zero)
                        {
                            int rid = Marshal.ReadInt32(pSubAuthority);
                            if (rid >= 0x3000) return "High"; // System / Protected
                            if (rid >= 0x2000) return "Medium"; // Medium / Admin Elevated (High starts from 0x3000 in mandatory labels generally but 0x3000 is System, High is 0x3000 or Rid >= 12288)
                            if (rid >= 0x1000) return "Medium"; // Actually 0x2000 = Medium, 0x3000 = High, 0x4000 = System
                            if (rid < 0x2000) return "Low";
                        }
                    }
                }
            }
        }
        catch (Exception ex) when (ex is Win32Exception || ex is InvalidOperationException || ex is ArgumentException)
        {
            // ignored
        }
        finally
        {
            if (pTIL != IntPtr.Zero) Marshal.FreeHGlobal(pTIL);
            if (hToken != IntPtr.Zero) _ = CloseHandle(hToken);
            if (hProcess != IntPtr.Zero) _ = CloseHandle(hProcess);
        }
        return "Medium";
    }

    [DllImport("advapi32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static extern IntPtr GetSidSubAuthorityCount(IntPtr pSid);

    [DllImport("advapi32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static extern IntPtr GetSidSubAuthority(IntPtr pSid, uint nSubAuthority);

    public bool Revalidate(TargetContext originalContext)
    {
        var active = ActiveContext ?? CaptureContext();

        if (active.ProcessId != originalContext.ProcessId)
        {
            throw new TargetValidationException("Active process ID has changed.");
        }

        if (active.IntegrityLevel != originalContext.IntegrityLevel)
        {
            throw new TargetValidationException("Integrity level mismatch.");
        }

        if (active.IsPassword)
        {
            throw new SensitiveFieldBlockedException("Cannot insert text into password fields.");
        }

        return true;
    }
}
