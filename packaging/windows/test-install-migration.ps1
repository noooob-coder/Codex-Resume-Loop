param(
    [string]$InstallerPath = $(Resolve-Path (Join-Path $PSScriptRoot "..\..\dist\crl-setup-windows-x64-0.1.0.exe")).Path
)

$ErrorActionPreference = "Stop"

$legacyBin = Join-Path $env:USERPROFILE ".local\bin"
$legacyInstall = Join-Path $env:LOCALAPPDATA "Programs\CRL"
$currentInstall = Join-Path $env:LOCALAPPDATA "Programs\Codex-Resume-Loop"
$backupRoot = Join-Path $env:TEMP ("crl-install-migration-" + [guid]::NewGuid().ToString())
$backupLegacyBin = Join-Path $backupRoot "legacy-bin"
$backupLegacyInstall = Join-Path $backupRoot "legacy-install"
$result = $null

function Restore-File($source, $destination) {
    if (Test-Path $source) {
        Copy-Item -Force $source $destination
    } elseif (Test-Path $destination) {
        Remove-Item -Force $destination
    }
}

New-Item -ItemType Directory -Force -Path $backupRoot | Out-Null
New-Item -ItemType Directory -Force -Path $backupLegacyBin | Out-Null
New-Item -ItemType Directory -Force -Path $legacyBin | Out-Null

$legacyBinFiles = @(
    "crl.exe",
    "codex-resume-loop.exe",
    "crl.cmd",
    "codex-resume-loop.cmd",
    "crl.ps1",
    "codex-resume-loop.ps1",
    "crl",
    "codex-resume-loop"
)

foreach ($name in $legacyBinFiles) {
    $target = Join-Path $legacyBin $name
    $backup = Join-Path $backupLegacyBin $name
    if (Test-Path $target) {
        Copy-Item -Force $target $backup
        Remove-Item -Force $target
    }
}

if (Test-Path $legacyInstall) {
    Copy-Item -Recurse -Force $legacyInstall $backupLegacyInstall
    Remove-Item -Recurse -Force $legacyInstall
}

try {
    Set-Content -Path (Join-Path $legacyBin "crl.exe") -Encoding ascii -Value "legacy"
    Set-Content -Path (Join-Path $legacyBin "codex-resume-loop.cmd") -Encoding ascii -Value "legacy"
    New-Item -ItemType Directory -Force -Path $legacyInstall | Out-Null
    Set-Content -Path (Join-Path $legacyInstall "crl.exe") -Encoding ascii -Value "legacy"

    $install = Start-Process -FilePath $InstallerPath -ArgumentList @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/TASKS=addtopath') -PassThru -Wait
    if ($install.ExitCode -ne 0) {
        throw "Installer failed with exit code $($install.ExitCode)"
    }

    $result = [pscustomobject]@{
        install_exit_code = $install.ExitCode
        current_install_exists = Test-Path (Join-Path $currentInstall "crl.exe")
        legacy_bin_crl_removed = -not (Test-Path (Join-Path $legacyBin "crl.exe"))
        legacy_bin_cmd_removed = -not (Test-Path (Join-Path $legacyBin "codex-resume-loop.cmd"))
        legacy_install_removed = -not (Test-Path $legacyInstall)
    }
}
finally {
    foreach ($name in $legacyBinFiles) {
        Restore-File (Join-Path $backupLegacyBin $name) (Join-Path $legacyBin $name)
    }

    if (Test-Path $legacyInstall) {
        Remove-Item -Recurse -Force $legacyInstall
    }
    if (Test-Path $backupLegacyInstall) {
        Copy-Item -Recurse -Force $backupLegacyInstall $legacyInstall
    }
    if (Test-Path $backupRoot) {
        Remove-Item -Recurse -Force $backupRoot
    }
}

if ($null -eq $result) {
    throw "Migration validation did not produce a result."
}

$result | ConvertTo-Json -Depth 3
