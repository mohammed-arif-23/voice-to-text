#nullable disable
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Interop;
using System.Windows.Media;
using Desktop.Core;
using Desktop.Insertion;
using Desktop.Targeting;
using Desktop.Transcription;

namespace DesktopApp;

public partial class MainWindow : Window
{
    // ──────────────────────────────────────────────
    // Win32 / WM_HOTKEY plumbing
    // ──────────────────────────────────────────────
    private const int WS_EX_NOACTIVATE = 0x08000000;
    private const int WS_EX_TOOLWINDOW = 0x00000080;
    private const int GWL_EXSTYLE = -20;
    private const int WM_HOTKEY = 0x0312;

    [DllImport("user32.dll", SetLastError = true)]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static extern int GetWindowLong(IntPtr hWnd, int nIndex);

    [DllImport("user32.dll", SetLastError = true)]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static extern int SetWindowLong(IntPtr hWnd, int nIndex, int dwNewLong);

    [DllImport("user32.dll", SetLastError = true)]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static extern bool RegisterHotKey(IntPtr hWnd, int id, uint fsModifiers, uint vk);

    [DllImport("user32.dll", SetLastError = true)]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static extern bool UnregisterHotKey(IntPtr hWnd, int id);

    // Hotkey IDs
    private const int HOTKEY_CAPSLOCK = 9000;
    private const int HOTKEY_CTRL_ALT_D = 9001;
    private const int HOTKEY_F9 = 9002;
    private const int HOTKEY_F10 = 9003;

    // Virtual key codes
    private const uint VK_CAPITAL = 0x14;
    private const uint VK_D = 0x44;
    private const uint VK_F9 = 0x78;
    private const uint VK_F10 = 0x79;
    private const uint MOD_ALT = 0x0001;
    private const uint MOD_CONTROL = 0x0002;

    // ──────────────────────────────────────────────
    // Services
    // ──────────────────────────────────────────────
    private readonly WindowsSpeechTranscriptionProvider? _speechProvider;
    private readonly TargetContextService? _targetContextService;
    private readonly InsertionAdapterChain? _insertionChain;
    private readonly TranscriptReconciler? _reconciler;

    // State
    private volatile bool _isDictating;
    private TargetContext? _capturedContext;

    // ──────────────────────────────────────────────
    // Constructor
    // ──────────────────────────────────────────────
    public MainWindow()
    {
        try
        {
            InitializeComponent();

            // Windows SAPI — completely free, offline, no API key
            _speechProvider = new WindowsSpeechTranscriptionProvider();
            _speechProvider.SegmentReceived += OnSegmentReceived;
            _speechProvider.RecognitionCompleted += OnRecognitionCompleted;
            _speechProvider.ErrorOccurred += OnSpeechError;

            _targetContextService = new TargetContextService();
            _reconciler = new TranscriptReconciler();

            var adapters = new List<ITextInsertionAdapter>
            {
                new UiaValuePatternAdapter(),   // UI Automation (most apps)
                new SendInputAdapter(),          // Raw keyboard simulation
                new ClipboardFallbackAdapter()   // Clipboard + Ctrl+V (ultimate fallback)
            };
            _insertionChain = new InsertionAdapterChain(adapters);

            Loaded += OnLoaded;
            Closing += OnClosing;
        }
        catch (Exception ex)
        {
            LogError("INIT", ex);
            MessageBox.Show($"Startup error: {ex.Message}\nSee desktop_app_error.txt for details.");
            Application.Current.Shutdown();
        }
    }

    // ──────────────────────────────────────────────
    // Window loaded
    // ──────────────────────────────────────────────
    private void OnLoaded(object sender, RoutedEventArgs e)
    {
        try
        {
            // Position bottom-right, above the taskbar
            var workArea = SystemParameters.WorkArea;
            Left = workArea.Right - Width - 16;
            Top = workArea.Bottom - Height - 16;

            var helper = new WindowInteropHelper(this);
            IntPtr hwnd = helper.Handle;

            // Prevent overlay from stealing focus
            int exStyle = GetWindowLong(hwnd, GWL_EXSTYLE);
            SetWindowLong(hwnd, GWL_EXSTYLE, exStyle | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW);

            // Hook window messages so we receive WM_HOTKEY
            HwndSource source = HwndSource.FromHwnd(hwnd);
            source.AddHook(HwndMessageHook);

            // Register hotkeys (all are independent toggles)
            RegisterHotKey(hwnd, HOTKEY_CAPSLOCK, 0, VK_CAPITAL);
            RegisterHotKey(hwnd, HOTKEY_CTRL_ALT_D, MOD_CONTROL | MOD_ALT, VK_D);
            RegisterHotKey(hwnd, HOTKEY_F9, 0, VK_F9);
            RegisterHotKey(hwnd, HOTKEY_F10, 0, VK_F10);

            SetIdle("Press F9 (or Caps Lock / Ctrl+Alt+D) to dictate");
        }
        catch (Exception ex)
        {
            LogError("ONLOADED", ex);
            MessageBox.Show($"Load error: {ex.Message}\nSee desktop_app_error.txt for details.");
            Application.Current.Shutdown();
        }
    }

