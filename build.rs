#[cfg(target_os = "windows")]
fn sync_windows_icon() {
    use std::path::Path;
    use std::process::Command;

    println!("cargo:rerun-if-changed=ui/assets/crl-icon.svg");
    println!("cargo:rerun-if-changed=packaging/windows/sync-icon.py");

    let script = Path::new("packaging/windows/sync-icon.py");
    let input = Path::new("ui/assets/crl-icon.svg");
    let output = Path::new("ui/assets/crl-icon.ico");

    let attempts: [(&str, &[&str]); 2] = [
        (
            "python",
            &[
                "packaging/windows/sync-icon.py",
                "ui/assets/crl-icon.svg",
                "ui/assets/crl-icon.ico",
            ],
        ),
        (
            "py",
            &[
                "-3",
                "packaging/windows/sync-icon.py",
                "ui/assets/crl-icon.svg",
                "ui/assets/crl-icon.ico",
            ],
        ),
    ];

    for (program, args) in attempts {
        match Command::new(program).args(args).status() {
            Ok(status) if status.success() => return,
            Ok(_) => continue,
            Err(_) => continue,
        }
    }

    panic!(
        "failed to synchronize Windows icon from {} to {} using {}. Install Python 3 with Pillow support and make `python` or `py -3` available.",
        input.display(),
        output.display(),
        script.display()
    );
}

fn main() {
    if std::env::var_os("CARGO_FEATURE_DESKTOP_UI").is_none() {
        return;
    }

    #[cfg(target_os = "windows")]
    {
        sync_windows_icon();

        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("ui/assets/crl-icon.ico");
        resource
            .compile()
            .expect("failed to compile Windows resources");
    }

    slint_build::compile("ui/main.slint").expect("failed to compile Slint UI");
}
