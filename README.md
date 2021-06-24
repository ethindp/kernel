# kernel
A custom OS kernel that followed (then diverged from) Philipp Oppermann's tutorial on writing an OS in Rust

## Contributions

I encourage anyone who wishes to help to help out when they can. I'd love contributions; a lot needs to get done, but we're in no rush. I don't expect this OS to rival Linux, any of the BSDs, etc., but I do hope it gets somewhere.

Note: Standard Github contribution guidelines apply.

## Building

The commands to build this are as follows:

```rust
rustup toolchain install nightly
rustup component add llvm-tools-preview rust-src
cargo install bootimage
# Clone repo...
cargo run
```

Note that you'll need Qemu installed.

## Unit tests

We currently have no unit tests because for that to work each test would need to be its own mini-kernel. Though I'd like to build some sometime, I'm more focused on testing everything as I go.

