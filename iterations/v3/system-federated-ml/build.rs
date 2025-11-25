// @author: @darianrosebrook
// Build script for system-federated-ml
// Links Swift runtime libraries required by transitive dependencies (data-infrastructure -> WhisperKit)

fn main() {
    #[cfg(target_os = "macos")]
    {
        // Swift runtime library paths - order matters for resolution
        // swift-5.5/macosx contains libswift_Concurrency.dylib which is required
        // by Swift async/await features used in WhisperKit (transitive dependency)
        let swift_runtime_paths = vec![
            "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.5/macosx",
            "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx",
            "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-6.0/macosx",
            "/usr/lib/swift",
        ];

        // Add all existing paths as native link search paths
        // This ensures dyld can find Swift runtime libraries at test time
        for path in &swift_runtime_paths {
            if std::path::Path::new(path).exists() {
                println!("cargo:rustc-link-search=native={}", path);
            }
        }

        // Link Swift Concurrency runtime dynamically - required for Swift async/await features
        // This is needed by WhisperKit and other Swift packages that use async/await
        // The library must be linked for the symbols to be resolved at link time
        let swift_concurrency_path = "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.5/macosx";
        if std::path::Path::new(swift_concurrency_path)
            .join("libswift_Concurrency.dylib")
            .exists()
        {
            // Use -L to add the library search path and -l to link the library
            println!(
                "cargo:rustc-link-arg=-L{}",
                swift_concurrency_path
            );
            println!("cargo:rustc-link-arg=-lswift_Concurrency");
        }

        // Also link swiftCore which is required by Swift Concurrency
        let swift_core_path = "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx";
        if std::path::Path::new(swift_core_path)
            .join("libswiftCore.dylib")
            .exists()
        {
            println!("cargo:rustc-link-arg=-L{}", swift_core_path);
            println!("cargo:rustc-link-arg=-lswiftCore");
        }

        // Add Swift runtime library paths to rpath for runtime resolution
        for path in &swift_runtime_paths {
            if std::path::Path::new(path).exists() {
                println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path);
            }
        }

        // Add framework paths
        println!("cargo:rustc-link-arg=-Wl,-rpath,/System/Library/Frameworks");

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

    #[cfg(not(target_os = "macos"))]
    {
        // No Swift runtime linking needed on non-macOS platforms
    }
}

