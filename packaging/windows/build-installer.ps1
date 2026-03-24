param(
    [string]$RepoRoot = $(Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
)

$ErrorActionPreference = "Stop"

$cargoToml = Get-Content -Raw (Join-Path $RepoRoot "Cargo.toml")
if ($cargoToml -notmatch 'version\s*=\s*"([^"]+)"') {
    throw "Unable to read package version from Cargo.toml"
}
$version = $Matches[1]

$distDir = Join-Path $RepoRoot "dist"
$buildDir = Join-Path $RepoRoot "dist-target"
$stageDir = Join-Path $buildDir "windows-installer-stage"
$releaseDir = Join-Path $RepoRoot "target\release"
$installerVersioned = Join-Path $distDir "crl-setup-windows-x64-$version.exe"
$installerStable = Join-Path $distDir "crl-setup-windows-x64.exe"

New-Item -ItemType Directory -Force -Path $distDir | Out-Null
if (Test-Path $stageDir) {
    Remove-Item -Recurse -Force $stageDir
}
New-Item -ItemType Directory -Force -Path $stageDir | Out-Null

Push-Location $RepoRoot
try {
    cargo build --release
} finally {
    Pop-Location
}

Copy-Item (Join-Path $releaseDir "crl-desktop.exe") (Join-Path $stageDir "crl-desktop.exe") -Force
Copy-Item (Join-Path $releaseDir "crl.exe") (Join-Path $stageDir "crl.exe") -Force
Copy-Item (Join-Path $RepoRoot "README.md") (Join-Path $stageDir "README.txt") -Force
Copy-Item (Join-Path $PSScriptRoot "install.ps1") (Join-Path $stageDir "install.ps1") -Force

$installCmd = @"
@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1"
exit /b %ERRORLEVEL%
"@
Set-Content -Path (Join-Path $stageDir "install.cmd") -Value $installCmd -Encoding Ascii

$sed = @"
[Version]
Class=IEXPRESS
SEDVersion=3
[Options]
PackagePurpose=InstallApp
ShowInstallProgramWindow=1
HideExtractAnimation=1
UseLongFileName=1
InsideCompressed=0
CAB_FixedSize=0
CAB_ResvCodeSigning=0
RebootMode=N
InstallPrompt=%InstallPrompt%
DisplayLicense=%DisplayLicense%
FinishMessage=%FinishMessage%
TargetName=%TargetName%
FriendlyName=%FriendlyName%
AppLaunched=%AppLaunched%
PostInstallCmd=%PostInstallCmd%
AdminQuietInstCmd=%AdminQuietInstCmd%
UserQuietInstCmd=%UserQuietInstCmd%
SourceFiles=SourceFiles
[Strings]
InstallPrompt=
DisplayLicense=
FinishMessage=CRL installation completed.
TargetName=$installerVersioned
FriendlyName=CRL Setup
AppLaunched=cmd.exe /d /s /c ""install.cmd""
PostInstallCmd=<None>
AdminQuietInstCmd=cmd.exe /d /s /c ""install.cmd""
UserQuietInstCmd=cmd.exe /d /s /c ""install.cmd""
FILE0=install.cmd
FILE1=install.ps1
FILE2=crl-desktop.exe
FILE3=crl.exe
FILE4=README.txt
[SourceFiles]
SourceFiles0=$stageDir
[SourceFiles0]
%FILE0%=
%FILE1%=
%FILE2%=
%FILE3%=
%FILE4%=
"@

$sedPath = Join-Path $stageDir "crl-installer.sed"
Set-Content -Path $sedPath -Value $sed -Encoding Ascii

& "$env:WINDIR\System32\iexpress.exe" /N /Q /M $sedPath
for ($attempt = 0; $attempt -lt 50 -and -not (Test-Path $installerVersioned); $attempt++) {
    Start-Sleep -Milliseconds 200
}
if (-not (Test-Path $installerVersioned)) {
    throw "IExpress did not create the expected installer: $installerVersioned"
}
Copy-Item $installerVersioned $installerStable -Force

Write-Host "Windows installer created:"
Write-Host "  $installerVersioned"
Write-Host "  $installerStable"
