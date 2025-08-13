use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    // Generate C header file
    let config = cbindgen::Config::default();
    
    cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(config)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_dir.join("wavecore.h"));
    
    // Copy header to include directory
    let include_dir = PathBuf::from(&crate_dir).join("..").join("include");
    if include_dir.exists() {
        std::fs::copy(
            out_dir.join("wavecore.h"),
            include_dir.join("wavecore.h")
        ).expect("Failed to copy header file");
    }
    
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=../include/wavecore.h");
} 