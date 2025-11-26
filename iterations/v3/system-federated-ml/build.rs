// @author: @darianrosebrook
// Build script for system-federated-ml
// Links Swift runtime libraries required by transitive dependencies (data-infrastructure -> WhisperKit)

fn main() {
    #[cfg(target_os = "macos")]
    {
        // Swift 6 is required for the new concurrency API with isolation parameter
        // WhisperKit (transitive dependency) was built with Swift 6 and requires these symbols
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

        // Add only the selected Swift runtime paths as native link search paths
        // This prevents mixing Swift 5.5 and Swift 6 libraries
        for path in &swift_runtime_paths {
            if std::path::Path::new(path).exists() {
                println!("cargo:rustc-link-search=native={}", path);
            }
        }

        // Swift 6 concurrency symbols with isolation parameter
        // WhisperKit requires Swift 6 concurrency symbols (withTaskGroup with isolation, etc.)
        // These symbols are part of the system Swift runtime on macOS
        if use_swift6 {
            // Add system Swift runtime paths for symbol resolution
            // Swift 6 concurrency symbols are resolved automatically by the system Swift runtime
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
                println!("cargo:rustc-link-arg=-lswift_Concurrency");
            }
        }

        // Link swiftCore from the selected Swift version
        if use_swift6 {
            if std::path::Path::new(swift6_toolchain_path).join("libswiftCore.dylib").exists() {
                println!("cargo:rustc-link-arg=-L{}", swift6_toolchain_path);
                println!("cargo:rustc-link-arg=-lswiftCore");
            }
        } else if std::path::Path::new(swift55_path).join("libswiftCore.dylib").exists() {
            println!("cargo:rustc-link-arg=-L{}", swift55_path);
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

