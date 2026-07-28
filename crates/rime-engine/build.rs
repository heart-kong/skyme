//! Build script for the `skyme-rime-engine` crate.
//!
//! Finds and links librime (librime / rimelib) on supported platforms.
//! If librime is not found, the crate still compiles for `cargo check`,
//! but `cargo build` will fail at the link step.
//!
//! # Environment variables
//!
//! - `RIME_LIB_DIR` — path to directory containing `librime.so` / `rime.dll` / `librime.dylib`.
//! - `RIME_INCLUDE_DIR` — path to directory containing `rime.h` (for bindgen).
//! - `RIME_USE_BINDGEN` — if set, run bindgen to generate fresh FFI declarations.
//!   Otherwise, the manually written `ffi.rs` declarations are used.
//!
//! On Windows, librime is typically installed via Weasel or a package manager.
//! On Linux, use `librime-dev` from your distribution or build from source.
//! On macOS, `brew install librime`.

fn main() {
    // ── Check for RIME_LIB_DIR ────────────────────────────────────────
    let lib_dir = std::env::var("RIME_LIB_DIR");
    let found = if let Ok(dir) = &lib_dir {
        println!("cargo:rustc-link-search=native={}", dir);
        link_rime();
        true
    } else {
        // Try common paths
        try_find_and_link()
    };

    if found {
        println!("cargo:rustc-cfg=rime_linked");
        println!("cargo:rerun-if-env-changed=RIME_LIB_DIR");
        println!("cargo:rerun-if-env-changed=RIME_INCLUDE_DIR");
    } else {
        println!("cargo:warning=librime not found. Set RIME_LIB_DIR to link against librime.");
        println!("cargo:warning=Without librime, cargo build will fail at the link step.");
    }

    // ── Optionally run bindgen ────────────────────────────────────────
    #[cfg(feature = "bindgen")]
    if std::env::var("RIME_USE_BINDGEN").is_ok() {
        generate_bindgen();
    }
}

/// Emit `cargo:rustc-link-lib` directives for the rime library.
fn link_rime() {
    // The library name differs per platform:
    //   Windows: rime.dll  →  rustc-link-lib=rime
    //   Linux:   librime.so  →  rustc-link-lib=rime
    //   macOS:   librime.dylib  →  rustc-link-lib=rime
    #[cfg(target_os = "windows")]
    println!("cargo:rustc-link-lib=dylib=rime");

    #[cfg(not(target_os = "windows"))]
    println!("cargo:rustc-link-lib=dylib=rime");
}

/// Search common paths for librime.
fn try_find_and_link() -> bool {
    let candidates = [
        // Windows — Weasel install
        r"C:\Program Files (x86)\Weasel",
        r"C:\Program Files\Weasel",
        // Linux — default system/lib directories
        "/usr/lib",
        "/usr/lib/x86_64-linux-gnu",
        "/usr/lib/aarch64-linux-gnu",
        "/usr/local/lib",
        // macOS — Homebrew
        "/usr/local/lib",
        "/opt/homebrew/lib",
        // Common build-from-source location
        "/usr/local/lib/librime",
    ];

    let lib_name = if cfg!(target_os = "windows") {
        "rime.dll"
    } else if cfg!(target_os = "macos") {
        "librime.dylib"
    } else {
        "librime.so"
    };

    for dir in &candidates {
        let path = std::path::Path::new(dir).join(lib_name);
        if path.exists() {
            println!("cargo:rustc-link-search=native={}", dir);
            link_rime();
            println!("cargo:warning=Found librime at {}", path.display());
            return true;
        }
    }

    // Also check via pkg-config.
    if pkg_config::Config::new()
        .atleast_version("1.0")
        .cargo_metadata(true)
        .print_system_libs(true)
        .probe("rime")
        .is_ok()
    {
        return true;
    }

    false
}

/// Generate FFI bindings using bindgen (requires librime headers).
#[cfg(feature = "bindgen")]
fn generate_bindgen() {
    let include_dir = std::env::var("RIME_INCLUDE_DIR").unwrap_or_else(|_| {
        // Try common rime_api.h locations.
        for dir in &["/usr/include", "/usr/local/include", "/opt/homebrew/include"] {
            if std::path::Path::new(dir).join("rime_api.h").exists() {
                return dir.to_string();
            }
        }
        "/usr/include".into()
    });

    let bindings = bindgen::Builder::default()
        .header(format!("{}/rime_api.h", include_dir))
        .allowlist_function("Rime.*")
        .allowlist_type("Rime.*")
        .generate()
        .expect("Unable to generate librime bindings with bindgen");

    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out.join("rime_ffi.rs"))
        .expect("Failed to write generated bindings");

    println!("cargo:warning=Regenerated FFI bindings in {}", out.display());
}
