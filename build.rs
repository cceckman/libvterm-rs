use glob::glob;
use std::env;
use std::path::PathBuf;

/// Generate Rust types and `extern` declarations from the vterm headers.
fn bindgen() {
    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    // println!("cargo:rustc-link-lib=bz2");

    // Invalidate the built crate whenever the header changes...
    println!("cargo:rerun-if-changed=vendor/libvterm/include/vterm.h");

    let bindings = bindgen::Builder::default()
        .header("vendor/libvterm/include/vterm.h")
        // ...or any of the included headers.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

/// Invoke a C compiler to build the libvterm object file, libvterm.a.
fn build_libvterm() {
    let vendor_path = env::current_dir().unwrap().join("vendor");

    let mut config = cc::Build::new();
    for file in glob(vendor_path.join("libvterm/src/*.c").to_str().unwrap()).unwrap() {
        config.file(file.unwrap());
    }
    config.file(vendor_path.join("rusty_shims.c").to_str().unwrap());
    config.include(vendor_path.join("libvterm/include").to_str().unwrap());
    config.include(vendor_path.join("libvterm/src").to_str().unwrap());
    config.compile("libvterm.a");
}

fn main() {
    bindgen();
    build_libvterm();
}
