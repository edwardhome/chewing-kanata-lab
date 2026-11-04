@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0do_install.ps1" %*
exit /b %errorlevel%
