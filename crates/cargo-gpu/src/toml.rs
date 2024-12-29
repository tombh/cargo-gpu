//! Build a shader based on the data in the `[package.metadata.rust-gpu.build.spirv-builder]` section of
//! a shader's `Cargo.toml`.

use anyhow::Context as _;
use clap::Parser;

use crate::{Cli, Command};

/// `cargo gpu toml`
#[derive(Debug, Parser)]
pub struct Toml {
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
    /// Entrypoint
    pub fn run(&self) -> anyhow::Result<()> {
        let (path, toml) = Self::parse_cargo_toml(self.path.clone())?;
        let working_directory = path
            .parent()
            .context("Couldn't find parent for shader's `Cargo.toml`")?;

        // Determine if this is a workspace's Cargo.toml or a crate's Cargo.toml
        let (toml_type, table) = if toml.contains_key("workspace") {
            let table = Self::get_metadata_rustgpu_table(&toml, "workspace")
                .with_context(|| {
                    format!(
                        "toml file '{}' is missing a [workspace.metadata.rust-gpu] table",
                        path.display()
                    )
                })?
                .clone();
            ("workspace", table)
        } else if toml.contains_key("package") {
            let mut table = Self::get_metadata_rustgpu_table(&toml, "package")
                .with_context(|| {
                    format!(
                        "toml file '{}' is missing a [package.metadata.rust-gpu] table",
                        path.display()
                    )
                })?
                .clone();
            // Ensure the package name is included as the shader-crate parameter
            if !table.contains_key("shader-crate") {
                table.insert(
                    "shader-crate".to_owned(),
                    format!("{}", working_directory.display()).into(),
                );
            }
            ("package", table)
        } else {
            anyhow::bail!("toml file '{}' must describe a workspace containing [workspace.metadata.rust-gpu.build] or a describe a crate with [package.metadata.rust-gpu.build]", path.display());
        };

        log::info!(
            "building with [{toml_type}.metadata.rust-gpu.build] section of the toml file at '{}'",
            path.display()
        );
        log::debug!("table: {table:#?}");

        log::info!(
            "issuing cargo commands from the working directory '{}'",
            working_directory.display()
        );
        std::env::set_current_dir(working_directory)?;

        let parameters = construct_build_parameters_from_toml_table(toml_type, &table)?;
        log::debug!("build parameters: {parameters:#?}");
        if let Cli {
            command: Command::Build(mut build),
        } = Cli::parse_from(parameters)
        {
            log::debug!("build: {build:?}");
            build.run()?;
        } else {
            log::error!("parameters found in [{toml_type}.metadata.rust-gpu.build] were not parameters to `cargo gpu build`");
            anyhow::bail!("could not determin build command");
        }

        Ok(())
    }

    /// Parse the contents of the shader's `Cargo.toml`
    pub fn parse_cargo_toml(
        mut path: std::path::PathBuf,
    ) -> anyhow::Result<(std::path::PathBuf, toml::Table)> {
        // Find the path to the toml file to use
        let parsed_path = if path.is_file() && path.ends_with(".toml") {
            path
        } else {
            path = path.join("Cargo.toml");
            if path.is_file() {
                path
            } else {
                log::error!("toml file '{}' is not a file", path.display());
                anyhow::bail!("toml file '{}' is not a file", path.display());
            }
        };

        log::info!("using toml file '{}'", parsed_path.display());

        let contents = std::fs::read_to_string(&parsed_path)?;
        let toml: toml::Table = toml::from_str(&contents)?;

        Ok((parsed_path, toml))
    }

    /// Parse the `[package.metadata.rust-gpu]` section.
    fn get_metadata_rustgpu_table<'toml>(
        toml: &'toml toml::Table,
        toml_type: &'static str,
    ) -> Option<&'toml toml::Table> {
        let workspace = toml.get(toml_type)?.as_table()?;
        let metadata = workspace.get("metadata")?.as_table()?;
        metadata.get("rust-gpu")?.as_table()
    }
}

/// Construct the cli parameters to run a `cargo gpu build` command from a TOML table.
fn construct_build_parameters_from_toml_table(
    toml_type: &str,
    table: &toml::map::Map<String, toml::Value>,
) -> Result<Vec<String>, anyhow::Error> {
    let build_table = table
        .get("build")
        .with_context(|| "toml is missing the 'build' table")?
        .as_table()
        .with_context(|| {
            format!("toml file's '{toml_type}.metadata.rust-gpu.build' property is not a table")
        })?;
    let mut parameters: Vec<String> = build_table
        .into_iter()
        .map(|(key, val)| -> anyhow::Result<Vec<String>> {
            Ok(match val {
                toml::Value::String(string) => vec![format!("--{key}"), string.clone()],
                toml::Value::Boolean(truthy) => {
                    if *truthy {
                        vec![format!("--{key}")]
                    } else {
                        vec![]
                    }
                }
                toml::Value::Integer(_)
                | toml::Value::Float(_)
                | toml::Value::Datetime(_)
                | toml::Value::Array(_)
                | toml::Value::Table(_) => {
                    let mut value = String::new();
                    let ser = toml::ser::ValueSerializer::new(&mut value);
                    serde::Serialize::serialize(val, ser)?;
                    vec![format!("--{key}"), value]
                }
            })
        })
        .collect::<anyhow::Result<Vec<Vec<String>>>>()?
        .into_iter()
        .flatten()
        .collect();
    parameters.insert(0, "cargo-gpu".to_owned());
    parameters.insert(1, "build".to_owned());
    Ok(parameters)
}
