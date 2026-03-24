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
$issPath = Join-Path $PSScriptRoot "crl.iss"
$isccPath = "C:\Users\shcem\AppData\Local\Programs\Inno Setup 6\ISCC.exe"

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
Copy-Item (Join-Path $RepoRoot "ui\assets\crl-icon.ico") (Join-Path $stageDir "crl-icon.ico") -Force

$env:CRL_VERSION = $version
$env:CRL_OUTPUT_DIR = $distDir
$env:CRL_STAGE_DIR = $stageDir

& $isccPath $issPath

$installerVersioned = Join-Path $distDir "crl-setup-windows-x64-$version.exe"
if (-not (Test-Path $installerVersioned)) {
    throw "Inno Setup did not create the expected installer: $installerVersioned"
}

Write-Host "Windows installer created:"
Write-Host "  $installerVersioned"
