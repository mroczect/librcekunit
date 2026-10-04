use std::env;
use std::process;

const HEADER_PATH: &str = "include/librcekunit.h";

fn main() {
    let crate_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(v) => v,
        Err(e) => {
            eprintln!("build.rs: CARGO_MANIFEST_DIR tidak di-set: {e}");
            process::exit(1);
        }
    };

    let config = match cbindgen::Config::from_file("cbindgen.toml") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("build.rs: gagal baca cbindgen.toml: {e}");
            process::exit(1);
        }
    };

    let bindings = match cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(config)
        .generate()
    {
        Ok(b) => b,
        Err(e) => {
            eprintln!("build.rs: cbindgen gagal generate: {e}");
            process::exit(1);
        }
    };

    let _changed = bindings.write_to_file(HEADER_PATH);

    println!("cargo:rerun-if-changed=cbindgen.toml");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/client.rs");
    println!("cargo:rerun-if-changed=src/error.rs");
    println!("cargo:rerun-if-changed=src/buffer.rs");
}
