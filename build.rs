extern crate bindgen;

const TYPES: &'static [&'static str] = &[

    "Purpose", "Function", "CameraType", "Orientation", "State",
    "VideoWriter", "Action", "EventCloseMode", "SharedData",
    "TriggerState", "TriggerData", "VideoStoreData",

];

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    let builder = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .derive_debug(true)
        .whitelist_recursively(false);

    let builder = TYPES.iter().fold(builder, |b, typ| b.whitelist_type(typ));
    let bindings = builder
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
