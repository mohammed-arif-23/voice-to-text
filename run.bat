@echo off
title Universal Dictation - Startup Manager
cls
echo ===================================================
echo   Starting Universal Dictation Application Services
echo ===================================================

echo.
echo [1/3] Starting Control Plane API...
start "Universal Dictation Control Plane" dotnet run --project src/ControlPlane.Api/ControlPlane.Api.csproj --configuration Release

echo.
echo [2/3] Waiting for Control Plane to initialize...
timeout /t 5 /nobreak > nul

echo.
echo [3/3] Launching WPF Desktop UI Overlay...
start "Universal Dictation Client" dotnet run --project src/DesktopApp/DesktopApp.csproj --configuration Release

echo.
echo ===================================================
echo   All services launched successfully.
echo ===================================================
pause
