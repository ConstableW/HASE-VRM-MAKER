@echo off
rem HASE VRM MAKER - Editor starten (Repo-Klon noetig: content\testcandidate muss vorhanden sein)
cd /d "%~dp0\..\.."
"%~dp0hase-editor.exe"
if errorlevel 1 (
  echo Start fehlgeschlagen - versuche Vulkan statt DX12
  set WGPU_BACKEND=vulkan
  "%~dp0hase-editor.exe"
)
pause
