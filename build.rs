// SPDX-License-Identifier: MIT OR Apache-2.0
use std::{env, fs, path::PathBuf};

fn main() {
    match env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("linux") => println!("cargo:rustc-link-arg-tests=-Wl,--export-dynamic"),
        Ok("macos") => println!("cargo:rustc-link-arg-tests=-Wl,-export_dynamic"),
        Ok("windows") => {
            let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
            let exports_path = manifest_dir.join("src/api/runtime/exports.rs");
            println!("cargo:rerun-if-changed={}", exports_path.display());

            let exports = fs::read_to_string(exports_path).expect("read N-API exports");
            for line in exports.lines() {
                let Some(symbol) = line.trim().strip_suffix(" as *const (),") else {
                    continue;
                };
                println!("cargo:rustc-link-arg-tests=/EXPORT:{symbol}");
            }
        }
        _ => {}
    }
}
