# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Note that, when a release point is hit where many changes are made, these changes are sorted, as best as possible, by date, with the newest changes first. This will prevent confusion if changes are reversed during the development cycle.

## [Unreleased]

### 2020-06-04

- Deps: update x86_64 to 0.9.6; bootloader to 0.8.9; uart_16550 to 0.2.4; pc-keyboard to 0.5.0; cpuio to 0.2.0; bit_field to 0.10.0; rusty-asm to 0.2.1; zerocopy to 0.2.8; and spin to 0.5.2
- Deps: add aes-gcm, aes-gcm-siv, aes-siv, crypto_box, raw-cpuid, uint, vga, and ps2-mouse dependencies
- crypto: add CRC 8, 16, 32, and 64 checksum algorithms
- crypto::crc: Add CRC 16 variants CCITT, DNP, Kermit, Sick, Modbus, and Xmodem
- crypto: add FNV-1 and FNV-1A hashing algorithms of sizes 32, 64, 128, 256, 512, and 1024 bits; sizes beyond 128 bits are handled by the uint crate
- misc: Reduce compilation features to just enabling SSE and software floating-point
- interrupts: split interrupt initialization routine into two stages: stage 1 initializes the PIC and stage 2 sets up the APIC if supported
- interrupts: Add APIC support
- interrupts: remove the PIC crate and handle PIC configuration and usage within the kernel itself using CPU IO
- interrupts: Add functions `is_apic_available()`, `apic_addr()`, and `signal_eoi()` for use in interrupt handlers and interrupt management routines only
- interrupts: do not force inlining of `is_apic_available()`, `apic_addr()`, and `signal_eoi()` using `#[inline]`
- memory: alter memory frames list to an `alloc::LinkedList<>` since the kernel heap is initialized
- init: initialize kernel heap before all else (except printing loading message)
- init: require RDRAND support
- drivers::fs: Add beginnings of EXT2 file system support
- drivers::storage: Add GPT parsing support

### 2020-03-20

- Deps: update goblin to v. 0.2.1
- Page alloc: Use a static array of frames for backing storage of memory frames to speed up page frame allocation (currently a size of 65535 frames is used)
- Heap: search for a random memory address for the heap that is modulo 32767 instead of placing it at a constant address
- Malloc: swap memory allocator to use a linked list allocator instead of slabs
- Security: compile with retpoline mitigations (features: retpoline, retpoline-external-thunk, retpoline-indirect-branches, retpoline-indirect-calls)
- Misc: Add MMX, SSE, SSE2/3/4/4.1/4.2 and SSSE3 features to compilation

### 2020-03-03

#### Changed

- Update the following dependencies to the versions listed:
    - bootloader: v0.8.8
    - goblin: v0.2.0
    - linked_list_allocator: v0.6.6
    - proc-macro2: v1.0.9
    - register: v0.4.2
    - syn: v1.0.16
    - uart_16550: v0.2.4
    - x86_64: v0.9.5

#### Removed

- Remove the following dependencies from the cargo dependency tree: array-init, cast, nodrop, rustc_version, semver, semver-parser, ux
