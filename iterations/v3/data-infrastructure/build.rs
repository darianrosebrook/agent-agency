fn main() {
    #[cfg(target_os = "macos")]
    {
        // Build Swift bridge if it exists (same logic as system-acceleration)
        // Path: iterations/v3/data-infrastructure -> project root -> models/languages/swift/coreml-bridge
        let bridge_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent() // iterations/v3
            .and_then(|p| p.parent()) // iterations
            .and_then(|p| p.parent()) // agent-agency
            .map(|p| {
                p.join("models")
                    .join("languages")
                    .join("swift")
                    .join("coreml-bridge")
            });

        if let Some(bridge_path) = bridge_dir {
            if bridge_path.exists() {
                println!(
                    "cargo:warning=Found Swift bridge at: {}",
                    bridge_path.display()
                );

                // Build Swift package (only if CoreML feature is enabled)
                #[cfg(feature = "coreml")]
                {
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

                    let build_dir = bridge_path
                        .join(".build")
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
                                println!("cargo:warning=Swift bridge built successfully");
                            } else {
                                println!("cargo:warning=Swift bridge build failed");
                                println!(
                                    "cargo:warning=Stdout: {}",
                                    String::from_utf8_lossy(&output.stdout)
                                );
                                println!(
                                    "cargo:warning=Stderr: {}",
                                    String::from_utf8_lossy(&output.stderr)
                                );
                            }
                        } else {
                            println!(
                                "cargo:warning=Could not execute Swift build: {:?}",
                                swift_build
                            );
                        }
                    } else {
                        println!(
                            "cargo:warning=Swift bridge library already exists, skipping build"
                        );
                    }

                    // Link the static library - Swift PM creates libCoreMLBridge.a
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
                                        println!(
                                            "cargo:rustc-link-search=native={}",
                                            release_dir.display()
                                        );
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    println!("cargo:rustc-link-lib=static=CoreMLBridge");
                    println!("cargo:warning=Swift bridge linked successfully");
                }
            } else {
                println!(
                    "cargo:warning=Swift bridge path does not exist: {}",
                    bridge_path.display()
                );
            }
        } else {
            println!("cargo:warning=Could not resolve Swift bridge path");
        }

        // Link Core ML and Metal frameworks (only if CoreML feature is enabled)
        #[cfg(feature = "coreml")]
        {
            println!("cargo:rustc-link-lib=framework=CoreML");
            println!("cargo:rustc-link-lib=framework=Metal");
            println!("cargo:rustc-link-lib=framework=Foundation");

            // Link Swift runtime libraries required for static Swift libraries
            // Only link if CoreML feature is enabled (production builds)
            let swift_runtime_path = "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx";
            if std::path::Path::new(swift_runtime_path).exists() {
                println!("cargo:rustc-link-search=native={}", swift_runtime_path);
                // Link Swift compatibility libraries required by Swift static libraries
                println!("cargo:rustc-link-lib=static=swiftCompatibility56");
                println!("cargo:rustc-link-lib=static=swiftCompatibilityConcurrency");
                println!("cargo:rustc-link-lib=static=swiftCompatibilityPacks");
            }

            // Add Swift runtime library paths to rpath for runtime resolution
            // Include system Swift library paths for Concurrency runtime
            // Also check for Swift 6.x paths (newer Xcode versions)
            let swift_runtime_paths = vec![
                "/usr/lib/swift",
                "/System/Library/Frameworks",
                "/System/Library/Frameworks/CoreML.framework",
                "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx",
                "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.5/macosx",
                "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-6.0/macosx",
            ];

            for path in swift_runtime_paths {
                if std::path::Path::new(path).exists() {
                    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path);
                }
            }

            // Also add @rpath resolution for Swift libraries embedded in frameworks
            println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/../Frameworks");

            // Add @executable_path for test binaries
            println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
            println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");

            // Print build info
            println!("cargo:warning=CoreML support enabled for macOS target");
            println!("cargo:warning=ANE acceleration will be available if supported by hardware");
        }
        #[cfg(not(feature = "coreml"))]
        {
            println!("cargo:warning=CoreML support disabled (feature not enabled)");
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        println!("cargo:warning=CoreML support disabled (not macOS)");
    }
}
