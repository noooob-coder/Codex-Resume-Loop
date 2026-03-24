# iOS Build Notes

Run `build-ui-and-cli.sh` on macOS with Xcode installed.

The current Windows environment can install Rust iOS targets, but it cannot link final iOS binaries because the Apple SDK and `xcrun` are not available here.

The script builds:

- `crl` as the CLI binary with `--no-default-features`
- `crl-desktop` as the UI binary with the default desktop feature set

Artifacts are copied into `dist/` on the macOS build host.
