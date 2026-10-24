# Windows-Benchmark (echte GPU): im Repo-Root oder in bench/morph-spike ausführen.
# Voraussetzung: Rust (rustup, MSVC-Toolchain). Ergebnis: bench/results/<Datum>_<Rechnername>.json
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot
cargo build --release
$root = Resolve-Path "..\.."
$out = Join-Path $root ("bench\results\{0}_{1}.json" -f (Get-Date -Format "yyyy-MM-dd"), $env:COMPUTERNAME)
Write-Host "Messe (ca. 1-3 Minuten) ..."
& .\target\release\morph-spike.exe all --copies 6 --frames 600 | Out-File -Encoding utf8 $out
Write-Host "Fertig: $out"
Write-Host "Bitte zusätzlich notieren: GPU-Modell, Treiberversion, Leistungsplan (Höchstleistung), laufende Hintergrundprogramme."
