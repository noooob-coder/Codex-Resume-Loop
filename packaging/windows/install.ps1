param(
    [string]$InstallDir = $(Join-Path $env:LOCALAPPDATA "Programs\CRL")
)

$ErrorActionPreference = "Stop"

function Add-ToUserPath {
    param([string]$Directory)

    $current = [Environment]::GetEnvironmentVariable("Path", "User")
    $parts = @()
    if ($current) {
        $parts = $current -split ";" | Where-Object { $_ }
    }

    $normalized = $Directory.TrimEnd("\")
    $exists = $parts | Where-Object { $_.TrimEnd("\") -ieq $normalized }
    if ($exists) {
        return
    }

    $parts += $Directory
    [Environment]::SetEnvironmentVariable("Path", ($parts -join ";"), "User")
}

function New-Shortcut {
    param(
        [string]$ShortcutPath,
        [string]$TargetPath,
        [string]$WorkingDirectory
    )

    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut($ShortcutPath)
    $shortcut.TargetPath = $TargetPath
    $shortcut.WorkingDirectory = $WorkingDirectory
    $shortcut.Save()
}

$scriptDir = Split-Path -Parent $PSCommandPath
$payloadFiles = @("crl-desktop.exe", "crl.exe", "README.txt")

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
foreach ($file in $payloadFiles) {
    Copy-Item (Join-Path $scriptDir $file) (Join-Path $InstallDir $file) -Force
}

Add-ToUserPath -Directory $InstallDir

$startMenuDir = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\CRL"
New-Item -ItemType Directory -Force -Path $startMenuDir | Out-Null
New-Shortcut `
    -ShortcutPath (Join-Path $startMenuDir "CRL Desktop.lnk") `
    -TargetPath (Join-Path $InstallDir "crl-desktop.exe") `
    -WorkingDirectory $InstallDir

Write-Host "Installed CRL to $InstallDir"
Write-Host "Added $InstallDir to the user PATH."
Write-Host "Start Menu shortcut created: CRL Desktop"
