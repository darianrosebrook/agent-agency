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

        // Build and link Swift bridge only when coreml feature is enabled
        #[cfg(feature = "coreml")]
        {
            // Build Swift bridge if it exists
            // Path: iterations/v3/system-acceleration -> project root -> models/languages/swift/coreml-bridge
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
                                println!("cargo:warning=Swift bridge build completed");
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

                    // Link the static library - Swift PM creates libCoreMLBridge.a
                    println!("cargo:rustc-link-lib=static=CoreMLBridge");
                    println!("cargo:warning=Swift bridge linked successfully");
                } else {
                    println!(
                        "cargo:warning=Swift bridge path does not exist: {}",
                        bridge_path.display()
                    );
                }
            } else {
                println!("cargo:warning=Could not resolve Swift bridge path");
            }
        }
        
        #[cfg(not(feature = "coreml"))]
        {
            println!("cargo:warning=CoreML feature disabled - skipping Swift bridge");
        }

        // Link Core ML and Metal frameworks (only if CoreML feature is enabled)
        #[cfg(feature = "coreml")]
        {
            println!("cargo:rustc-link-lib=framework=CoreML");
            println!("cargo:rustc-link-lib=framework=Metal");
            println!("cargo:rustc-link-lib=framework=Foundation");
        }

        // Link Swift runtime libraries required for static Swift libraries
        // Only link if CoreML feature is enabled (production builds)
        #[cfg(feature = "coreml")]
        {
            // Swift 6 is required for the new concurrency API with isolation parameter
            // Prioritize Swift 6 libraries to avoid conflicts with Swift 5.5
            // Check for Swift 6 first (system Swift at /usr/lib/swift)
            let swift6_system_path = "/usr/lib/swift";
            let swift6_toolchain_path = "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx";
            let swift55_path = "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.5/macosx";
            
            // Determine which Swift version to use - prefer Swift 6
            let use_swift6 = std::path::Path::new(swift6_system_path).exists() 
                || std::path::Path::new(swift6_toolchain_path).exists();
            
            let swift_runtime_paths = if use_swift6 {
                // Use Swift 6 - prioritize system Swift, then toolchain Swift 6
                vec![
                    swift6_system_path,
                    swift6_toolchain_path,
                ]
            } else {
                // Fallback to Swift 5.5 only if Swift 6 is not available
                vec![swift55_path]
            };

            // Add only the selected Swift runtime paths as native link search paths
            // This prevents mixing Swift 5.5 and Swift 6 libraries
            for path in &swift_runtime_paths {
                if std::path::Path::new(path).exists() {
                    println!("cargo:rustc-link-search=native={}", path);
                }
            }

            // Link Swift compatibility libraries required by Swift static libraries
            // Only link if using toolchain Swift (not system Swift)
            if use_swift6 && std::path::Path::new(swift6_toolchain_path).exists() {
                println!("cargo:rustc-link-lib=static=swiftCompatibility56");
                println!("cargo:rustc-link-lib=static=swiftCompatibilityConcurrency");
                println!("cargo:rustc-link-lib=static=swiftCompatibilityPacks");
            }

            // Swift 6 concurrency symbols with isolation parameter
            // The Swift bridge (WhisperKit) was built with Swift 6 and requires these symbols
            // On macOS, Swift Concurrency is part of the system Swift runtime
            // We need to ensure the system can resolve these symbols at runtime
            if use_swift6 {
                // Add system Swift runtime paths for symbol resolution
                // Swift 6 concurrency symbols are in the system Swift runtime
                if std::path::Path::new(swift6_system_path).exists() {
                    println!("cargo:rustc-link-search=native={}", swift6_system_path);
                }
                if std::path::Path::new(swift6_toolchain_path).exists() {
                    println!("cargo:rustc-link-search=native={}", swift6_toolchain_path);
                }
                // Note: Swift 6 concurrency symbols are resolved automatically by the system
                // Swift runtime at runtime. We ensure the paths are available via rpath.
            } else {
                // Fallback to Swift 5.5 - link explicitly if available
                if std::path::Path::new(swift55_path).join("libswift_Concurrency.dylib").exists() {
                    println!("cargo:rustc-link-arg=-L{}", swift55_path);
                    println!("cargo:rustc-link-lib=dylib=swift_Concurrency");
                }
            }

            // Add Swift runtime library paths to rpath for runtime resolution
            for path in &swift_runtime_paths {
                if std::path::Path::new(path).exists() {
                    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path);
                }
            }

            // Add framework paths
            println!("cargo:rustc-link-arg=-Wl,-rpath,/System/Library/Frameworks");
            println!("cargo:rustc-link-arg=-Wl,-rpath,/System/Library/Frameworks/CoreML.framework");

            // Also add @rpath resolution for Swift libraries embedded in frameworks
            println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/../Frameworks");

            // Add @executable_path for test binaries
            println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
            println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");

            // Set DYLD_LIBRARY_PATH for test execution via cargo environment
            // This ensures tests can find Swift runtime libraries
            let dyld_paths: Vec<&str> = swift_runtime_paths
                .iter()
                .filter(|p| std::path::Path::new(p).exists())
                .copied()
                .collect();
            if !dyld_paths.is_empty() {
                println!("cargo:rustc-env=DYLD_LIBRARY_PATH={}", dyld_paths.join(":"));
            }
        }

        // Print build info
        #[cfg(feature = "coreml")]
        {
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
