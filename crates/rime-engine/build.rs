//! Build script for `skyme-rime-engine`.
//!
//! librime is loaded dynamically at runtime via `libloading`.
//! No compile-time linking is needed.
//!
//! # Bindgen (optional)
//!
//! Set `RIME_USE_BINDGEN=1` and enable the `bindgen` feature to regenerate
//! FFI declarations from `rime_api.h` headers. The current hand-written
//! declarations in `ffi_structs.rs.in` are the default.

fn main() {
    #[cfg(feature = "bindgen")]
    if std::env::var("RIME_USE_BINDGEN").is_ok() {
        generate_bindgen();
    }
}

#[cfg(feature = "bindgen")]
fn generate_bindgen() {
    let include_dir = std::env::var("RIME_INCLUDE_DIR").unwrap_or_else(|_| {
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
    bindings.write_to_file(out.join("rime_ffi.rs"))
        .expect("Failed to write generated bindings");
    println!("cargo:warning=Regenerated FFI bindings in {}", out.display());
}
