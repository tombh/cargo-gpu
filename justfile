[group: 'ci']
build-shader-template:
  cargo install --path crates/cargo-gpu
  cargo gpu install --shader-crate crates/shader-crate-template
  cargo gpu build --shader-crate crates/shader-crate-template --output-dir test-shaders
  ls -lah test-shaders
  cat test-shaders/manifest.json

[group: 'ci']
setup-lints:
	cargo binstall cargo-shear

[group: 'ci']
lints:
  cargo clippy -- --deny warnings
  cargo fmt --check
  # Look for unused crates
  cargo shear

