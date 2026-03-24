fn main() {
    if std::env::var_os("CARGO_FEATURE_DESKTOP_UI").is_none() {
        return;
    }

    #[cfg(target_os = "windows")]
    {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("ui/assets/crl-icon.ico");
        resource
            .compile()
            .expect("failed to compile Windows resources");
    }

    slint_build::compile("ui/main.slint").expect("failed to compile Slint UI");
}
