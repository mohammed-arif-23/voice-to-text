using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using System.Threading;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Interop;
using System.Windows.Media;
using Desktop.Audio;
using Desktop.Core;
using Desktop.Insertion;
using Desktop.Targeting;
using Desktop.Transcription;

namespace DesktopApp;

public partial class MainWindow : Window
{
    private const int WS_EX_NOACTIVATE = 0x08000000;
    private const int WS_EX_TOOLWINDOW = 0x00000080;
    private const int GWL_EXSTYLE = -20;

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

    private const int HOTKEY_ID = 9000;
    private const uint VK_CAPITAL = 0x14; // Caps Lock

    private readonly WasapiAudioCaptureService _audioCapture;
    private readonly DeepgramTranscriptionProvider _transcriptionProvider;
    private readonly TargetContextService _targetContextService;
    private readonly InsertionAdapterChain _insertionChain;
    private readonly TranscriptReconciler _reconciler;
    private readonly DictationSessionStateMachine _stateMachine;
    private CancellationTokenSource? _captureCts;

    private TargetContext? _capturedContext;
    private bool _isDictating;

    public MainWindow()
    {
        InitializeComponent();

        _stateMachine = new DictationSessionStateMachine();
        _audioCapture = new WasapiAudioCaptureService();
        _transcriptionProvider = new DeepgramTranscriptionProvider();
        _targetContextService = new TargetContextService();
        _reconciler = new TranscriptReconciler();

        var adapters = new List<ITextInsertionAdapter>
        {
            new BrowserExtensionAdapter { ExtensionConnected = false },
            new UiaValuePatternAdapter(),
            new SendInputAdapter(),
            new ClipboardFallbackAdapter()
        };
        _insertionChain = new InsertionAdapterChain(adapters);

        _transcriptionProvider.SegmentReceived += OnSegmentReceived;
        _transcriptionProvider.ErrorOccurred += OnTranscriptionError;
        _audioCapture.BufferOverflow += OnAudioOverflow;

        Loaded += OnLoaded;
        Closing += OnClosing;
    }

    private void OnLoaded(object sender, RoutedEventArgs e)
    {
        // Position window near System Tray (bottom right)
        var desktopWorkingArea = SystemParameters.WorkArea;
        Left = desktopWorkingArea.Right - Width - 10;
        Top = desktopWorkingArea.Bottom - Height - 10;

        // Apply WS_EX_NOACTIVATE to prevent focus stealing
        var helper = new WindowInteropHelper(this);
        int exStyle = GetWindowLong(helper.Handle, GWL_EXSTYLE);
        SetWindowLong(helper.Handle, GWL_EXSTYLE, exStyle | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW);

        // Bind hotkey
        HwndSource source = HwndSource.FromHwnd(helper.Handle);
        source.AddHook(HwndMessageHook);

        RegisterCapslockHotkey(helper.Handle);
    }

    private void RegisterCapslockHotkey(IntPtr hwnd)
    {
        // Register VK_CAPITAL (Caps Lock) without modifiers
        RegisterHotKey(hwnd, HOTKEY_ID, 0, VK_CAPITAL);
    }

    private IntPtr HwndMessageHook(IntPtr hwnd, int msg, IntPtr wParam, IntPtr lParam, ref bool handled)
    {
        const int WM_HOTKEY = 0x0312;

        if (msg == WM_HOTKEY && wParam.ToInt32() == HOTKEY_ID)
        {
            if (!_isDictating)
            {
                StartDictationFlow();
            }
            else
            {
                StopDictationFlow();
            }
            handled = true;
        }

        return IntPtr.Zero;
    }

    private void StartDictationFlow()
    {
        _isDictating = true;
        _reconciler.Clear();

        try
        {
            // Capture focus context
            _capturedContext = _targetContextService.CaptureContext();

            // Set UI State
            Dispatcher.Invoke(() =>
            {
                StatusIndicator.Fill = new SolidColorBrush(Color.FromRgb(255, 59, 48)); // Red
                StatusLabel.Text = "CAPTURING";
                TranscriptText.Text = "Listening...";
            });

            _stateMachine.TransitionTo(DictationState.Arming);
            _audioCapture.StartCapture("Default Mic");
            _stateMachine.TransitionTo(DictationState.Capturing);

            // Connect Deepgram
            _stateMachine.TransitionTo(DictationState.Streaming);
            _transcriptionProvider.ConnectAsync("dg_valid_key", CancellationToken.None).GetAwaiter().GetResult();

            _captureCts = new CancellationTokenSource();
            Task.Run(() => StreamAudioAsync(_captureCts.Token));
        }
        catch (Exception ex)
        {
            HandleFailure(ex.Message);
        }
    }

