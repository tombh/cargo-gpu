# cargo-gpu
Command line tool for building Rust shaders using rust-gpu.

## Getting Started

### installation
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
