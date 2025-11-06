fn main() {
    #[cfg(target_os = "macos")]
    {
        // Validate Xcode Command Line Tools are installed
        let output = std::process::Command::new("xcode-select")
            .arg("-p")
            .output()
            .expect("xcode-select not found - install Xcode Command Line Tools");

        if !output.status.success() {
            panic!("Xcode Command Line Tools not installed. Install with: xcode-select --install");
        }

        // Build Swift bridge if it exists
        // Path: iterations/v3/system-acceleration -> project root -> models/languages/swift/coreml-bridge
        let bridge_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()  // iterations/v3
            .and_then(|p| p.parent())  // iterations
            .and_then(|p| p.parent())  // agent-agency
            .map(|p| p.join("models").join("languages").join("swift").join("coreml-bridge"));
        
        if let Some(bridge_path) = bridge_dir {
            if bridge_path.exists() {
                println!("cargo:warning=Found Swift bridge at: {}", bridge_path.display());
                
                // Build Swift package
                let swift_build = std::process::Command::new("swift")
                    .args(&["build", "-c", "release"])
                    .current_dir(&bridge_path)
                    .output();
                
                if let Ok(output) = swift_build {
                    if output.status.success() {
                        // Swift Package Manager builds static libraries in .build/release/
                        // The library name matches the package name
                        let build_dir = bridge_path.join(".build").join("release");
                        println!("cargo:rustc-link-search=native={}", build_dir.display());
                        
                        // Link the static library - Swift PM creates libCoreMLBridge.a
                        println!("cargo:rustc-link-lib=static=CoreMLBridge");
                        println!("cargo:warning=Swift bridge built and linked successfully");
                    } else {
                        println!("cargo:warning=Swift bridge build failed");
                        println!("cargo:warning=Stdout: {}", String::from_utf8_lossy(&output.stdout));
                        println!("cargo:warning=Stderr: {}", String::from_utf8_lossy(&output.stderr));
                    }
                } else {
                    println!("cargo:warning=Could not execute Swift build: {:?}", swift_build);
                }
            } else {
                println!("cargo:warning=Swift bridge path does not exist: {}", bridge_path.display());
            }
        } else {
            println!("cargo:warning=Could not resolve Swift bridge path");
        }

        // Link Core ML and Metal frameworks
        println!("cargo:rustc-link-lib=framework=CoreML");
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=Foundation");

        // Print build info
        println!("cargo:warning=CoreML support enabled for macOS target");
        println!("cargo:warning=ANE acceleration will be available if supported by hardware");
    }

    #[cfg(not(target_os = "macos"))]
    {
        println!("cargo:warning=CoreML support disabled (not macOS)");
    }
}
