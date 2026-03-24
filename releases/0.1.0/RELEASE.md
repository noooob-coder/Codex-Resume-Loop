# Release 0.1.0

## What's Changed

- Fixed middle panel layout so it stays vertically scrollable without horizontal overflow.
- Switched command output to real-time streaming instead of waiting for line or stage completion.
- Reduced workspace switching latency by reusing a session catalog and using a single-pass workspace projection path.
- Tightened platform packaging so Linux CLI can build without dragging desktop UI dependencies into the build graph.

## Artifacts

- Windows installer: `releases/0.1.0/windows/crl-setup-windows-x64-0.1.0.exe`
- Windows desktop binary: `releases/0.1.0/windows/crl-desktop-windows-x64-0.1.0.exe`
- Windows CLI binary: `releases/0.1.0/windows/crl-cli-windows-x64-0.1.0.exe`
- Linux CLI archive: `releases/0.1.0/linux/crl-cli-linux-x86_64.tar.gz`
- iOS build kit: `releases/0.1.0/ios/crl-ios-ui-and-cli-build-kit.tar.gz`

## Notes

- Windows installer adds `crl` to PATH, so the CLI can be called directly after installation.
- Linux release is CLI-only by design. Extract the archive and run `install.sh`, then call `crl` directly.
- iOS release is provided as a build kit because final iOS binaries require macOS, Xcode, `xcrun`, and the Apple SDK for linking.
