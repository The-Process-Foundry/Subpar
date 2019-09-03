extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-search=/usr/local/lib");
    println!("cargo:rustc-link-lib=xlsxwriter");
    // TODO: Add libxlsxwriter as a git submodule and use cc module to build it
    // https://doc.rust-lang.org/cargo/reference/build-scripts.html#case-study-building-some-native-code

    let bindings = bindgen::Builder::default()
        .header("headers/xmlwriter.h")
        .header("headers/workbook.h")
        .header("headers/worksheet.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
