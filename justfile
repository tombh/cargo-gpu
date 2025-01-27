[group: 'ci']
build-shader-template:
  scripts/build_shader_template.sh

[group: 'ci']
setup-lints:
	cargo binstall cargo-shear

[group: 'ci']
lints:
  cargo clippy -- --deny warnings
  cargo fmt --check
  # Look for unused crates
  cargo shear

