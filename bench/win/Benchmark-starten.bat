@echo off
REM HASE VRM MAKER - Performance-Benchmark (SP-08). Doppelklick genuegt. Dauer ca. 2-6 Minuten.
REM Benoetigt die Repo-Ordner content\testcandidate (Daten). Ergebnisse: bench\results\
chcp 65001 >nul
cd /d "%~dp0..\.."
if not exist content\testcandidate\mpfb2-cc0\base.obj (
  echo FEHLER: Bitte diesen Ordner innerhalb des vollstaendigen Repository-Klons lassen ^(content\testcandidate fehlt^).
  pause & exit /b 1
)
if not exist bench\results mkdir bench\results
set OUT=bench\results\windows_%COMPUTERNAME%
echo [1/3] GPU-Informationen ...
powershell -NoProfile -Command "Get-CimInstance Win32_VideoController | Select-Object Name,DriverVersion,AdapterRAM | ConvertTo-Json" > "%OUT%_gpu_info.json"
powershell -NoProfile -Command "Get-CimInstance Win32_Processor | Select-Object Name,NumberOfCores,NumberOfLogicalProcessors | ConvertTo-Json" > "%OUT%_cpu_info.json"
echo [2/3] Benchmark mit DirectX 12 ...
set WGPU_BACKEND=dx12
bench\win\morph-spike.exe all --copies 6 --frames 600 > "%OUT%_dx12.json" 2> "%OUT%_dx12.err"
echo [3/3] Benchmark mit Vulkan ...
set WGPU_BACKEND=vulkan
bench\win\morph-spike.exe all --copies 6 --frames 600 > "%OUT%_vulkan.json" 2> "%OUT%_vulkan.err"
echo.
echo Fertig. Bitte die Dateien in bench\results\ ^(windows_%COMPUTERNAME%_*^) committen oder mir schicken.
echo Hinweis: Leistungsplan "Hoechstleistung" und keine anderen schweren Programme waehrend des Laufs.
pause
