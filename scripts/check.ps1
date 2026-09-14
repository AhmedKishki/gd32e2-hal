#!/usr/bin/env pwsh
# The checks CI runs, in the same order. Stops at the first failure.
#
# The library is linted on both ends of the range: the largest part, and the
# smallest one, whose x4 pin map and small-package gates the largest never sees.

$ErrorActionPreference = 'Stop'

$steps = @(
    @('fmt', '--check'),
    @('libclippy', '--', '-D', 'warnings'),
    @('clippy', '--features', 'gd32e230f4xx', '--', '-D', 'warnings'),
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