    // ──────────────────────────────────────────────
    // Hotkey handler — runs on UI thread
    // ──────────────────────────────────────────────
    private IntPtr HwndMessageHook(IntPtr hwnd, int msg, IntPtr wParam, IntPtr lParam, ref bool handled)
    {
        if (msg != WM_HOTKEY) return IntPtr.Zero;

        int id = wParam.ToInt32();
        bool isOurHotkey = id == HOTKEY_CAPSLOCK || id == HOTKEY_CTRL_ALT_D
                         || id == HOTKEY_F9     || id == HOTKEY_F10;
        if (!isOurHotkey) return IntPtr.Zero;

        handled = true;

        if (_isDictating)
        {
            // User pressed hotkey again to cancel manually
            StopDictation(insertText: false);
        }
        else
        {
            // ⚡ Capture BEFORE the overlay activates — this is the text field the user wants
            try { _capturedContext = _targetContextService.CaptureContext(); }
            catch { _capturedContext = null; }

            StartDictation();
        }

        return IntPtr.Zero;
    }

    // ──────────────────────────────────────────────
    // Start dictation
    // ──────────────────────────────────────────────
    private void StartDictation()
    {
        if (_isDictating) return;
        _isDictating = true;
        _reconciler.Clear();

        SetCapturing();

        // Kick off SAPI on a background thread (non-blocking)
        Task.Run(() =>
        {
            try
            {
                _speechProvider.StartListening(silenceSeconds: 2);
            }
            catch (Exception ex)
            {
                // Most common: no microphone configured in Windows
                Dispatcher.Invoke(() => HandleError("Microphone error: " + ex.Message));
            }
        });
    }

    // ──────────────────────────────────────────────
    // Stop dictation (called from either UI or background thread)
    // ──────────────────────────────────────────────
    private void StopDictation(bool insertText)
    {
        if (!_isDictating) return;
        _isDictating = false;

        _speechProvider.StopListening();

        if (!insertText)
        {
            Dispatcher.Invoke(() => SetIdle("Cancelled."));
            Task.Delay(1500).ContinueWith(_ =>
                Dispatcher.Invoke(() => SetIdle("Press F9 (or Caps Lock / Ctrl+Alt+D) to dictate")));
            return;
        }

        // Insert on a background thread — never block the UI
        Task.Run(async () =>
        {
            try
            {
                string text = _reconciler.GetReconciledText();

                if (string.IsNullOrWhiteSpace(text))
                {
                    Dispatcher.Invoke(() => SetIdle("Nothing heard. Try again."));
                    await Task.Delay(2000);
                    Dispatcher.Invoke(() => SetIdle("Press F9 (or Caps Lock / Ctrl+Alt+D) to dictate"));
                    return;
                }

                Dispatcher.Invoke(() =>
                {
                    StatusIndicator.Fill = new SolidColorBrush(Color.FromRgb(48, 209, 88)); // Green
                    StatusLabel.Text = "INSERTING";
                    TranscriptText.Text = text;
                });

                // Small delay so Windows focus has time to return to the original window
                await Task.Delay(150);

                bool inserted = false;
                if (_capturedContext != null)
                {
                    var enabledAdapters = new List<AdapterKind>
                    {
                        AdapterKind.UiaValuePattern,
                        AdapterKind.SendInput,
                        AdapterKind.ClipboardFallback
                    };

                    try
                    {
                        var result = await _insertionChain.ExecuteAsync(text, _capturedContext, enabledAdapters);
                        inserted = result.Success;
                    }
                    catch (Exception ex)
                    {
                        LogError("INSERT", ex);
                    }
                }

                if (!inserted)
                {
                    // Ultimate fallback: put text in clipboard and Ctrl+V
                    await TryClipboardPaste(text);
                }

                Dispatcher.Invoke(() => SetIdle("Done! Press F9 to dictate again."));
                await Task.Delay(3000);
                Dispatcher.Invoke(() => SetIdle("Press F9 (or Caps Lock / Ctrl+Alt+D) to dictate"));
            }
            catch (Exception ex)
            {
                LogError("STOP_FLOW", ex);
                Dispatcher.Invoke(() => HandleError(ex.Message));
            }
        });
    }

