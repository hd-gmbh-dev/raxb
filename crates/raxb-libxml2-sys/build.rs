use std::path::{Path, PathBuf};

fn main() {
    let mut cfg = cmake::Config::new(Path::new("./third_party/libxml2"));
    cfg.define("BUILD_SHARED_LIBS", "OFF");
    cfg.define("LIBXML2_WITH_ICONV", "OFF");
    cfg.define("LIBXML2_WITH_LZMA", "OFF");
    cfg.define("LIBXML2_WITH_PYTHON", "OFF");
    cfg.define("LIBXML2_WITH_ZLIB", "OFF");
    cfg.define("LIBXML2_WITH_CATALOG", "OFF");
    cfg.define("LIBXML2_WITH_HTTP", "OFF");
    cfg.define("LIBXML2_WITH_HTML", "OFF");
    cfg.define("LIBXML2_WITH_MODULES", "OFF");
    cfg.define("LIBXML2_WITH_WRITER", "OFF");
    cfg.define("LIBXML2_WITH_TESTS", "OFF");
    cfg.define("LIBXML2_WITH_PROGRAMS", "OFF");
    cfg.define("LIBXML2_WITH_OUTPUT", "ON");
    let libxml = cfg.build();
    println!(
        "cargo:rustc-link-search=native={}",
        libxml.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=xml2");
    let libxml_absolute_path = std::fs::canonicalize(Path::new("./third_party/libxml2/include")).unwrap();
    let libxml_build_absolute_path = std::fs::canonicalize(libxml.join("include")).unwrap();
    eprintln!("LIBXML PATH: {:?}", libxml_absolute_path.display());
    let bindings = bindgen::Builder::default()
        // .rust_target(bindgen::RustTarget::Nightly)
        // .detect_include_paths(false)
        .clang_arg(&format!("-I{}", libxml_absolute_path.display()))
        .clang_arg(&format!("-I{}", libxml_build_absolute_path.display()))
        // .clang_args(["-x", "c", "-std=c11"])
        // The input header we would like to generate
        // bindings for.
        .header("bindings.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_type("xmlCharEncoding_XML_CHAR_ENCODING_UTF8")
        .allowlist_function("xmlInitParser")
        .allowlist_function("xmlRegisterInputCallbacks")
        .allowlist_function("xmlParserInputBufferCreateMem")
        .allowlist_function("xmlSchemaFreeParserCtxt")
        .allowlist_function("xmlSchemaFreeValidCtxt")
        .allowlist_function("xmlSchemaNewMemParserCtxt")
        .allowlist_function("xmlSchemaNewValidCtxt")
        .allowlist_function("xmlSchemaParse")
        .allowlist_function("xmlSchemaSetValidStructuredErrors")
        .allowlist_function("xmlSchemaValidateStream")
        .allowlist_var("xmlSchemaParserCtxtPtr")
        .allowlist_var("xmlSchemaPtr")
        .allowlist_var("xmlSchemaValidCtxtPtr")
        .allowlist_var("xmlSAXHandler")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}