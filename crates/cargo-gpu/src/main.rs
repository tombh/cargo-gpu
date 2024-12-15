//! Rust GPU shader crate builder.
//!
//! This program manages installations of `spirv-builder-cli` and `rustc_codegen_spirv`.
//! It uses these tools to compile Rust code into SPIR-V.
//!
//! # How it works
//!
//! In order to build shader crates, we must invoke cargo/rustc with a special backend
//! that performs the SPIR-V code generation. This backend is a dynamic library known
//! by its project name `rustc_codegen_spirv`. The name of the artifact itself is
//! OS-dependent.
//!
//! There are a lot of special flags to wrangle and so we use a command line program
//! that wraps `cargo` to perform the building of shader crates. This cli program is
//! called `spirv-builder-cli`, which itself is a cli wrapper around the `spirv-builder`
//! library.
//!
//! ## Where the binaries are
//!
//! `cargo-gpu` maintains different versions `spirv-builder-cli` and `rustc_codegen_spirv`
//! in a cache dir. The location is OS-dependent, for example on macOS it's in
//! `~/Library/Caches/rust-gpu`. Specific versions live inside the cache dir, prefixed
//! by their `spirv-builder` cargo dependency and rust toolchain pair.
//!
//! Building a specific "binary pair" of `spirv-builder-cli` and `rustc_codegen_spirv`
//! happens when there is no existing pair that matches the computed prefix, or if
//! a force rebuild is specified on the command line.
//!
//! ## Building the "binary pairs"
//!
//! The source of `spirv-builder-cli` lives alongside this source file, in crate that
//! is not included in the workspace. That same source code is also included statically
//! in **this** source file.
//!
//! When `spirv-builder-cli` needs to be built, a new directory is created in the cache
//! where the source to `spirv-builder-cli` is copied into, containing the specific cargo
//! dependency for `spirv-builder` and the matching rust toolchain channel.
//!
//! Then `cargo` is invoked in that cache directory to build the pair of artifacts, which
//! are then put into the top level of that cache directory.
//!
//! This pair of artifacts is then used to build shader crates.
//!
//! ## Building shader crates
//!
//! `cargo-gpu` takes a path to a shader crate to build, as well as a path to a directory
//! to put the compiled `spv` source files. It also takes a path to an output mainifest
//! file where all shader entry points will be mapped to their `spv` source files. This
//! manifest file can be used by build scripts (`build.rs` files) to generate linkage or
//! conduct other post-processing, like converting the `spv` files into `wgsl` files,
//! for example.

use builder::Build;
use clap::Parser as _;
use install::Install;
use show::Show;
use toml::Toml;

mod builder;
mod install;
mod show;
mod spirv;
mod toml;

fn main() {
    env_logger::builder().init();

    let args = std::env::args()
        .filter(|arg| {
            // Calling cargo-gpu as the cargo subcommand "cargo gpu" passes "gpu"
            // as the first parameter, which we want to ignore.
            arg != "gpu"
        })
        .collect::<Vec<_>>();
    log::trace!("args: {args:?}");
    let cli = Cli::parse_from(args);

    match cli.command {
        Command::Install(install) => {
            let (_, _) = install.run();
        }
        Command::Build(mut build) => build.run(),
        Command::Toml(toml) => toml.run(),
        Command::Show(show) => show.run(),
        Command::DumpUsage => dump_full_usage_for_readme(),
    }
}

/// All of the available subcommands for `cargo gpu`
#[derive(clap::Subcommand)]
enum Command {
    /// Install rust-gpu compiler artifacts.
    Install(Install),

    /// Compile a shader crate to SPIR-V.
    Build(Build),

    /// Compile a shader crate according to the `cargo gpu build` parameters
    /// found in the given toml file.
    Toml(Toml),

    /// Show some useful values.
    Show(Show),

    /// A hidden command that can be used to recursively print out all the subcommand help messages:
    ///   `cargo gpu dump-usage`
    /// Useful for updating the README.
    #[clap(hide(true))]
    DumpUsage,
}

#[derive(clap::Parser)]
#[clap(author, version, about, subcommand_required = true)]
pub(crate) struct Cli {
    /// The command to run.
    #[clap(subcommand)]
    command: Command,
}

fn cache_dir() -> std::path::PathBuf {
    directories::BaseDirs::new()
        .unwrap_or_else(|| {
            log::error!("could not find the user home directory");
            panic!("cache_dir failed");
        })
        .cache_dir()
        .join("rust-gpu")
}

/// Location of the target spec metadata files
fn target_spec_dir() -> std::path::PathBuf {
    let dir = cache_dir().join("target-specs");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Convenience function for internal use. Dumps all the CLI usage instructions. Useful for
/// updating the README.
fn dump_full_usage_for_readme() {
    use clap::CommandFactory as _;
    let mut command = Cli::command();

    let mut buffer: Vec<u8> = Vec::default();
    command.build();

    write_help(&mut buffer, &mut command, 0);
    println!("{}", String::from_utf8(buffer).unwrap());
}

/// Recursive function to print the usage instructions for each subcommand.
fn write_help(buffer: &mut impl std::io::Write, cmd: &mut clap::Command, _depth: usize) {
    if cmd.get_name() == "help" {
        return;
    }

    let mut command = cmd.get_name().to_owned();
    writeln!(
        buffer,
        "\n* {}{}",
        command.remove(0).to_uppercase(),
        command
    )
    .unwrap();
    writeln!(buffer).unwrap();
    cmd.write_long_help(buffer).unwrap();

    for sub in cmd.get_subcommands_mut() {
        writeln!(buffer).unwrap();
        #[expect(clippy::used_underscore_binding, reason = "Used in recursion only")]
        write_help(buffer, sub, _depth + 1);
    }
}

#[cfg(test)]
mod test {
    use spirv::Spirv;

    use super::*;

    #[test]
    fn cached_checkout_dir_sanity() {
        // Test that
        let spirv = Spirv::default();
        let dir = spirv.cached_checkout_path();
        let name = dir
            .file_name()
            .unwrap()
            .to_str()
            .map(std::string::ToString::to_string)
            .unwrap();
        assert_eq!(
            "git_https___github_com_Rust-GPU_rust-gpu_git+nightly-2024-04-24",
            &name
        );
    }

    #[test]
    fn builder_from_params() {
        let shader_crate = std::path::PathBuf::from("../shader-crate-template");
        let output_dir = std::path::PathBuf::from("../shader-crate-template/shaders");
        let args = [
            "target/debug/cargo-gpu",
            "build",
            "--shader-crate",
            &format!("{}", shader_crate.display()),
            "--output-dir",
            &format!("{}", output_dir.display()),
        ];
        if let Cli {
            command: Command::Build(mut build),
        } = Cli::parse_from(args)
        {
            assert_eq!(shader_crate, build.shader_crate);
            assert_eq!(output_dir, build.output_dir);

            // TODO:
            // What's the best way to reset caches for this? For example we could add a
            // `--force-spirv-cli-rebuild`, but that would slow down each test. But without
            // something like that we might not be getting actual idempotent tests.
            build.run();
        } else {
            panic!("was not a build command");
        }
    }
}
