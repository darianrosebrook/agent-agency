fn main() {
    #[cfg(all(feature = "coreml", not(target_os = "macos")))]
    {
        panic!("CoreML feature requires macOS platform. Use --target aarch64-apple-darwin");
    }

    #[cfg(all(feature = "coreml", target_os = "macos"))]
    {
        // Validate Xcode Command Line Tools are installed
        let output = std::process::Command::new("xcode-select")
            .arg("-p")
            .output()
            .expect("xcode-select not found - install Xcode Command Line Tools");

        if !output.status.success() {
            panic!("Xcode Command Line Tools not installed. Install with: xcode-select --install");
        }

        // Link Core ML and Metal frameworks
        println!("cargo:rustc-link-lib=framework=CoreML");
        println!("cargo:rustc-link-lib=framework=Metal");

        // Print build info
        println!("cargo:warning=CoreML support enabled for macOS target");
        println!("cargo:warning=ANE acceleration will be available if supported by hardware");
    }

    #[cfg(not(any(feature = "coreml", target_os = "macos")))]
    {
        println!("cargo:warning=CoreML support disabled (not macOS or coreml feature not enabled)");
    }
}
