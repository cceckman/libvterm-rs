extern crate cc;
extern crate glob;

use glob::glob;
use std::env;

fn main() {
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
