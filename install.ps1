<#
.SYNOPSIS
Installs the Magic Trackpad 2 Windows driver package when a built package is present.

.DESCRIPTION
This script detects Windows architecture, finds a matching AmtPtpDevice.inf in a
release/build package, and installs it with pnputil. It does not build or sign
drivers; Windows kernel drivers must already be signed.
#>

[CmdletBinding()]
param(
    [switch]$NoControlPanel
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Write-Step {
    param([string]$Message)
    Write-Host "==> $Message"
}

function Stop-WithMessage {
    param([string]$Message)
    Write-Error $Message
    exit 1
}

function Test-Administrator {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

if (-not $IsWindows) {
    Stop-WithMessage "This installer is for Windows only. On Linux, this Windows WDF driver cannot be installed directly."
}

if (-not (Test-Administrator)) {
    Stop-WithMessage "Please run PowerShell as Administrator, then run this script again."
}

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$arch = switch ([Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()) {
    "X64" { "AMD64" }
    "Arm64" { "ARM64" }
    default {
        Stop-WithMessage "Unsupported Windows architecture: $([Runtime.InteropServices.RuntimeInformation]::OSArchitecture)"
    }
}

Write-Step "Detected Windows $arch"

$candidateDirs = @(
    (Join-Path $root $arch),
    (Join-Path $root "result\$arch"),
    (Join-Path $root "build\result\$arch"),
    (Join-Path $root "build")
)

$inf = $null
foreach ($dir in $candidateDirs) {
    $candidate = Join-Path $dir "AmtPtpDevice.inf"
    if (Test-Path $candidate) {
        $inf = (Resolve-Path $candidate).Path
        break
    }
}

if ($null -eq $inf) {
    Stop-WithMessage @"
Could not find a built driver package for $arch.

Expected one of:
  $($candidateDirs -join [Environment]::NewLine + "  ")\AmtPtpDevice.inf

Use a release zip, or build/sign the driver first with build\make.bat.
"@
}

$packageDir = Split-Path -Parent $inf
$requiredFiles = @("AmtPtpDeviceUsbUm.dll", "AmtPtpHidFilter.sys")
foreach ($file in $requiredFiles) {
    if (-not (Test-Path (Join-Path $packageDir $file))) {
        Stop-WithMessage "Found $inf, but missing $file in $packageDir. Use a complete release/build package."
    }
}

Write-Step "Installing $inf"
$pnputil = Join-Path $env:SystemRoot "System32\pnputil.exe"
& $pnputil /add-driver "$inf" /install
if ($LASTEXITCODE -ne 0) {
    Stop-WithMessage "pnputil failed with exit code $LASTEXITCODE."
}

if (-not $NoControlPanel) {
    $uiCandidates = @(
        (Join-Path $root "MagicTrackpadRs.exe"),
        (Join-Path $root "mt2-win-ui.exe"),
        (Join-Path $root "result\MagicTrackpadRs.exe"),
        (Join-Path $root "build\result\MagicTrackpadRs.exe"),
        (Join-Path $root "target\release\mt2-win-ui.exe")
    )

    foreach ($candidate in $uiCandidates) {
        if (Test-Path $candidate) {
            Write-Step "Opening Rust desktop UI"
            Start-Process -FilePath (Resolve-Path $candidate).Path
            break
        }
    }
}

Write-Host ""
Write-Host "Install command completed. Pair or reconnect the Magic Trackpad 2 if Windows does not pick it up immediately."
