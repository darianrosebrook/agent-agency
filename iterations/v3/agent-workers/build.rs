// @author: @darianrosebrook
//
// Build script for agent-workers
// Links the Swift CoreML bridge library for examples and binaries

fn main() {
    #[cfg(target_os = "macos")]
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

                // Swift runtime paths - /usr/lib/swift first for Swift 6 concurrency symbols
                let swift_runtime_paths = vec![
                    "/usr/lib/swift",
                    "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx",
                    "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.5/macosx",
                ];

                for path in &swift_runtime_paths {
                    if std::path::Path::new(path).exists() {
                        println!("cargo:rustc-link-search=native={}", path);
                    }
                }

                // Link Swift compatibility libraries
                let swift_compat_path = "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx";
                if std::path::Path::new(swift_compat_path).exists() {
                    println!("cargo:rustc-link-lib=static=swiftCompatibility56");
                    println!("cargo:rustc-link-lib=static=swiftCompatibilityConcurrency");
                    println!("cargo:rustc-link-lib=static=swiftCompatibilityPacks");
                }

                // Link Swift Concurrency from system (contains Swift 6 isolation symbols)
                println!("cargo:rustc-link-arg=-L/usr/lib/swift");
                println!("cargo:rustc-link-lib=dylib=swift_Concurrency");

                // Link swiftCore
                let swift_core_lib = format!("{}/libswiftCore.dylib", swift_compat_path);
                if std::path::Path::new(&swift_core_lib).exists() {
                    println!("cargo:rustc-link-arg=-L{}", swift_compat_path);
                    println!("cargo:rustc-link-arg={}", swift_core_lib);
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
}

