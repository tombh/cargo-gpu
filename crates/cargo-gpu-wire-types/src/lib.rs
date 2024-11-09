//! Wire types for `cargo-gpu` and `spirv-builder-cli`.

/// Shader source and entry point that can be used to create shader linkage.
#[derive(serde::Serialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Linkage {
    pub source_path: std::path::PathBuf,
    pub entry_point: String,
    pub wgsl_entry_point: String,
}

impl Linkage {
    pub fn new(entry_point: impl AsRef<str>, source_path: impl AsRef<std::path::Path>) -> Self {
        Self {
            source_path: source_path.as_ref().to_path_buf(),
            wgsl_entry_point: entry_point.as_ref().replace("::", ""),
            entry_point: entry_point.as_ref().to_string(),
        }
    }

    pub fn fn_name(&self) -> &str {
        self.entry_point.split("::").last().unwrap()
    }
}

pub mod spirv_builder_cli {
    //! `spirv-builder-cli` interface types.
    //!
    //! This module is exposed here to keep `spirv-build-cli`'s source
    //! as small as possible. It is not expected to be used by any
    //! user-facing code.

    /// `spirv-builder-cli` command line interface.
    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Args {
        /// Path to rustc_codegen_spirv dylib.
        pub dylib_path: std::path::PathBuf,

        /// Directory containing the shader crate to compile.
        pub shader_crate: std::path::PathBuf,

        /// Shader target.
        pub shader_target: String,

        /// Set cargo default-features.
        pub no_default_features: bool,

        /// Set cargo features.
        pub features: Vec<String>,

        /// Path to the output directory for the compiled shaders.
        pub output_dir: std::path::PathBuf,

        /// Dry run or not
        pub dry_run: bool,
    }

    /// A built shader entry-point, used in `spirv-builder-cli` to generate
    /// a `build-manifest.json` used by `cargo-gpu`.
    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ShaderModule {
        pub entry: String,
        pub path: std::path::PathBuf,
    }

    impl ShaderModule {
        pub fn new(entry: impl AsRef<str>, path: impl AsRef<std::path::Path>) -> Self {
            Self {
                entry: entry.as_ref().into(),
                path: path.as_ref().into(),
            }
        }
    }
}
