#!/bin/sh

# Unix and OSX have different versions of this argument.
SED_INPLACE="-i"
if [ "$(uname)" = "Darwin" ]; then
	SED_INPLACE="-i ''"
fi

# Matching windows paths when they're at root (/) causes problems with how we canoniclaize paths.
TMP_DIR=".$(mktemp --directory)"

cargo install --path crates/cargo-gpu
cargo gpu install --shader-crate crates/shader-crate-template --auto-install-rust-toolchain

# We change the output directory in the shader crate's `Cargo.toml` rather than just using the simpler
# `--output-dir` CLI arg, because we want to smoke test that setting config in `Cargo.toml` works.
sed "$SED_INPLACE" "s#^output-dir =.*#output-dir = \"$TMP_DIR\"#" crates/shader-crate-template/Cargo.toml

cargo gpu build --shader-crate crates/shader-crate-template --force-spirv-cli-rebuild
ls -lah "$TMP_DIR"
cat "$TMP_DIR"/manifest.json
