# cargo-gpu
Command line tool for building Rust shaders using rust-gpu.

## Getting Started

### Installation
To install the tool ensure you have `rustup`. Then run: 

```
cargo install --git https://github.com/rust-gpu/cargo-gpu
```

After that you can use `cargo gpu` to compile your shader crates with: 

```
cargo gpu build
```

This plain invocation will compile the crate in the current directory and 
place the compiled shaders in the current directory.

Use `cargo gpu help` to see other options :)

### Next Steps

You can try this out using the example repo at <https://github.com/rust-GPU/shader-crate-template>.
Keep in mind <https://github.com/rust-GPU/shader-crate-template> is _not_ yet a cargo generate template,
it's just a normal repo.

```
git clone https://github.com/rust-GPU/shader-crate-template
cd shader-crate-template
cargo gpu build
```

## Usage

```
Commands:
  install  Install rust-gpu compiler artifacts
  build    Compile a shader crate to SPIR-V
  toml     Compile a shader crate according to the `cargo gpu build` parameters found in the given toml file
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help

  -V, --version
          Print version


* Install

Install rust-gpu compiler artifacts

Usage: cargo-gpu install [OPTIONS]

Options:
      --spirv-builder <SPIRV_BUILDER>
          spirv-builder dependency, written just like in a Cargo.toml file

          [default: "{ git = \"https://github.com/Rust-GPU/rust-gpu.git\" }"]

      --rust-toolchain <RUST_TOOLCHAIN>
          Rust toolchain channel to use to build `spirv-builder`.

          This must match the `spirv_builder` argument.

          [default: nightly-2024-04-24]

      --force-spirv-cli-rebuild
          Force `spirv-builder-cli` and `rustc_codegen_spirv` to be rebuilt

  -h, --help
          Print help (see a summary with '-h')


* Build

Compile a shader crate to SPIR-V

Usage: cargo-gpu build [OPTIONS]

Options:
      --spirv-builder <SPIRV_BUILDER>
          spirv-builder dependency, written just like in a Cargo.toml file

          [default: "{ git = \"https://github.com/Rust-GPU/rust-gpu.git\" }"]

      --rust-toolchain <RUST_TOOLCHAIN>
          Rust toolchain channel to use to build `spirv-builder`.

          This must match the `spirv_builder` argument.

          [default: nightly-2024-04-24]

      --force-spirv-cli-rebuild
          Force `spirv-builder-cli` and `rustc_codegen_spirv` to be rebuilt

      --shader-crate <SHADER_CRATE>
          Directory containing the shader crate to compile

          [default: ./]

      --shader-target <SHADER_TARGET>
          Shader target

          [default: spirv-unknown-vulkan1.2]

      --no-default-features
          Set cargo default-features

      --features <FEATURES>
          Set cargo features

  -o, --output-dir <OUTPUT_DIR>
          Path to the output directory for the compiled shaders

          [default: ./]

  -h, --help
          Print help (see a summary with '-h')


* Toml

Compile a shader crate according to the `cargo gpu build` parameters found in the given toml file

Usage: cargo-gpu toml [PATH]

Arguments:
  [PATH]
          Path to a workspace or package Cargo.toml file.

          Must include a [[workspace | package].metadata.rust-gpu.build] section where
          arguments to `cargo gpu build` are listed.

          Path arguments like `output-dir` and `shader-manifest` must be relative to
          the location of the Cargo.toml file.

          Example:

          ```toml
              [package.metadata.rust-gpu.build.spirv-builder]
              git = "https://github.com/Rust-GPU/rust-gpu.git"
              rev = "0da80f8"

              [package.metadata.rust-gpu.build]
              output-dir = "shaders"
              shader-manifest = "shaders/manifest.json"
          ```

          Calling `cargo gpu toml {path/to/Cargo.toml}` with a Cargo.toml that
          contains the example above would compile the crate and place the compiled
          `.spv` files and manifest in a directory "shaders".

          [default: ./Cargo.toml]

Options:
  -h, --help
          Print help (see a summary with '-h')

```
