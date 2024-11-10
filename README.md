# cargo-gpu
Command line tool for building Rust shaders using rust-gpu.

## Getting Started

### installation
To install the tool ensure you have `rustup`. Then run: 

```
cargo +nightly-2024-04-24 install --git https://github.com/rust-gpu/cargo-gpu
```

After that you can use `cargo gpu` to compile your shader crates with: 

```
cargo gpu
```

This plain invocation will compile the crate in the current directory and 
place the compiled shaders in the current directory.

Use `cargo gpu --help` to see other options :)

### Next Steps

TODO(schell) - finish up the cargo-generate template repo and explain it here.
