#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(not(feature = "desktop-ui"))]
compile_error!("crl-desktop binary requires the `desktop-ui` feature");

#[cfg(feature = "desktop-ui")]
fn main() -> Result<(), slint::PlatformError> {
    crl_desktop::desktop::run_desktop()
}
