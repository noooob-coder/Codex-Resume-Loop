param(
    [string]$InstallerPath = $(Resolve-Path (Join-Path $PSScriptRoot "..\..\dist\crl-setup-windows-x64-0.1.0.exe")).Path,
    [int]$FastExitThresholdMs = 1500
)

$ErrorActionPreference = "Stop"

$installRoot = Join-Path $env:LOCALAPPDATA "Programs\Codex-Resume-Loop"
$configRoot = Join-Path $env:APPDATA "shcem\crl-desktop\config"
$backupRoot = Join-Path $env:TEMP ("crl-uninstall-e2e-" + [guid]::NewGuid().ToString())
$backupInstall = Join-Path $backupRoot "install-backup"
$backupConfig = Join-Path $backupRoot "config-backup"
$uninstallLog = Join-Path $backupRoot "uninstall.log"
$originalUserPath = [Environment]::GetEnvironmentVariable("Path", "User")
$result = $null

function Restore-Backup($source, $destination) {
    if (Test-Path $destination) {
        Remove-Item -Recurse -Force $destination
    }
    if (Test-Path $source) {
        Copy-Item -Recurse -Force $source $destination
    }
}

New-Item -ItemType Directory -Force -Path $backupRoot | Out-Null

if (Test-Path $installRoot) {
    Copy-Item -Recurse -Force $installRoot $backupInstall
    Remove-Item -Recurse -Force $installRoot
}
if (Test-Path $configRoot) {
    Copy-Item -Recurse -Force $configRoot $backupConfig
    Remove-Item -Recurse -Force $configRoot
}

try {
    $installArgs = @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/TASKS=addtopath')
    $install = Start-Process -FilePath $InstallerPath -ArgumentList $installArgs -PassThru -Wait
    if ($install.ExitCode -ne 0) {
        throw "Installer failed with exit code $($install.ExitCode)"
    }

    if (-not (Test-Path (Join-Path $installRoot 'crl.exe'))) {
        throw "Installed CLI not found at $installRoot"
    }

    New-Item -ItemType Directory -Force -Path $configRoot | Out-Null
    Set-Content -Path (Join-Path $configRoot 'state.json') -Encoding utf8 -Value '{"probe":"value"}'
    Set-Content -Path (Join-Path $configRoot 'crl-desktop.log') -Encoding utf8 -Value 'probe'

    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = (Join-Path $installRoot 'crl.exe')
    $psi.Arguments = '--uninstall --purge-history'
    $psi.WorkingDirectory = $installRoot
    $psi.UseShellExecute = $false
    $psi.RedirectStandardInput = $true
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.Environment['CRL_UNINSTALL_LOG'] = $uninstallLog
    $process = [System.Diagnostics.Process]::Start($psi)
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $process.StandardInput.WriteLine('y')
    $process.StandardInput.Close()
    $process.WaitForExit()
    $stopwatch.Stop()

    $stdout = $process.StandardOutput.ReadToEnd()
    $stderr = $process.StandardError.ReadToEnd()
    if ($process.ExitCode -ne 0) {
        throw "crl --uninstall failed with exit code $($process.ExitCode)`nSTDOUT:`n$stdout`nSTDERR:`n$stderr"
    }

    $removeDeadline = (Get-Date).AddSeconds(8)
    while (((Test-Path $installRoot) -or (Test-Path $configRoot)) -and (Get-Date) -lt $removeDeadline) {
        Start-Sleep -Milliseconds 200
    }

    $result = [pscustomobject]@{
        uninstall_cli_duration_ms = [Math]::Round($stopwatch.Elapsed.TotalMilliseconds, 0)
        below_threshold = ($stopwatch.Elapsed.TotalMilliseconds -lt $FastExitThresholdMs)
        install_removed = -not (Test-Path $installRoot)
        history_removed = -not (Test-Path $configRoot)
        stdout = $stdout.Trim()
        stderr = $stderr.Trim()
        uninstall_log = if (Test-Path $uninstallLog) { Get-Content -Raw $uninstallLog } else { "" }
    }
}
finally {
    [Environment]::SetEnvironmentVariable("Path", $originalUserPath, "User")
    if (Test-Path $installRoot) {
        Remove-Item -Recurse -Force $installRoot
    }
    if (Test-Path $configRoot) {
        Remove-Item -Recurse -Force $configRoot
    }
    Restore-Backup $backupInstall $installRoot
    Restore-Backup $backupConfig $configRoot
    if (Test-Path $backupRoot) {
        Remove-Item -Recurse -Force $backupRoot
    }
    if ($null -ne $result) {
        $result | ConvertTo-Json -Depth 3
    }
}
