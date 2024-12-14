use clap::Parser;

use crate::{Cli, Command};

#[derive(Parser)]
pub(crate) struct Toml {
    /// Path to a workspace or package Cargo.toml file.
    ///
    /// Must include a [[workspace | package].metadata.rust-gpu.build] section where
    /// arguments to `cargo gpu build` are listed.
    ///
    /// Path arguments like `output-dir` and `shader-manifest` must be relative to
    /// the location of the Cargo.toml file.
    ///
    /// Example:
    ///
    /// ```toml
    ///     [package.metadata.rust-gpu.build.spirv-builder]
    ///     git = "https://github.com/Rust-GPU/rust-gpu.git"
    ///     rev = "0da80f8"
    ///
    ///     [package.metadata.rust-gpu.build]
    ///     output-dir = "shaders"
    ///     shader-manifest = "shaders/manifest.json"
    /// ```
    ///
    /// Calling `cargo gpu toml {path/to/Cargo.toml}` with a Cargo.toml that
    /// contains the example above would compile the crate and place the compiled
    /// `.spv` files and manifest in a directory "shaders".
    #[clap(default_value = "./Cargo.toml", verbatim_doc_comment)]
    path: std::path::PathBuf,
}

impl Toml {
    pub fn run(&self) {
        // Find the path to the toml file to use
        let path = if self.path.is_file() && self.path.ends_with(".toml") {
            self.path.clone()
        } else {
            let path = self.path.join("Cargo.toml");
            if path.is_file() {
                path
            } else {
                log::error!("toml file '{}' is not a file", self.path.display());
                panic!("toml file '{}' is not a file", self.path.display());
            }
        };

        log::info!("using toml file '{}'", path.display());

        // Determine if this is a workspace's Cargo.toml or a crate's Cargo.toml
        let contents = std::fs::read_to_string(&path).unwrap();
        let toml: toml::Table = toml::from_str(&contents).unwrap();

        fn get_metadata_rustgpu_table<'a>(
            toml: &'a toml::Table,
            toml_type: &'static str,
        ) -> Option<&'a toml::Table> {
            let workspace = toml.get(toml_type)?.as_table()?;
            let metadata = workspace.get("metadata")?.as_table()?;
            metadata.get("rust-gpu")?.as_table()
        }

        let (toml_type, table) = if toml.contains_key("workspace") {
            let table = get_metadata_rustgpu_table(&toml, "workspace")
                .unwrap_or_else(|| {
                    panic!(
                        "toml file '{}' is missing a [workspace.metadata.rust-gpu] table",
                        path.display()
                    );
                })
                .clone();
            ("workspace", table)
        } else if toml.contains_key("package") {
            let mut table = get_metadata_rustgpu_table(&toml, "package")
                .unwrap_or_else(|| {
                    panic!(
                        "toml file '{}' is missing a [package.metadata.rust-gpu] table",
                        path.display()
                    );
                })
                .clone();
            // Ensure the package name is included as the shader-crate parameter
            if !table.contains_key("shader-crate") {
                table.insert(
                    "shader-crate".to_string(),
                    format!("{}", path.parent().unwrap().display()).into(),
                );
            }
            ("package", table)
        } else {
            panic!("toml file '{}' must describe a workspace containing [workspace.metadata.rust-gpu.build] or a describe a crate with [package.metadata.rust-gpu.build]", path.display());
        };
        log::info!(
            "building with [{toml_type}.metadata.rust-gpu.build] section of the toml file at '{}'",
            path.display()
        );
        log::debug!("table: {table:#?}");

        let mut parameters = table
            .get("build")
            .unwrap_or_else(|| panic!("toml is missing the 'build' table"))
            .as_table()
            .unwrap_or_else(|| {
                panic!("toml file's '{toml_type}.metadata.rust-gpu.build' property is not a table")
            })
            .into_iter()
            .flat_map(|(k, v)| match v {
                toml::Value::String(s) => [format!("--{k}"), s.clone()],
                _ => {
                    let mut value = String::new();
                    let ser = toml::ser::ValueSerializer::new(&mut value);
                    serde::Serialize::serialize(v, ser).unwrap();
                    [format!("--{k}"), value]
                }
            })
            .collect::<Vec<_>>();
        parameters.insert(0, "cargo-gpu".to_string());
        parameters.insert(1, "build".to_string());

        let working_directory = path.parent().unwrap();
        log::info!(
            "issuing cargo commands from the working directory '{}'",
            working_directory.display()
        );
        std::env::set_current_dir(working_directory).unwrap();

        log::debug!("build parameters: {parameters:#?}");
        if let Cli {
            command: Command::Build(mut build),
            ..
        } = Cli::parse_from(parameters)
        {
            log::debug!("build: {build:?}");
            build.run();
        } else {
            log::error!("parameters found in [{toml_type}.metadata.rust-gpu.build] were not parameters to `cargo gpu build`");
            panic!("could not determin build command");
        }
    }
}
