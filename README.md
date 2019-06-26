# kernel
A custom OS kernel that followed (then diverged from) Philipp Oppermann's tutorial on writing an OS in Rust

## What this kernel has (as of June 25, 2019)

This kernel can:

* Interpret commands via an extensible command console opened after boot
* Output to both serial ports and the VGA buffer
* Enumerate PCI devices and configure internal data structures for them
* Handle most CPU exceptions (#of, #br, #df, #pf, ...)
* Configure the RTC to tick at a rate of 122 Us

This kernel also has a fully asynchronous keyboard input driver, and mouse input is coming soon!

## How to build and boot

1. Install your compiler toolchain of choice
2. Install rustup and run the following commands:

Add the nightly rust toolchain:

```
rustup toolchain add nightly
```

Install LLVM preview tools:

```
rustup component add llvm-tools-preview
```

Install cargo xbuild:

```
cargo install cargo-xbuild
```

Install bootimage:

```
cargo install bootimage
```

Install rust-src component:

```
rustup component add rust-src
```

3. Change into the directory where you cloned this repository and build:

```cargo xbuild```

or

```
cargo xrun # run the kernel after building
```

Note: Don't use cargo run/build! It will fail!

To build an image:

```
bootimage build
```

I am working on documentation right now, while juggling kernel updates, so docs may take a while.
