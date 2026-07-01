@echo off
title Universal Dictation - Startup Manager
cls

:: Change directory to the folder where this batch script is located
cd /d "%~dp0"

echo ===================================================
echo   Stopping existing application processes...
echo ===================================================
taskkill /f /im ControlPlane.Api.exe 2>nul
taskkill /f /im DesktopApp.exe 2>nul

echo.
echo ===================================================
echo   Building Universal Dictation Solution...
echo ===================================================
dotnet build UniversalDictation.sln -c Release -v q
if %ERRORLEVEL% neq 0 (
    echo.
    echo [ERROR] Build failed. Please fix compilation issues first.
    pause
    exit /b %ERRORLEVEL%
)

echo.
echo ===================================================
echo   Starting Universal Dictation Application Services
echo ===================================================

echo.
echo [1/2] Starting Control Plane API...
start "Universal Dictation Control Plane" "src\ControlPlane.Api\bin\Release\net10.0\ControlPlane.Api.exe"

echo.
echo [2/2] Launching WPF Desktop UI Overlay...
start "Universal Dictation Client" "src\DesktopApp\bin\Release\net10.0-windows10.0.19041\DesktopApp.exe"

echo.
echo ===================================================
echo   All services launched successfully.
echo ===================================================
exit /b 0