    private async Task StreamAudioAsync(CancellationToken token)
    {
        try
        {
            await foreach (var frame in _audioCapture.StreamFramesAsync(token))
            {
                await _transcriptionProvider.SendAudioAsync(frame, token);
            }
        }
        catch (Exception)
        {
            // Stream stopped or cancelled
        }
    }

    private void StopDictationFlow()
    {
        _isDictating = false;
        _captureCts?.Cancel();

        try
        {
            _stateMachine.TransitionTo(DictationState.Finalizing);
            _audioCapture.StopCapture();
            _transcriptionProvider.DisconnectAsync().GetAwaiter().GetResult();

            string textToInsert = _reconciler.GetReconciledText();

            if (string.IsNullOrWhiteSpace(textToInsert))
            {
                ResetToIdle("No speech detected.");
                return;
            }

            _stateMachine.TransitionTo(DictationState.ReadyToInsert);

            // Revalidate Target Context
            if (_capturedContext != null)
            {
                _stateMachine.TransitionTo(DictationState.ValidatingTarget);
                _targetContextService.Revalidate(_capturedContext);

                // Insert Text
                _stateMachine.TransitionTo(DictationState.Inserting);
                var enabledAdapters = new List<AdapterKind>
                {
                    AdapterKind.BrowserExtension,
                    AdapterKind.UiaValuePattern,
                    AdapterKind.SendInput,
                    AdapterKind.ClipboardFallback
                };

                _stateMachine.TransitionTo(DictationState.Verifying);
                var result = _insertionChain.ExecuteAsync(textToInsert, _capturedContext, enabledAdapters).GetAwaiter().GetResult();

                if (result.Success)
                {
                    _stateMachine.TransitionTo(DictationState.Completed);
                    ResetToIdle("Text inserted!");
                }
                else
                {
                    throw new InsertionFailedException("Target insertion chain failed.");
                }
            }
        }
        catch (Exception ex)
        {
            HandleFailure(ex.Message);
        }
    }

    private void OnSegmentReceived(TranscriptSegment segment)
    {
        _reconciler.AddSegment(segment);
        string currentText = _reconciler.GetReconciledText();

        Dispatcher.Invoke(() =>
        {
            TranscriptText.Text = string.IsNullOrEmpty(currentText) ? "Listening..." : currentText;
        });
    }

    private void OnTranscriptionError(Exception ex)
    {
        HandleFailure("Transcription provider error.");
    }

    private void OnAudioOverflow(AudioBufferOverflowEvent ev)
    {
        Dispatcher.Invoke(() =>
        {
            TranscriptText.Text = "[Warning: Audio buffer overflow]";
        });
    }

    private void HandleFailure(string message)
    {
        _isDictating = false;
        _captureCts?.Cancel();
        _stateMachine.TransitionTo(DictationState.FatalFailure);

        Dispatcher.Invoke(() =>
        {
            StatusIndicator.Fill = new SolidColorBrush(Color.FromRgb(255, 149, 0)); // Orange/Error
            StatusLabel.Text = "FAILURE";
            TranscriptText.Text = message;
        });

        Task.Delay(3000).ContinueWith(_ => ResetToIdle("Press Caps Lock to dictate..."));
    }

    private void ResetToIdle(string placeholderText)
    {
        _stateMachine.TransitionTo(DictationState.Idle);
        Dispatcher.Invoke(() =>
        {
            StatusIndicator.Fill = new SolidColorBrush(Color.FromRgb(142, 142, 147)); // Gray
            StatusLabel.Text = "IDLE";
            TranscriptText.Text = placeholderText;
        });
    }

    private void OnClosing(object? sender, System.ComponentModel.CancelEventArgs e)
    {
        var helper = new WindowInteropHelper(this);
        UnregisterHotKey(helper.Handle, HOTKEY_ID);
        _audioCapture.Dispose();
        _transcriptionProvider.Dispose();
    }
}
