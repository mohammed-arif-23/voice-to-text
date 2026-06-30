@echo off
setlocal

REM =============================================
REM   ScribeRx Siri-like Voice Assistant Launcher
REM   Double-click or run in your own CMD window
REM =============================================

cd /d "E:\voice-to-text\voice-to-text"

echo.
echo  ===================================================
echo    ScribeRx  -  Siri-Style Voice Dictation Assistant
echo  ===================================================
echo    UI      : Translucent Floating Glassmorphic GUI
echo    VISUALS : Live Pulsing Siri Waveform Visualizer
echo    ENGINE  : Fast Offline Whisper (base.en)
echo    VAD     : Real-time 2s Auto-Stop and Direct Type
echo.
echo    HOTKEY  : Ctrl + Alt + F9  (global toggle)
echo    FALLBACK: Press [Enter] here to toggle
echo  ===================================================
echo.
echo  Initializing background speech engines...
echo.

target\debug\app-shell.exe

pause
