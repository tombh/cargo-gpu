//! Build script that copies the built `librustc_codegen_spirv.dylib` into
//! a known location.
// TODO: remove the dependency on spirv-builder and subsequently rustc_codegen_spirv.

use std::str::FromStr;

fn dylib_filename() -> String {
    format!(
        "{}rustc_codegen_spirv{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    )
}

fn find_file(dir: &std::path::Path, filename: &str) -> std::path::PathBuf {
    let mut path = dir.join(filename);
    while !path.exists() {
        if let Some(parent) = path.parent().and_then(|p| p.parent()) {
            path = parent.join(filename);
        } else {
            break;
        }
    }

    if !path.exists() {
        panic!("could not find file '{filename}'");
    }
    path
}

fn main() {
    let out_dir = std::path::PathBuf::from(
        std::env::var("OUT_DIR").expect("Environment variable OUT_DIR was not set"),
    );
    let built_dylib_path = find_file(&out_dir, &dylib_filename());

    let dest_dir = env_home::env_home_dir()
        .expect("home directory is not set")
        .join(".rust-gpu");
    std::fs::create_dir_all(&dest_dir).expect("could not create ~/.rust-gpu directory");
    let dest_path = dest_dir.join(dylib_filename());

    std::fs::copy(&built_dylib_path, &dest_path).unwrap_or_else(|e| {
        panic!(
            "failed to copy dylib '{}' to destination '{}': {e}",
            built_dylib_path.display(),
            dest_path.display()
        )
    });

    println!("cargo:rustc-env=DYLIB_PATH={}", dest_path.display());

    let rust_toolchain_toml = include_str!("../rust-toolchain.toml");
    let table = toml::Table::from_str(rust_toolchain_toml)
        .unwrap_or_else(|e| panic!("could not parse rust-toolchain.toml: {e}"));
    let toolchain = table
        .get("toolchain")
        .unwrap_or_else(|| panic!("rust-toolchain.toml is missing 'toolchain'"))
        .as_table()
        .unwrap();
    let channel = toolchain
        .get("channel")
        .unwrap_or_else(|| panic!("rust-toolchain.toml is missing 'channel'"));
    println!("cargo:rustc-env=RUSTC_NIGHTLY_CHANNEL={channel}");
}
