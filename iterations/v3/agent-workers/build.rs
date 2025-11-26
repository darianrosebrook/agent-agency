// @author: @darianrosebrook
//
// Build script for agent-workers
// Links the Swift CoreML bridge library for examples and binaries
// Only links when the coreml feature is enabled to avoid Swift toolchain issues

fn main() {
    // CoreML bridge linking is disabled by default to avoid Swift toolchain version issues
    // The Swift 6 concurrency API uses different function signatures than Swift 5.5
    // Enable via the coreml feature when Swift 6 runtime is available
    
    #[cfg(all(target_os = "macos", feature = "coreml"))]
    {
        // Path to Swift bridge: agent-workers -> v3 -> iterations -> agent-agency -> models/...
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
            // Check for the library in the architecture-specific build directory
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

            if lib_path.exists() {
                println!("cargo:rustc-link-search=native={}", build_dir.display());
                println!("cargo:rustc-link-lib=static=CoreMLBridge");

                // Link frameworks
                println!("cargo:rustc-link-lib=framework=CoreML");
                println!("cargo:rustc-link-lib=framework=Metal");
                println!("cargo:rustc-link-lib=framework=Foundation");

                // Swift 6 is required for the new concurrency API with isolation parameter
                // Prioritize Swift 6 libraries to avoid conflicts with Swift 5.5
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

                // Add only the selected Swift runtime paths
                for path in &swift_runtime_paths {
                    if std::path::Path::new(path).exists() {
                        println!("cargo:rustc-link-search=native={}", path);
                    }
                }

                // Link Swift compatibility libraries (only if using toolchain Swift 6)
                if use_swift6 && std::path::Path::new(swift6_toolchain_path).exists() {
                    println!("cargo:rustc-link-lib=static=swiftCompatibility56");
                    println!("cargo:rustc-link-lib=static=swiftCompatibilityConcurrency");
                    println!("cargo:rustc-link-lib=static=swiftCompatibilityPacks");
                }

                // Link Swift Concurrency - prefer Swift 6, avoid Swift 5.5 if Swift 6 is available
                if use_swift6 {
                    // Try system Swift 6 first
                    if std::path::Path::new(swift6_system_path).join("libswift_Concurrency.dylib").exists() {
                        println!("cargo:rustc-link-arg=-L{}", swift6_system_path);
                        println!("cargo:rustc-link-lib=dylib=swift_Concurrency");
                    } else if std::path::Path::new(swift6_toolchain_path).join("libswift_Concurrency.dylib").exists() {
                        println!("cargo:rustc-link-arg=-L{}", swift6_toolchain_path);
                        println!("cargo:rustc-link-lib=dylib=swift_Concurrency");
                    }
                } else {
                    // Fallback to Swift 5.5 only if Swift 6 is not available
                    if std::path::Path::new(swift55_path).join("libswift_Concurrency.dylib").exists() {
                        println!("cargo:rustc-link-arg=-L{}", swift55_path);
                        println!("cargo:rustc-link-lib=dylib=swift_Concurrency");
                    }
                }

                // Link swiftCore from the selected Swift version
                if use_swift6 {
                    if std::path::Path::new(swift6_toolchain_path).join("libswiftCore.dylib").exists() {
                        println!("cargo:rustc-link-arg=-L{}", swift6_toolchain_path);
                    }
                } else if std::path::Path::new(swift55_path).join("libswiftCore.dylib").exists() {
                    println!("cargo:rustc-link-arg=-L{}", swift55_path);
                }

                // Add rpaths for runtime resolution
                for path in &swift_runtime_paths {
                    if std::path::Path::new(path).exists() {
                        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path);
                    }
                }

                println!("cargo:rustc-link-arg=-Wl,-rpath,/System/Library/Frameworks");
                println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
            } else {
                // Fallback to generic release directory
                let fallback_dir = bridge_path.join(".build").join("release");
                if fallback_dir.join("libCoreMLBridge.a").exists() {
                    println!("cargo:rustc-link-search=native={}", fallback_dir.display());
                    println!("cargo:rustc-link-lib=static=CoreMLBridge");
                }
            }
        }
    }
    
    #[cfg(all(target_os = "macos", not(feature = "coreml")))]
    {
        println!("cargo:warning=CoreML bridge disabled - coreml feature not enabled");
    }
}
