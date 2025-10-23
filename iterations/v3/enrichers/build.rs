fn main() {
    // Only link Swift bridge libraries when the feature is enabled
    // Note: We don't link during tests to avoid issues with cross-compilation
    #[cfg(all(target_os = "macos", feature = "swift-bridge", not(test)))]
    {
        println!("cargo:warning=Building with Swift bridge enabled");
        // Target ARM64 architecture for Apple Silicon
        let current_dir = std::env::current_dir().unwrap();
        let lib_path = current_dir.join("../coreml-bridge/.build/arm64-apple-macosx/release");
        println!("cargo:rustc-link-search=native={}", lib_path.display());
        println!("cargo:rustc-link-lib=static=CoreMLBridge");
        println!("cargo:rustc-link-lib=framework=CoreML");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Accelerate");
        println!("cargo:rustc-link-lib=framework=AVFoundation");
        println!("cargo:warning=Swift bridge linking configured");
    }
    #[cfg(not(all(target_os = "macos", feature = "swift-bridge", not(test))))]
    {
        println!("cargo:warning=Swift bridge NOT enabled - conditions not met");
    }
}