//! Compress the Hungarian spelling dictionary (`assets/proofing/hu-hu.aff`, `hu-hu.dic`, Magyar
//! Ispell, MPL-2.0) into `OUT_DIR` with raw deflate, so the binary carries ~0.9 MB instead of
//! ~4 MB. A missing or unreadable file gives an empty output and a warning: the build still
//! links, and Hungarian spelling reports itself unavailable at run time. Reads only local files.

use std::path::{Path, PathBuf};

fn main() {
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/proofing");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap_or_default());
    for name in ["hu-hu.aff", "hu-hu.dic"] {
        let src = assets.join(name);
        println!("cargo::rerun-if-changed={}", src.display());
        let packed = match std::fs::read(&src) {
            Ok(bytes) => miniz_oxide::deflate::compress_to_vec(&bytes, 9),
            Err(e) => {
                println!("cargo::warning={}: {e}; Hungarian spelling is off in this build", src.display());
                Vec::new()
            }
        };
        let dst = out.join(format!("{name}.deflate"));
        if let Err(e) = std::fs::write(&dst, packed) {
            println!("cargo::error=writing {}: {e}", dst.display());
        }
    }
}
