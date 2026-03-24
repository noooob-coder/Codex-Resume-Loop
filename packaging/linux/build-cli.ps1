param(
    [string]$RepoRoot = $(Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path,
    [string]$Target = "x86_64-unknown-linux-gnu"
)

$ErrorActionPreference = "Stop"

$distDir = Join-Path $RepoRoot "dist"
$targetDir = Join-Path $RepoRoot "dist-target-linux"
$stageDir = Join-Path $targetDir "linux-cli-stage"
$archivePath = Join-Path $distDir "crl-cli-linux-x86_64.tar.gz"

Push-Location $RepoRoot
try {
    cargo zigbuild --release --target $Target --no-default-features --bin crl --target-dir $targetDir
} finally {
    Pop-Location
}

if (Test-Path $stageDir) {
    Remove-Item -Recurse -Force $stageDir
}
New-Item -ItemType Directory -Force -Path $stageDir | Out-Null
New-Item -ItemType Directory -Force -Path $distDir | Out-Null

Copy-Item (Join-Path $targetDir "$Target\release\crl") (Join-Path $stageDir "crl") -Force
Copy-Item (Join-Path $RepoRoot "README.md") (Join-Path $stageDir "README.md") -Force

$python = @"
import pathlib
import tarfile

stage = pathlib.Path(r"$stageDir")
archive = pathlib.Path(r"$archivePath")
with tarfile.open(archive, "w:gz") as tar:
    for name, mode in [("crl", 0o755), ("README.md", 0o644)]:
        path = stage / name
        info = tar.gettarinfo(str(path), arcname=name)
        info.mode = mode
        with path.open("rb") as fh:
            tar.addfile(info, fh)
"@

$python | python -
Write-Host "Linux CLI archive created: $archivePath"
