fn main() {
    // -------------------------
    // Platform-specific OpenSSL
    // -------------------------
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=ssl");
        println!("cargo:rustc-link-lib=crypto");
    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=ssl");
        println!("cargo:rustc-link-lib=crypto");
        // Homebrew OpenSSL path
        println!("cargo:rustc-link-search=native=/usr/local/opt/openssl/lib");
    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=ssl");
        println!("cargo:rustc-link-lib=crypto");
    }

    // -------------------------
    // Build flash.c
    // -------------------------
    cc::Build::new()
        .file("c_utils/flash.c")
        .include("c_utils")
        .opt_level(3)          // -O3
        .flag("-march=native") // CPU-specific optimizations
        .flag("-funroll-loops")
        .flag("-Ofast")        // optional fast math
        .compile("flash");

    println!("cargo:rustc-link-lib=static=flash");

    // -------------------------
    // Build verify.c
    // -------------------------
    let mut verify_build = cc::Build::new();
    verify_build
        .file("c_utils/verify.c")
        .include("c_utils")
        .opt_level(3)
        .flag("-march=native")
        .flag("-funroll-loops")
        .flag("-Ofast");

    // macOS Homebrew OpenSSL headers
    if cfg!(target_os = "macos") {
        verify_build.include("/usr/local/opt/openssl/include");
    }

    verify_build.compile("verify");

    // Link the static verify library
    println!("cargo:rustc-link-lib=static=verify");

    // Make sure OpenSSL is linked for verify.c
    println!("cargo:rustc-link-lib=ssl");
    println!("cargo:rustc-link-lib=crypto");

    // -------------------------
    // Re-run build.rs if C files change
    // -------------------------
    println!("cargo:rerun-if-changed=c_utils/flash.c");
    println!("cargo:rerun-if-changed=c_utils/verify.c");
    println!("cargo:rerun-if-changed=c_utils/verify.h");
}
