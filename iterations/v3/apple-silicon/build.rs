//! Build script for apple-silicon crate
//!
//! Configures torch-sys to link against the libtorch installation
//! when torch features are enabled.

use std::env;
use std::path::PathBuf;

fn main() {
    // Check if download-libtorch feature is enabled
    let download_libtorch = env::var("CARGO_FEATURE_DOWNLOAD_LIBTORCH").is_ok();

    if download_libtorch {
        // Let tch download its own libtorch version - do not interfere
        println!("cargo:warning=Using download-libtorch feature - will download compatible libtorch version automatically");
        // Do not set LIBTORCH or link libraries when download-libtorch is enabled
    } else {
        // Use our existing libtorch installation
        let libtorch_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("libtorch");

        if libtorch_path.exists() {
            println!("cargo:rustc-env=LIBTORCH={}", libtorch_path.display());
            println!("cargo:rustc-link-search=native={}/lib", libtorch_path.display());
            println!("cargo:rustc-link-lib=dylib=torch");
            println!("cargo:rustc-link-lib=dylib=torch_cpu");
            println!("cargo:rustc-link-lib=dylib=c10");
            println!("cargo:rerun-if-changed={}", libtorch_path.display());
        } else {
            println!("cargo:warning=libtorch not found at {}, torch features will be disabled", libtorch_path.display());
        }
    }

    println!("cargo:rerun-if-env-changed=LIBTORCH");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_DOWNLOAD_LIBTORCH");
}



