// SPDX-License-Identifier: MIT OR Apache-2.0
use std::env;

fn main() {
    match env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("linux") => println!("cargo:rustc-link-arg-tests=-Wl,--export-dynamic"),
        Ok("macos") => println!("cargo:rustc-link-arg-tests=-Wl,-export_dynamic"),
        _ => {}
    }
}
