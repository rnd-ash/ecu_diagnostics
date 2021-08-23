extern crate cbindgen;

use cbindgen::{Config, Language};
use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_file = PathBuf::from("")
        .join(format!("{}.hpp", package_name))
        .display()
        .to_string();

    let mut config = Config {
        include_guard: Some(String::from("ECU_DIAG_H_")),
        namespace: Some(String::from("ecu_diagnostics")),
        language: Language::Cxx,
        ..Default::default()
    };
    config.parse.parse_deps = true;
    config.parse.include = Some(vec!["ecu_diagnostics".into()]);

    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write_to_file(&output_file);
}
