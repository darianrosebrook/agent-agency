fn main() {
    #[cfg(target_os = "macos")]
    {
        // Build Swift bridge if it exists (same logic as system-acceleration)
        // Path: iterations/v3/data-infrastructure -> project root -> models/languages/swift/coreml-bridge
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
                        // Swift Package Manager builds static libraries in architecture-specific directories
                        let arch = std::env::var("TARGET")
                            .unwrap_or_else(|_| "unknown".to_string())
                            .split('-')
                            .next()
                            .unwrap_or("unknown")
                            .to_string();
                        
                        // Map Rust target arch to Swift arch
                        let swift_arch = match arch.as_str() {
                            "aarch64" => "arm64",
                            "x86_64" => "x86_64",
                            _ => "arm64", // Default to arm64 for Apple Silicon
                        };
                        
                        let build_dir = bridge_path.join(".build")
                            .join(format!("{}-apple-macosx", swift_arch))
                            .join("release");
                        
                        // Also check the generic release directory as fallback
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








