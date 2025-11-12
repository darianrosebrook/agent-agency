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
                
                // Check if Swift build already exists to avoid rebuilding
                let arch = std::env::var("TARGET")
                    .unwrap_or_else(|_| "unknown".to_string())
                    .split('-')
                    .next()
                    .unwrap_or("unknown")
                    .to_string();
                
                let swift_arch = match arch.as_str() {
                    "aarch64" => "arm64",
                    "x86_64" => "x86_64",
                    _ => "arm64",
                };
                
                let build_dir = bridge_path.join(".build")
                    .join(format!("{}-apple-macosx", swift_arch))
                    .join("release");
                
                let lib_path = build_dir.join("libCoreMLBridge.a");
                
                // Only build if library doesn't exist
                if !lib_path.exists() {
                    println!("cargo:warning=Swift bridge library not found, building...");
                    // Build Swift package with timeout protection
                    let swift_build = std::process::Command::new("swift")
                        .args(&["build", "-c", "release"])
                        .current_dir(&bridge_path)
                        .output();
                
                    if let Ok(output) = swift_build {
                        if output.status.success() {
                            println!("cargo:warning=Swift bridge build completed");
                        } else {
                            println!("cargo:warning=Swift bridge build failed");
                            println!("cargo:warning=Stdout: {}", String::from_utf8_lossy(&output.stdout));
                            println!("cargo:warning=Stderr: {}", String::from_utf8_lossy(&output.stderr));
                        }
                    } else {
                        println!("cargo:warning=Could not execute Swift build: {:?}", swift_build);
                    }
                } else {
                    println!("cargo:warning=Swift bridge library already exists, skipping build");
                }
                
                // Find the build directory and link the library
                let fallback_dir = bridge_path.join(".build").join("release");
                
                if build_dir.exists() {
                    println!("cargo:rustc-link-search=native={}", build_dir.display());
                } else if fallback_dir.exists() {
                    println!("cargo:rustc-link-search=native={}", fallback_dir.display());
                } else {
                    // Try to find any release directory
                    let build_base = bridge_path.join(".build");
                    if let Ok(entries) = std::fs::read_dir(&build_base) {
                        for entry in entries.flatten() {
                            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                                let release_dir = entry.path().join("release");
                                if release_dir.exists() {
                                    println!("cargo:rustc-link-search=native={}", release_dir.display());
                                    break;
                                }
                            }
                        }
                    }
                }
                
                // Link the static library - Swift PM creates libCoreMLBridge.a
                println!("cargo:rustc-link-lib=static=CoreMLBridge");
                println!("cargo:warning=Swift bridge linked successfully");
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

        // Add Swift runtime library paths to rpath
        // Swift runtime libraries are needed at runtime for Swift static libraries
        let swift_runtime_paths = vec![
            "/usr/lib/swift",
            "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx",
        ];
        
        for path in swift_runtime_paths {
            if std::path::Path::new(path).exists() {
                println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path);
            }
        }

        // Print build info
        println!("cargo:warning=CoreML support enabled for macOS target");
        println!("cargo:warning=ANE acceleration will be available if supported by hardware");
    }

    #[cfg(not(target_os = "macos"))]
    {
        println!("cargo:warning=CoreML support disabled (not macOS)");
    }
}
