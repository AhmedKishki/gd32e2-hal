#!/usr/bin/env pwsh
# Every check CI runs, plus the documentation build with warnings denied, so
# broken intra-doc links fail too. Stops at the first failure.

$ErrorActionPreference = 'Stop'

$steps = @(
    @('fmt', '--check'),
    @('libclippy', '--', '-D', 'warnings'),
    @('lib'),
    @('examplesclippy', '--', '-D', 'warnings'),
    @('examples'),
    @('doc', '--no-deps', '--target', 'thumbv8m.base-none-eabi', '--features', 'defmt,gd32e230c8xx')
)

Push-Location (Split-Path $PSScriptRoot)
$rustdocflags = $env:RUSTDOCFLAGS
$env:RUSTDOCFLAGS = '-D warnings'
try {
    foreach ($step in $steps) {
        Write-Host "==> cargo $($step -join ' ')" -ForegroundColor Cyan
        & cargo @step
        if ($LASTEXITCODE -ne 0) {
            Write-Host "FAILED: cargo $($step -join ' ')" -ForegroundColor Red
            exit $LASTEXITCODE
        }
    }
    Write-Host 'All checks passed.' -ForegroundColor Green
}
finally {
    $env:RUSTDOCFLAGS = $rustdocflags
    Pop-Location
}
