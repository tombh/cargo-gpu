//! This program builds rust-gpu shader crates and writes generated spv files
//! into the main source repo.
use std::io::Write;

use clap::Parser;
use spirv_builder::{CompileResult, MetadataPrintout, ModuleResult, SpirvBuilder};

use cargo_gpu::linkage;

const RUSTC_CODEGEN_SPIRV_PATH: &str = std::env!("DYLIB_PATH");

const RUSTC_NIGHTLY_CHANNEL: &str = std::env!("RUSTC_NIGHTLY_CHANNEL");

fn set_rustup_toolchain() {
    log::trace!(
        "setting RUSTUP_TOOLCHAIN = '{}'",
        RUSTC_NIGHTLY_CHANNEL.trim_matches('"')
    );
    std::env::set_var("RUSTUP_TOOLCHAIN", RUSTC_NIGHTLY_CHANNEL.trim_matches('"'));
}

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    /// Directory containing the shader crate to compile.
    #[clap(long, default_value = "./")]
    pub shader_crate: std::path::PathBuf,

    /// Shader target.
    #[clap(long, default_value = "spirv-unknown-vulkan1.2")]
    pub shader_target: String,

    /// Path to the output JSON manifest file where the paths to .spv files
    /// and the names of their entry points will be saved.
    #[clap(long)]
    pub shader_manifest: Option<std::path::PathBuf>,

    /// Set cargo default-features.
    #[clap(long)]
    pub no_default_features: bool,

    /// Set cargo features.
    #[clap(long)]
    pub features: Vec<String>,

    /// Path to the output directory for the compiled shaders.
    #[clap(long, short, default_value = "./")]
    pub output_dir: std::path::PathBuf,

    /// If set the shaders will be compiled but not put into place.
    #[clap(long, short)]
    pub dry_run: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder().init();

    set_rustup_toolchain();

    let Cli {
        shader_crate,
        shader_target,
        no_default_features,
        features,
        output_dir,
        shader_manifest: output_manifest,
        dry_run,
    } = Cli::parse_from(std::env::args().filter(|p| {
        // Calling cargo-gpu as the cargo subcommand "cargo gpu" passes "gpu"
        // as the first parameter, which we want to ignore.
        p != "gpu"
    }));

    std::fs::create_dir_all(&output_dir).unwrap();

    assert!(
        shader_crate.exists(),
        "shader crate '{}' does not exist. (Current dir is '{}')",
        shader_crate.display(),
        std::env::current_dir().unwrap().display()
    );

    let start = std::time::Instant::now();

    let CompileResult {
        entry_points,
        module,
    } = {
        let mut builder = SpirvBuilder::new(shader_crate, &shader_target)
            .rustc_codegen_spirv_location(RUSTC_CODEGEN_SPIRV_PATH)
            .print_metadata(MetadataPrintout::None)
            .multimodule(true);

        if no_default_features {
            log::info!("setting cargo --no-default-features");
            builder = builder.shader_crate_default_features(false);
        }
        if !features.is_empty() {
            log::info!("setting --features {features:?}");
            builder = builder.shader_crate_features(features);
        }

        builder.build()?
    };

    let dir = output_dir;
    let mut shaders = vec![];
    match module {
        ModuleResult::MultiModule(modules) => {
            assert!(!modules.is_empty(), "No shader modules to compile");
            for (entry, filepath) in modules.into_iter() {
                let path = dir.join(filepath.file_name().unwrap());
                if !dry_run {
                    std::fs::copy(filepath, &path).unwrap();
                }
                shaders.push(linkage::Linkage::new(entry, path));
            }
        }
        ModuleResult::SingleModule(filepath) => {
            let path = dir.join(filepath.file_name().unwrap());
            if !dry_run {
                std::fs::copy(filepath, &path).unwrap();
            }
            for entry in entry_points {
                shaders.push(linkage::Linkage::new(entry, path.clone()));
            }
        }
    }

    // Write the shader manifest json file
    if !dry_run {
        if let Some(manifest_path) = output_manifest {
            // Sort the contents so the output is deterministic
            shaders.sort();
            // UNWRAP: safe because we know this always serializes
            let json = serde_json::to_string_pretty(&shaders).unwrap();
            let mut file = std::fs::File::create(&manifest_path).unwrap_or_else(|e| {
                log::error!(
                    "could not create shader manifest file '{}': {e}",
                    manifest_path.display(),
                );
                panic!("{e}")
            });
            file.write_all(json.as_bytes()).unwrap_or_else(|e| {
                log::error!(
                    "could not write shader manifest file '{}': {e}",
                    manifest_path.display(),
                );
                panic!("{e}")
            });
        }
    }

    let end = std::time::Instant::now();
    log::debug!("finished in {:?}", (end - start));

    Ok(())
}
