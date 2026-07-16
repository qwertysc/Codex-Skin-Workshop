@echo off
setlocal
cd /d "%~dp0"

set "APP=%~dp0Codex 皮肤工坊.exe"

if not exist "%APP%" (
  echo.
  echo [Codex 皮肤工坊]
  echo 找不到 Codex 皮肤工坊.exe。
  echo 请先完整解压 ZIP，并保留启动器和程序在同一文件夹。
  echo.
  pause
  exit /b 1
)

rem Only remove the download-zone marker from this portable executable.
powershell.exe -NoLogo -NoProfile -NonInteractive -Command "Unblock-File -LiteralPath $env:APP" >nul 2>&1

start "" "%APP%"
exit /b 0
