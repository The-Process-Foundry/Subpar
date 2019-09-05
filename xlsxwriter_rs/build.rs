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
        .header("headers/app.h")
        .header("headers/chart.h")
        .header("headers/chartsheet.h")
        .header("headers/common.h")
        .header("headers/content_types.h")
        .header("headers/core.h")
        .header("headers/custom.h")
        .header("headers/drawing.h")
        .header("headers/format.h")
        .header("headers/hash_table.h")
        .header("headers/packager.h")
        .header("headers/relationships.h")
        .header("headers/shared_strings.h")
        .header("headers/styles.h")
        .header("headers/theme.h")
        .header("headers/utility.h")
        .header("headers/third_party/ioapi.h")
        .header("headers/third_party/queue.h")
        .header("headers/third_party/tmpfileplus.h")
        .header("headers/third_party/tree.h")
        .header("headers/third_party/zip.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