    // ──────────────────────────────────────────────
    // SAPI event handlers
    // ──────────────────────────────────────────────

    /// Called for interim (partial) results — update UI live
    private void OnSegmentReceived(TranscriptSegment segment)
    {
        _reconciler.AddSegment(segment);
        string current = _reconciler.GetReconciledText();

        // For interim, show a preview including the partial word
        string display = segment.Kind == SegmentKind.Interim
            ? (string.IsNullOrEmpty(current) ? segment.Text : current + " " + segment.Text).Trim()
            : current;

        Dispatcher.Invoke(() =>
        {
            TranscriptText.Text = string.IsNullOrWhiteSpace(display) ? "Listening..." : display;
        });
    }

    /// Called when SAPI silence-timeout fires — this is the auto-stop
    private void OnRecognitionCompleted()
    {
        // Guard: only act if we are still in dictation mode
        if (!_isDictating) return;
        StopDictation(insertText: true);
    }

    private void OnSpeechError(Exception ex)
    {
        LogError("SPEECH", ex);
        Dispatcher.Invoke(() => HandleError("Speech recognition error: " + ex.Message));
    }

    // ──────────────────────────────────────────────
    // Clipboard paste fallback (no dependencies)
    // ──────────────────────────────────────────────
    [DllImport("user32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static extern void keybd_event(byte bVk, byte bScan, uint dwFlags, int dwExtraInfo);

    private const byte VK_CONTROL = 0x11;
    private const byte VK_V = 0x56;
    private const uint KEYEVENTF_KEYUP = 0x0002;

    private static async Task TryClipboardPaste(string text)
    {
        try
        {
            // Must set clipboard on STA thread
            var tcs = new TaskCompletionSource<bool>();
            var sta = new System.Threading.Thread(() =>
            {
                try { System.Windows.Clipboard.SetText(text); tcs.SetResult(true); }
                catch { tcs.SetResult(false); }
            });
            sta.SetApartmentState(System.Threading.ApartmentState.STA);
            sta.Start();
            await tcs.Task;

            await Task.Delay(80);

            // Simulate Ctrl+V
            keybd_event(VK_CONTROL, 0, 0, 0);
            keybd_event(VK_V, 0, 0, 0);
            keybd_event(VK_V, 0, KEYEVENTF_KEYUP, 0);
            keybd_event(VK_CONTROL, 0, KEYEVENTF_KEYUP, 0);
        }
        catch { /* ignore */ }
    }

    // ──────────────────────────────────────────────
    // UI state helpers
    // ──────────────────────────────────────────────
    private void SetCapturing()
    {
        Dispatcher.Invoke(() =>
        {
            StatusIndicator.Fill = new SolidColorBrush(Color.FromRgb(255, 59, 48)); // Red
            StatusLabel.Text = "LISTENING";
            TranscriptText.Text = "Listening...";
        });
    }

    private void SetIdle(string message = "Press F9 (or Caps Lock / Ctrl+Alt+D) to dictate")
    {
        StatusIndicator.Fill = new SolidColorBrush(Color.FromRgb(142, 142, 147)); // Gray
        StatusLabel.Text = "IDLE";
        TranscriptText.Text = message;
    }

    private void HandleError(string message)
    {
        _isDictating = false;
        StatusIndicator.Fill = new SolidColorBrush(Color.FromRgb(255, 149, 0)); // Orange
        StatusLabel.Text = "ERROR";
        TranscriptText.Text = message;

        Task.Delay(4000).ContinueWith(_ =>
            Dispatcher.Invoke(() => SetIdle("Press F9 (or Caps Lock / Ctrl+Alt+D) to dictate")));
    }

    private static void LogError(string context, Exception ex)
    {
        try
        {
            System.IO.File.WriteAllText(
                "desktop_app_error.txt",
                $"[{context}] {DateTime.Now:HH:mm:ss}\r\n{ex}\r\n");
        }
        catch { /* ignore */ }
    }

    // ──────────────────────────────────────────────
    // Window closing
    // ──────────────────────────────────────────────
    private void OnClosing(object? sender, System.ComponentModel.CancelEventArgs e)
    {
        var helper = new WindowInteropHelper(this);
        UnregisterHotKey(helper.Handle, HOTKEY_CAPSLOCK);
        UnregisterHotKey(helper.Handle, HOTKEY_CTRL_ALT_D);
        UnregisterHotKey(helper.Handle, HOTKEY_F9);
        UnregisterHotKey(helper.Handle, HOTKEY_F10);
        _speechProvider.Dispose();
    }
}
