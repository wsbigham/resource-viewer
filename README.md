# Resource View

A lightweight desktop application written in Rust to monitor and visualize real-time system resource usage (CPU, memory, disk, network).

## Running Locally

Ensure you have [Rust installed](https://rustup.rs/), then start the app by running:

```bash
cargo run --release
```

## Exporting / Building Binaries

To build a standalone executable for distribution, run the build command for your target platform. Binaries will be located in the `target/.../release/` directory.

### Linux
```bash
cargo build --release
```

### Windows
If you are on Windows, simply run `cargo build --release`. To cross-compile from Linux/macOS:
```bash
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

### macOS
For Apple Silicon (M-series):
```bash
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

For Intel Macs:
```bash
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
```

*(Note: Cross-compiling for macOS from Linux/Windows or cross-compiling for Windows from Linux/macOS may require installing host-specific linkers like `mingw-w64` or osxcross.)*
