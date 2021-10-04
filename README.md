# kernel
A custom OS kernel that followed (then diverged from) Philipp Oppermann's tutorial on writing an OS in Rust

## Contributions

I encourage anyone to help out when they can. I'd love contributions; a lot needs to get done, but we're in no rush. I don't expect this OS to rival Linux, any of the BSDs, etc., but I do hope it gets somewhere.

Note: please sign each commit and agree to the [Developer Certificate of Origin](https://developercertificate.org). Pull requests are also welcome.

For steps on contributing, see [CONTRIBUTING.md](CONTRIBUTING.md). Please remember to abide by the [code of conduct](code_of_conduct.md).

## Building

The commands to build this are as follows:

```rust
rustup toolchain install nightly
rustup component add llvm-tools-preview rust-src
cargo install cargo-make
# Clone repo...
cargo make
```

Note that you'll need Qemu installed.

## Unit tests

We currently have no unit tests because for that to work each test would need to be its own mini-kernel. Though I'd like to build some sometime, I'm more focused on testing everything as I go.

## To do and what exists

Please see [this issue](https://github.com/ethindp/kernel/issues/2) for a small list of the tasks that need to get done and that've already been done.

## Goal

This aims to be a small, safe, secure, and fast microkernel OS. The goal is to limit the amount of syscalls as much as possible and to put as much as we can into userspace without degrading performance in any way.

The idea is that drivers run in userspace. When the system wants to do something, it requests a shared memory buffer to the driver in question and sends a request to the driver. The driver executes the request and returns a response, notifying the application. The shared memory buffer remains open so long as the application runs.

A problem arises with the above idea though: TLB shootdowns and flushes. A TLB shootdown is required when the context of the processor is changed in some manner. When one processor alters memory that has been cached by all the other processors in the system, all the other processors must flush their TLBs so that they can sync with the processor that caused the change. This action occurs many times: every time a process is haulted and a new one needs to run, every time a page table entry is altered, and so on.

Intel and AMD on x86 (as well as systems before their time and even other architectures) have come up with various solutions to this problem to maximize performance and to limit these operations. This includes memory protection keys and process-context identifiers (PCIDs). This kernel aims to use all of the resources that a system implements to minimize TLB shootdowns and costs of context switches and PTE changes.


